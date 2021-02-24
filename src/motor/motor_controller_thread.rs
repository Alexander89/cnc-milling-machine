#![allow(clippy::too_many_arguments)]
use super::motor_controller::{ExternalInput, ExternalInputRequest};
use super::task::{
    CalibrateType, InnerTask, InnerTaskCalibrate, InnerTaskProduction, ManualInstruction, Task,
};
use super::Motor;
use crate::gnc::NextMiscellaneous;
use crate::io::{Actor, Switch};
use crate::types::{
    CircleDirection, CircleStep, CircleStepCCW, CircleStepCW, CircleStepDir, Direction, Location,
    MachineState, SteppedCircleMovement, SteppedLinearMovement,
    SteppedMoveType::{Circle, Linear, Rapid},
};
use std::{
    fmt::Debug,
    ops::{Div, Mul},
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering::Relaxed},
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

#[derive(Debug)]
pub struct MotorControllerThread {
    motor_x: Motor,
    motor_y: Motor,
    motor_z: Motor,

    z_calibrate: Option<Switch>,

    x_step: Arc<AtomicI64>,
    y_step: Arc<AtomicI64>,
    z_step: Arc<AtomicI64>,

    current_task: Option<InnerTask>,
    current_location: Location<f64>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    steps_todo: Arc<AtomicI64>,
    steps_done: Arc<AtomicI64>,
    task_query: Arc<Mutex<Vec<Task>>>,
    manual_instruction_receiver: Receiver<ManualInstruction>,

    on_off_state: Arc<AtomicBool>,
    on_off: Option<Actor>,
    switch_on_off_delay: f64,

    external_input_enabled: bool,
    external_input_required: bool,
    external_input_receiver: Receiver<ExternalInput>,
    external_input_request_sender: Sender<ExternalInputRequest>,
}

impl MotorControllerThread {
    pub fn new(
        x_step: Arc<AtomicI64>,
        y_step: Arc<AtomicI64>,
        z_step: Arc<AtomicI64>,
        motor_x: Motor,
        motor_y: Motor,
        motor_z: Motor,
        z_calibrate: Option<Switch>,
        current_task: Option<InnerTask>,
        current_location: Location<f64>,
        state: Arc<AtomicU32>,
        steps_todo: Arc<AtomicI64>,
        steps_done: Arc<AtomicI64>,
        cancel_task: Arc<AtomicBool>,
        task_query: Arc<Mutex<Vec<Task>>>,
        manual_instruction_receiver: Receiver<ManualInstruction>,
        external_input_enabled: bool,
        external_input_receiver: Receiver<ExternalInput>,
        external_input_request_sender: Sender<ExternalInputRequest>,
        on_off_state: Arc<AtomicBool>,
        on_off: Option<Actor>,
        switch_on_off_delay: f64,
    ) -> MotorControllerThread {
        MotorControllerThread {
            x_step,
            y_step,
            z_step,
            motor_x,
            motor_y,
            motor_z,
            z_calibrate,
            current_task,
            current_location,
            state,
            steps_todo,
            steps_done,
            cancel_task,
            task_query,
            manual_instruction_receiver,
            on_off_state,
            on_off,
            switch_on_off_delay,
            external_input_enabled,
            external_input_required: false,
            external_input_receiver,
            external_input_request_sender,
        }
    }
    pub fn get_pos(&self) -> Location<i64> {
        Location {
            x: self.x_step.load(Relaxed),
            y: self.y_step.load(Relaxed),
            z: self.z_step.load(Relaxed),
        }
    }
    fn get_step_sizes(&self) -> Location<f64> {
        Location {
            x: self.motor_x.get_step_size(),
            y: self.motor_y.get_step_size(),
            z: self.motor_z.get_step_size(),
        }
    }
    pub fn run(&mut self) {
        let mut last_step: Option<SystemTime> = None; //SystemTime::now();
        let mut curve_close_to_destination = false;
        let mut last_distance_to_destination = 100;
        let mut q_ptr = 0;

        let mut stepper_delay = 0.0f64; // sec a motor is block during a linier move

        let mut calculate_z_phase = 0i32;
        let mut calibrate_z_pos_temp = Location::<i64>::default();

        let program_task: u32 = MachineState::ProgramTask.into();
        let calibrate: u32 = MachineState::Calibrate.into();

        loop {
            // read it but drop it to avoid a command jam after program or calibration completed
            let next_manual_task = self.manual_instruction_receiver.try_recv();
            if self.state.load(Relaxed) != program_task && self.state.load(Relaxed) != calibrate {
                match next_manual_task {
                    Ok(ManualInstruction::Movement(next_task)) => {
                        let max_speed = next_task.speed;
                        let task = Task::Manual(next_task);
                        self.state.store(task.machine_state().into(), Relaxed);
                        self.current_task = InnerTask::from_task(
                            task,
                            self.get_pos(),
                            self.get_step_sizes(),
                            max_speed,
                        );
                    }
                    Ok(ManualInstruction::Miscellaneous(next_miscellaneous)) => {
                        match next_miscellaneous {
                            NextMiscellaneous::SwitchOn => {
                                self.switch_on();
                                self.current_task = None;
                            }
                            NextMiscellaneous::SwitchOff => {
                                self.switch_off();
                                self.current_task = None;
                            }
                            _ => (),
                        }
                    }
                    Err(_) => (),
                };
            }

            // check flag to cancel current task
            if self.cancel_task.load(Relaxed) {
                self.cancel_task.store(false, Relaxed);
                self.current_task = None;
                self.steps_todo.store(0, Relaxed);
                self.steps_done.store(0, Relaxed);
                println!("MotorControllerThread: cancel task");
            };

            // check flag if machine wait for external input (tool change, new stock, turn stock, speed changed, ...)
            if self.external_input_required {
                // try_recv() => sleep + continue; To keep the cancel task in the loop
                match self.external_input_receiver.try_recv() {
                    Ok(ExternalInput::ToolChanged) | Ok(ExternalInput::SpeedChanged) => {
                        self.external_input_required = false
                    }
                    Ok(ExternalInput::StockTurned) => {
                        // check fix points somehow ??
                        self.external_input_required = false;
                    }
                    Ok(ExternalInput::NewStock) => {
                        // calibrate height?
                        self.external_input_required = false;
                    }
                    Err(_) => {
                        thread::sleep(Duration::new(0, 10_000));
                        continue;
                    }
                }
            }

            match &self.current_task {
                Some(InnerTask::Production(InnerTaskProduction {
                    start_time,
                    move_type,
                    from,
                    destination,
                    ..
                })) => match move_type {
                    Linear(SteppedLinearMovement {
                        delta,
                        speed,
                        distance,
                    })
                    | Rapid(SteppedLinearMovement {
                        delta,
                        speed,
                        distance,
                    }) => {
                        if let Ok(elapsed) = start_time.elapsed() {
                            if *speed == 0.0f64 || *distance == 0.0f64 {
                                self.current_task = None;
                                continue;
                            }
                            let already_moved_this_task = (self.get_pos() - from.clone()).abs();

                            let complete_runtime =
                                Duration::from_secs_f64(*distance / (*speed / 60.0f64)).as_micros()
                                    as u64;

                            // remove time waiting for the steppers to ramp-up.
                            // Other wise the rest of the track will try compensate it with driving to fast
                            let runtime: u64 = (elapsed.as_micros()
                                - Duration::from_secs_f64(stepper_delay).as_micros())
                                as u64;

                            let (x, y, z) = delta.split();

                            if delta.abs() == already_moved_this_task.clone()
                                || (x == 0 && y == 0 && z == 0)
                            {
                                self.current_task = None;
                            } else {
                                if x != 0
                                    && x.abs() as u64 != already_moved_this_task.x
                                    && runtime
                                        > complete_runtime / x.abs() as u64
                                            * already_moved_this_task.x
                                {
                                    let dir = if x > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    stepper_delay += self.motor_x.step(dir);
                                }

                                if y != 0
                                    && y.abs() as u64 != already_moved_this_task.y
                                    && runtime
                                        > complete_runtime / y.abs() as u64
                                            * already_moved_this_task.y
                                {
                                    let dir = if y > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    stepper_delay += self.motor_y.step(dir);
                                }

                                if z != 0
                                    && z.abs() as u64 != already_moved_this_task.z
                                    && runtime
                                        > complete_runtime / z.abs() as u64
                                            * already_moved_this_task.z
                                {
                                    let dir = if z > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    stepper_delay += self.motor_z.step(dir);
                                }
                            }
                        } else {
                            self.current_task = None;
                            println!("failed at start_time.elapsed()");
                        }
                    }
                    Circle(SteppedCircleMovement {
                        turn_direction,
                        center,
                        radius_sq,
                        step_sizes,
                        speed,
                        step_delay,
                        ..
                    }) => {
                        if last_step.is_some()
                            && last_step.unwrap().elapsed().unwrap().as_secs_f64() <= *step_delay
                        {
                            continue;
                        }
                        last_step = Some(SystemTime::now());

                        let abs_center: Location<i64> = from.clone() + center.clone();
                        let rel_to_center = self.get_pos() - abs_center.clone();

                        let step_dir: CircleStep = match turn_direction {
                            CircleDirection::CW => {
                                let next_step: CircleStepCW = rel_to_center.into();
                                next_step.into()
                            }
                            CircleDirection::CCW => {
                                let next_step: CircleStepCCW = rel_to_center.into();
                                next_step.into()
                            }
                        };
                        match step_dir.main {
                            CircleStepDir::Right => self.motor_x.step(Direction::Right),
                            CircleStepDir::Down => self.motor_y.step(Direction::Left),
                            CircleStepDir::Left => self.motor_x.step(Direction::Left),
                            CircleStepDir::Up => self.motor_y.step(Direction::Right),
                        };
                        let pos_before_move = self.get_pos();
                        let delta_before_op: Location<f64> =
                            (pos_before_move.clone() - abs_center.clone()).into();
                        let delta_before_op_step_correct =
                            delta_before_op.clone() * step_sizes.clone();
                        let delta_radius_before_op =
                            radius_sq - delta_before_op_step_correct.distance_sq();
                        let pos_after_move = pos_before_move
                            + match step_dir.opt {
                                CircleStepDir::Right => Location::<i64>::new(1, 0, 0),
                                CircleStepDir::Down => Location::<i64>::new(0, -1, 0),
                                CircleStepDir::Left => Location::<i64>::new(-1, 0, 0),
                                CircleStepDir::Up => Location::<i64>::new(0, 1, 0),
                            };

                        let delta_after_op: Location<f64> =
                            (pos_after_move - abs_center.clone()).into();
                        let delta_after_op_step_correct =
                            delta_after_op.clone() * step_sizes.clone();
                        let delta_radius_after_op =
                            radius_sq - delta_after_op_step_correct.distance_sq();
                        if delta_radius_before_op.abs() > delta_radius_after_op.abs() {
                            match step_dir.opt {
                                CircleStepDir::Right => self.motor_x.step(Direction::Right),
                                CircleStepDir::Down => self.motor_y.step(Direction::Left),
                                CircleStepDir::Left => self.motor_x.step(Direction::Left),
                                CircleStepDir::Up => self.motor_y.step(Direction::Right),
                            };
                        }

                        let dist_destination: Location<i64> = destination.clone() - self.get_pos();
                        let dist_to_dest = dist_destination.clone().distance_sq();
                        if dist_to_dest < 25 * 25 && !curve_close_to_destination {
                            curve_close_to_destination = true;
                        }

                        if curve_close_to_destination && dist_to_dest > last_distance_to_destination
                        {
                            let dist_destination_f64: Location<f64> =
                                dist_destination.clone().into();
                            let distance = (dist_destination_f64 * step_sizes.clone()).distance();

                            self.current_task = Some(InnerTask::Production(InnerTaskProduction {
                                destination: destination.clone(),
                                from: self.get_pos(),
                                start_time: SystemTime::now(),
                                move_type: Linear(SteppedLinearMovement {
                                    delta: dist_destination.clone(),
                                    distance,
                                    speed: *speed,
                                }),
                            }));
                            curve_close_to_destination = false;
                            last_distance_to_destination = 100;
                        } else if curve_close_to_destination {
                            last_distance_to_destination = dist_to_dest;
                        }

                        if dist_destination.distance_sq() == 0 {
                            curve_close_to_destination = false;
                            last_distance_to_destination = 100;
                            println!("at destination, set currentTask to NONE");
                            self.current_task = None;
                        }
                    }
                },
                Some(InnerTask::Calibrate(InnerTaskCalibrate {
                    z,
                    from,
                    start_time,
                    step_sizes,
                    ..
                })) => {
                    let runtime = start_time.elapsed().unwrap().as_micros() as u64;
                    let move_in_task = (self.get_pos() - from.clone()).abs();
                    let z_steps = move_in_task.z;

                    if runtime > z_steps * 4_000 {
                        match z {
                            CalibrateType::Min => {
                                if self.motor_z.is_blocked() == Some(Direction::Left) {
                                    self.current_task = None;
                                } else {
                                    self.motor_z.step(Direction::Left);
                                }
                            }
                            CalibrateType::Max => {
                                if self.motor_z.is_blocked() == Some(Direction::Right) {
                                    self.current_task = None;
                                } else {
                                    self.motor_z.step(Direction::Right);
                                }
                            }
                            CalibrateType::Middle => {
                                match calculate_z_phase {
                                    0 => {
                                        // move to min
                                        if self.motor_z.is_blocked() == Some(Direction::Left) {
                                            calibrate_z_pos_temp = self.get_pos();
                                            calculate_z_phase = 1;
                                        } else {
                                            self.motor_z.step(Direction::Left);
                                        }
                                    }
                                    1 => {
                                        // move to max
                                        if self.motor_z.is_blocked() == Some(Direction::Right) {
                                            let delta = (self.get_pos()
                                                - calibrate_z_pos_temp.clone())
                                            .div(2);
                                            let delta_f64: Location<f64> = delta.clone().into();
                                            let distance =
                                                delta_f64.mul(step_sizes.clone()).distance();

                                            self.current_task =
                                                Some(InnerTask::Production(InnerTaskProduction {
                                                    destination: delta.clone(),
                                                    from: self.get_pos(),
                                                    start_time: SystemTime::now(),
                                                    move_type: Linear(SteppedLinearMovement {
                                                        delta, // or (calibrate_z_pos_temp - self.get_pos()).div(2)
                                                        distance,
                                                        speed: 360.0f64,
                                                    }),
                                                }));
                                        } else {
                                            self.motor_z.step(Direction::Right);
                                        }
                                    }
                                    _ => (),
                                };
                                println!("!!!!!!! untested implementation. Do not use in production !!!!!!!");
                                self.current_task = None;
                            }
                            CalibrateType::ContactPin => {
                                if self.z_calibrate.is_none()
                                    || self.z_calibrate.as_mut().unwrap().is_closed()
                                    || self.motor_z.is_blocked() == Some(Direction::Right)
                                {
                                    self.current_task = None;
                                } else {
                                    self.motor_z.step(Direction::Right);
                                };
                            }
                            CalibrateType::None => (),
                        }
                    }
                }
                Some(InnerTask::Miscellaneous(hardware_task)) => match hardware_task {
                    NextMiscellaneous::SwitchOn => {
                        self.switch_on();
                        thread::sleep(Duration::from_secs_f64(self.switch_on_off_delay));
                        self.current_task = None;
                    }
                    NextMiscellaneous::SwitchOff => {
                        self.switch_off();
                        thread::sleep(Duration::from_secs_f64(self.switch_on_off_delay));
                        self.current_task = None;
                    }
                    NextMiscellaneous::ToolChange(tool) if self.external_input_enabled => {
                        self.external_input_required = true;
                        self.external_input_request_sender
                            .send(ExternalInputRequest::ChangeTool(*tool))
                            .unwrap();
                    }
                    NextMiscellaneous::SpeedChange(speed) if self.external_input_enabled => {
                        self.external_input_required = true;
                        self.external_input_request_sender
                            .send(ExternalInputRequest::ChangeSpeed(*speed))
                            .unwrap();
                    }
                    NextMiscellaneous::ToolChange(_) | NextMiscellaneous::SpeedChange(_) => (),
                },
                None => {
                    calculate_z_phase = 0;
                    match self.task_query.lock() {
                        Ok(ref mut locked_queue) if locked_queue.len() > q_ptr => {
                            let next = locked_queue[q_ptr].clone();
                            self.state.store(next.machine_state().into(), Relaxed);
                            self.steps_done.store(q_ptr as i64, Relaxed);
                            self.steps_todo
                                .store((locked_queue.len() - q_ptr) as i64, Relaxed);
                            // println!("next {:?} {:?}", q_ptr, locked_queue.len() - q_ptr);
                            q_ptr += 1;

                            self.current_task = InnerTask::from_task(
                                next,
                                self.get_pos(),
                                self.get_step_sizes(),
                                40.0f64,
                            );
                        }
                        Ok(ref mut locked_queue) => {
                            locked_queue.clear();
                            q_ptr = 0;

                            let idle: u32 = MachineState::Idle.into();
                            if self.state.load(Relaxed) != idle {
                                self.steps_todo.store(0, Relaxed);
                                self.steps_done.store(0, Relaxed);
                                self.state.store(idle, Relaxed);
                            }
                            #[allow(clippy::drop_ref)]
                            drop(locked_queue);
                            thread::sleep(Duration::new(0, 10_000));
                        }
                        _ => {
                            thread::sleep(Duration::new(0, 10_000));
                        }
                    }
                }
            }
        }
    }

    fn switch_on(&mut self) {
        println!("switch on now");
        if let Some(actor) = self.on_off.as_mut() {
            actor.set_high()
        }
        self.on_off_state.store(true, Relaxed);
    }
    fn switch_off(&mut self) {
        println!("switch off now");
        if let Some(actor) = self.on_off.as_mut() {
            actor.set_low()
        }

        self.on_off_state.store(false, Relaxed);
    }
}
