pub mod executer;
pub mod hardware_controller_interface;
pub mod hardware_feedback;
pub mod instruction;
pub mod motor;
pub mod types;

use executer::{Executer, ManualSpindel, OnOffSpindel};
use instruction::{CalibrateType, Instruction};
use motor::{Motor, SettingsMotor};
use types::*;

use crate::{settings::Settings, types::{Direction, Location, MachineState}};
use crate::{
    io::Switch,
    types::{CircleStep, CircleStepCCW, CircleStepCW, CircleStepDir},
};
use crossbeam_channel::Sender as CbSender;
use std::{
    collections::LinkedList,
    fmt::Debug,
    sync::mpsc::Receiver,
    thread,
    time::{Duration, SystemTime},
};

use self::motor::{Driver, MockMotor, StepMotor};
pub use hardware_controller_interface::*;
pub use instruction::*;
pub use hardware_feedback::{PosData, HardwareFeedback};

#[derive(Clone, Debug)]
pub struct SettingsHardwareController {
    pub motor_x: SettingsMotor,
    pub motor_y: SettingsMotor,
    pub motor_z: SettingsMotor,
    pub calibrate_z_gpio: Option<u8>,
    pub on_off_gpio: Option<u8>,
    pub on_off_invert: bool,
    pub on_off_switch_delay: f64,
    pub pos_update_every_x_sec: f64,
    pub external_input_enabled: bool,
    pub dev_mode: bool,
}

impl SettingsHardwareController {
    #[allow(dead_code)]
    pub fn mock() -> SettingsHardwareController {
        SettingsHardwareController {
            motor_x: SettingsMotor {
                pull_gpio: 1,         //u8,
                dir_gpio: 1,          //u8,
                invert_dir: false,    //bool,
                ena_gpio: None,       //Option<u8>,
                end_left_gpio: None,  //Option<u8>,
                end_right_gpio: None, //Option<u8>,
                acceleration: 0.01,
                deceleration: 0.02,
            },
            motor_y: SettingsMotor {
                pull_gpio: 1,         //u8,
                dir_gpio: 1,          //u8,
                invert_dir: false,    //bool,
                ena_gpio: None,       //Option<u8>,
                end_left_gpio: None,  //Option<u8>,
                end_right_gpio: None, //Option<u8>,
                acceleration: 0.01,
                deceleration: 0.02,
            },
            motor_z: SettingsMotor {
                pull_gpio: 1,         //u8,
                dir_gpio: 1,          //u8,
                invert_dir: false,    //bool,
                ena_gpio: None,       //Option<u8>,
                end_left_gpio: None,  //Option<u8>,
                end_right_gpio: None, //Option<u8>,
                acceleration: 0.01,
                deceleration: 0.02,
            },
            calibrate_z_gpio: Some(1),
            on_off_gpio: Some(2),
            on_off_invert: false,
            on_off_switch_delay: 2.0,
            pos_update_every_x_sec: 0.2,
            external_input_enabled: true,
            dev_mode: true,
        }
    }
    pub fn from(settings: Settings) -> SettingsHardwareController {
        SettingsHardwareController {
            motor_x: SettingsMotor {
                pull_gpio: settings.motor_x.pull_gpio,         //u8,
                dir_gpio: settings.motor_x.dir_gpio,          //u8,
                invert_dir: settings.motor_x.invert_dir,    //bool,
                ena_gpio: settings.motor_x.ena_gpio,       //Option<u8>,
                end_left_gpio: settings.motor_x.end_left_gpio,  //Option<u8>,
                end_right_gpio: settings.motor_x.end_right_gpio, //Option<u8>,
                acceleration: settings.motor_x.acceleration / 1e6,
                deceleration: settings.motor_x.deceleration / 1e6,
            },
            motor_y: SettingsMotor {
                pull_gpio: settings.motor_y.pull_gpio,         //u8,
                dir_gpio: settings.motor_y.dir_gpio,          //u8,
                invert_dir: settings.motor_y.invert_dir,    //bool,
                ena_gpio: settings.motor_y.ena_gpio,       //Option<u8>,
                end_left_gpio: settings.motor_y.end_left_gpio,  //Option<u8>,
                end_right_gpio: settings.motor_y.end_right_gpio, //Option<u8>,
                acceleration: settings.motor_y.acceleration / 1e6,
                deceleration: settings.motor_y.deceleration / 1e6,
            },
            motor_z: SettingsMotor {
                pull_gpio: settings.motor_z.pull_gpio,         //u8,
                dir_gpio: settings.motor_z.dir_gpio,          //u8,
                invert_dir: settings.motor_z.invert_dir,    //bool,
                ena_gpio: settings.motor_z.ena_gpio,       //Option<u8>,
                end_left_gpio: settings.motor_z.end_left_gpio,  //Option<u8>,
                end_right_gpio: settings.motor_z.end_right_gpio, //Option<u8>,
                acceleration: settings.motor_z.acceleration / 1e6,
                deceleration: settings.motor_z.deceleration / 1e6,
            },
            calibrate_z_gpio: settings.calibrate_z_gpio,
            on_off_gpio: settings.on_off_gpio,
            on_off_invert: settings.on_off_invert,
            on_off_switch_delay: settings.on_off_switch_delay,
            pos_update_every_x_sec: settings.pos_update_every_x_sec,
            external_input_enabled: settings.external_input_enabled,
            dev_mode: settings.dev_mode,
        }
    }
}

pub struct HardwareController {
    motor_x: Motor,
    motor_y: Motor,
    motor_z: Motor,

    z_calibrate: Option<Switch>,

    state: MachineState,
    pre_paused_state: MachineState,
    instructions_done: u32,
    instructions_todo: u32,

    current_instruction: Option<Instruction>,
    instruction_queue: LinkedList<Instruction>,

    instruction_receiver: Receiver<Instruction>,
    feedback_sender: CbSender<HardwareFeedback>,

    executer: Box<dyn Executer>,
    settings: SettingsHardwareController,
}

impl HardwareController {
    pub fn new(
        settings: SettingsHardwareController,
        instruction_receiver: Receiver<Instruction>,
        feedback_sender: CbSender<HardwareFeedback>,
    ) -> HardwareController {
        let executer = settings
            .on_off_gpio
            .map(|pin| {
                Box::new(OnOffSpindel::new(
                    pin,
                    settings.on_off_invert,
                    settings.on_off_switch_delay,
                )) as Box<dyn Executer>
            })
            .unwrap_or_else(|| Box::new(ManualSpindel::new(settings.on_off_switch_delay)));

        let z_calibrate = settings.calibrate_z_gpio.map(|pin| Switch::new(pin, false));

        let driver_x: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_x.clone()))
        };
        let motor_x = Motor::new("x".to_string(), settings.motor_x.clone(), driver_x);

        let driver_y: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_y.clone()))
        };
        let motor_y = Motor::new("y".to_string(), settings.motor_y.clone(), driver_y);

        let driver_z: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_z.clone()))
        };
        let motor_z = Motor::new("z".to_string(), settings.motor_z.clone(), driver_z);

        HardwareController {
            motor_x,
            motor_y,
            motor_z,
            z_calibrate,

            state: MachineState::Idle,
            pre_paused_state: MachineState::Idle,

            instructions_done: 0,
            instructions_todo: 0,

            current_instruction: None,
            instruction_queue: LinkedList::default(),

            instruction_receiver,
            feedback_sender,

            executer,
            settings,
        }
    }

    fn apply_settings(&mut self, settings: SettingsHardwareController) {
        self.executer = settings
            .on_off_gpio
            .map(|pin| {
                Box::new(OnOffSpindel::new(
                    pin,
                    settings.on_off_invert,
                    settings.on_off_switch_delay,
                )) as Box<dyn Executer>
            })
            .unwrap_or_else(|| Box::new(ManualSpindel::new(settings.on_off_switch_delay)));

        self.z_calibrate = settings.calibrate_z_gpio.map(|pin| Switch::new(pin, false));

        let driver_x: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_x.clone()))
        };
        self.motor_x = Motor::new("x".to_string(), settings.motor_x, driver_x);

        let driver_y: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_y.clone()))
        };
        self.motor_y = Motor::new("y".to_string(), settings.motor_y, driver_y);

        let driver_z: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new())
        } else {
            Box::new(StepMotor::from_settings(settings.motor_z.clone()))
        };
        self.motor_z = Motor::new("z".to_string(), settings.motor_z, driver_z);

        // reset the rest to get back to a fresh state
        self.set_state(MachineState::Idle);
        self.pre_paused_state = MachineState::Idle;

        self.instructions_done = 0;
        self.instructions_todo = 0;

        self.current_instruction = None;
        self.instruction_queue = LinkedList::default();
    }
    fn try_send(&mut self, msg: HardwareFeedback) {
        if let Err(e) = self.feedback_sender.try_send(msg) {
            println!("try-send failed {:?}", e);
        };
    }
    fn send(&mut self, msg: HardwareFeedback) {
        if let Err(e) = self.feedback_sender.send(msg) {
            println!("send failed {:?}", e);
        };
    }
    fn set_state(&mut self, state: MachineState) {
        if self.state != state {
            let msg = HardwareFeedback::State(state);
            self.try_send(msg);
            self.state = state;
        }
    }
    fn set_progress(&mut self, todo: u32, done: u32) {
        if self.instructions_todo != todo || self.instructions_done != done {
            let msg = HardwareFeedback::Progress(todo, done);
            self.try_send(msg);
            self.instructions_todo = todo;
            self.instructions_done = done;
        }
    }
    fn get_pos(&self) -> Location<i64> {
        Location {
            x: self.motor_x.get_pos(),
            y: self.motor_y.get_pos(),
            z: self.motor_z.get_pos(),
        }
    }

    pub fn run(&mut self) {
        let mut op_state = OpState::default();

        while !op_state.shutdown {
            op_state = self.process_input_channel(op_state);
            op_state = self.execute_current_instruction(op_state);
            op_state = self.switch_to_next_task(op_state);
            op_state = self.send_outbound_data(op_state);
        }
    }

    fn process_input_channel(&mut self, mut op_state: OpState) -> OpState {
        'input_loop: loop {
            match self.instruction_receiver.try_recv() {
                Ok(Instruction::Shutdown) => op_state.shutdown = true,
                Ok(i @ Instruction::ManualMovement(_))
                    if self.state != MachineState::Program
                        && self.state != MachineState::Calibrate =>
                {
                    self.current_instruction = None;
                    self.instruction_queue.clear();
                    self.instruction_queue.push_back(i);
                }
                Ok(Instruction::Emergency) => {
                    self.current_instruction = None;
                    self.instruction_queue.clear();
                    self.instructions_todo = 0;
                    self.instructions_done = 0;
                    println!("MotorControllerThread: Emergency");
                }
                Ok(Instruction::Stop) => {
                    // add Move Up and go stop spindle
                    self.current_instruction = None;
                    self.instruction_queue.clear();
                    self.instructions_todo = 0;
                    self.instructions_done = 0;
                    let p_start = self.get_pos();
                    let p_end = Location::new(p_start.x, p_start.y, 0);
                    self.instruction_queue.push_back(Instruction::Line(
                        InstructionLine::create_without_ramps(p_start, p_end, 0.000_01),
                    ));
                    println!("MotorControllerThread: Stop task");
                }
                Ok(Instruction::Pause) => {
                    // add stop spindle
                    self.pre_paused_state = self.state;
                    self.set_state(MachineState::Paused);
                    println!("MotorControllerThread: pause machine");
                }
                Ok(Instruction::Resume) => {
                    // add resume spindle
                    self.set_state(self.pre_paused_state);
                    println!("MotorControllerThread: Resume machine");
                }
                Ok(Instruction::Settings(settings)) => {
                    self.apply_settings(settings);
                    op_state = OpState::default();
                }
                Ok(Instruction::ToolChanged(InstructionToolChanged { tool_id, length })) => {
                    self.set_state(self.pre_paused_state);
                    println!("MotorControllerThread: ToolChanged");
                    op_state.tool_id = tool_id;
                    op_state.tool_length = length.unwrap_or_default();
                    self.current_instruction = None;
                }
                Ok(elt) => {
                    self.instruction_queue.push_back(elt);
                }
                Err(_) => break 'input_loop,
            };
        }
        op_state
    }

    fn execute_current_instruction(&mut self, op_state: OpState) -> OpState {
        match self.current_instruction.to_owned() {
            Some(Instruction::Condition(instruction)) => {
                self.probe_condition(&instruction, op_state)
            }
            Some(Instruction::ManualMovement(instruction)) => {
                self.exec_manual_input(&instruction, op_state)
            }
            Some(Instruction::Line(line)) => self.exec_line(&line, op_state),
            Some(Instruction::Curve(curve)) => self.exec_curve(&curve, op_state),
            Some(Instruction::Calibrate(instruction)) => {
                self.exec_calibrate_instruction(&instruction, op_state)
            }
            Some(Instruction::MotorOn(InstructionMotorOn { speed, cw })) => {
                self.executer_change_speed(speed, cw, op_state)
            }
            Some(Instruction::MotorOff) => self.executer_off(op_state),
            Some(Instruction::SetSpeed(InstructionSpeed { speed, cw })) => {
                self.executer_change_speed(speed, cw, op_state)
            }
            Some(Instruction::Delay(delay)) => self.exec_delay(delay, op_state),
            Some(Instruction::WaitFor(instruction)) => self.exec_wait_for(instruction, op_state),

            // user interactions
            Some(Instruction::Start)
            | Some(Instruction::Stop)
            | Some(Instruction::Pause)
            | Some(Instruction::Resume)
            | Some(Instruction::Emergency)
            | Some(Instruction::ToolChanged(_))
            | Some(Instruction::Settings(_))
            | Some(Instruction::Shutdown)
            | None => op_state,
        }
    }

    fn switch_to_next_task(&mut self, mut op_state: OpState) -> OpState {
        // check next task if no task in progress
        if self.current_instruction.is_none() {
            match self.instruction_queue.pop_front() {
                Some(instruction) => {
                    self.set_progress(
                        self.instruction_queue.len() as u32,
                        self.instructions_done + 1,
                    );
                    self.current_instruction = Some(instruction);
                    self.set_state(MachineState::Program);
                    op_state.start_time = Some(SystemTime::now());
                    op_state.start_pos = self.get_pos();
                }
                None => {
                    self.set_progress(0, 0);
                    self.set_state(MachineState::Idle);
                    thread::sleep(Duration::new(0, 10_000));
                }
            }
        }
        op_state
    }

    fn send_outbound_data(&mut self, mut op_state: OpState) -> OpState {
        if op_state.last_data_send.elapsed().unwrap().as_secs_f64()
            > self.settings.pos_update_every_x_sec
        {
            op_state.last_data_send = SystemTime::now();
            let position = self.get_pos();
            self.send(HardwareFeedback::Pos(PosData {
                x: position.x,
                y: position.y,
                z: position.z,
            }))
        }
        op_state
    }

    /** add tasks at front of the Queue if condition is true */
    fn probe_condition(
        &mut self,
        instruction: &InstructionCondition,
        op_state: OpState,
    ) -> OpState {
        let add_inst = match instruction.condition {
            InstructionConditions::DifferentTool(new_tool) => {
                (op_state.tool_id == new_tool) != instruction.invert
            }
            InstructionConditions::MotorOn => self.is_executer_on() != instruction.invert,
            InstructionConditions::MotorOff => self.is_executer_on() == instruction.invert,
        };
        if add_inst {
            let mut sub_inst = instruction.sub_instructions.clone();
            sub_inst.reverse();
            for elt in sub_inst.iter() {
                self.instruction_queue.push_front(elt.clone())
            }
        }
        op_state
    }

    fn exec_manual_input(
        &mut self,
        instruction: &InstructionManualMovement,
        op_state: OpState,
    ) -> OpState {
        // should not happen
        if op_state.start_time.is_none() {
            self.current_instruction = None;
            return op_state;
        }
        if instruction.is_stopped() {
            self.set_state(MachineState::Idle);
            self.current_instruction = None;
        } else {
            self.set_state(MachineState::Manual);
        }

        let elapsed = op_state.start_time.unwrap().elapsed().unwrap();
        let already_moved_this_task = self.get_pos() - op_state.start_pos.clone();

        let target = instruction.steps_in_time(elapsed);
        let (d_x, d_y, d_z) = (target - already_moved_this_task).split();

        if d_x != 0 {
            let _ = self.motor_x.step(d_x.into());
        }
        if d_y != 0 {
            let _ = self.motor_y.step(d_y.into());
        }
        if d_z != 0 {
            let _ = self.motor_z.step(d_z.into());
        }

        return op_state;
    }

    fn exec_line(&mut self, line: &InstructionLine, op_state: OpState) -> OpState {
        // should not happen
        if op_state.start_time.is_none() {
            self.current_instruction = None;
            return op_state;
        }
        let elapsed = op_state.start_time.unwrap().elapsed().unwrap();
        let already_moved_this_task = self.get_pos() - op_state.start_pos.clone();

        // check if already arrived -> complete
        if line.is_complete(already_moved_this_task.clone()) {
            self.current_instruction = None;
            return op_state;
        }

        let expected_pos = line.get_expected_steps_until_now(elapsed);
        let (d_x, d_y, d_z) = (expected_pos - already_moved_this_task).split();

        if d_x != 0 {
            if let Err(e) = self.motor_x.step(d_x.into()) {
                println!("run out of space {}", e)
            };
        }
        if d_y != 0 {
            if let Err(e) = self.motor_y.step(d_y.into()) {
                println!("run out of space {}", e)
            };
        }
        if d_z != 0 {
            if let Err(e) = self.motor_z.step(d_z.into()) {
                println!("run out of space {}", e)
            };
        }

        return op_state;
    }

    fn exec_curve(&mut self, curve: &InstructionCurve, mut op_state: OpState) -> OpState {
        if op_state.start_time.is_none() {
            self.current_instruction = None;
            return op_state;
        }

        let elapsed = op_state.start_time.unwrap().elapsed().unwrap();
        if op_state.curve_steps_done as f64 * curve.step_delay > elapsed.as_secs_f64() {
            return op_state;
        }
        op_state.curve_steps_done += 1;

        let abs_center: Location<i64> = op_state.start_pos.clone() + curve.p_center.clone();
        let rel_to_center = self.get_pos() - abs_center.clone();

        let step_dir: CircleStep = match curve.turn_direction {
            CircleDirection::CW => {
                let next_step: CircleStepCW = rel_to_center.into();
                next_step.into()
            }
            CircleDirection::CCW => {
                let next_step: CircleStepCCW = rel_to_center.into();
                next_step.into()
            }
        };
        let res = match step_dir.main {
            CircleStepDir::Right => self.motor_x.step(Direction::Right),
            CircleStepDir::Down => self.motor_y.step(Direction::Left),
            CircleStepDir::Left => self.motor_x.step(Direction::Left),
            CircleStepDir::Up => self.motor_y.step(Direction::Right),
        };
        if let Err(e) = res {
            println!("run out of space {}", e);
        }
        let pos_before_move = self.get_pos();
        let delta_before_op: Location<f64> = (pos_before_move.clone() - abs_center.clone()).into();
        let delta_before_op_step_correct = delta_before_op * curve.step_sizes.clone();
        let delta_radius_before_op = curve.radius_sq - delta_before_op_step_correct.distance_sq();
        let pos_after_move = pos_before_move
            + match step_dir.opt {
                CircleStepDir::Right => Location::<i64>::new(1, 0, 0),
                CircleStepDir::Down => Location::<i64>::new(0, -1, 0),
                CircleStepDir::Left => Location::<i64>::new(-1, 0, 0),
                CircleStepDir::Up => Location::<i64>::new(0, 1, 0),
            };

        let delta_after_op: Location<f64> = (pos_after_move - abs_center).into();
        let delta_after_op_step_correct = delta_after_op * curve.step_sizes.clone();
        let delta_radius_after_op = curve.radius_sq - delta_after_op_step_correct.distance_sq();
        if delta_radius_before_op.abs() > delta_radius_after_op.abs() {
            let res = match step_dir.opt {
                CircleStepDir::Right => self.motor_x.step(Direction::Right),
                CircleStepDir::Down => self.motor_y.step(Direction::Left),
                CircleStepDir::Left => self.motor_x.step(Direction::Left),
                CircleStepDir::Up => self.motor_y.step(Direction::Right),
            };
            if let Err(e) = res {
                println!("run out of space {}", e);
            }
        }

        let dist_destination: Location<i64> = curve.p_end.clone() - self.get_pos();
        let dist_to_dest = dist_destination.distance_sq() as u32;
        if dist_to_dest < 25 * 25 && !op_state.curve_close_to_destination {
            op_state.curve_close_to_destination = true;
        }

        if op_state.curve_close_to_destination
            && dist_to_dest > op_state.last_distance_to_destination
        {
            self.current_instruction =
                Some(Instruction::Line(InstructionLine::create_without_ramps(
                    self.get_pos(),
                    curve.p_end.clone(),
                    curve.v_max,
                )));
            op_state.curve_close_to_destination = false;
            op_state.last_distance_to_destination = 100;
        } else if op_state.curve_close_to_destination {
            op_state.last_distance_to_destination = dist_to_dest;
        }

        if dist_destination.distance_sq() == 0 {
            op_state.curve_close_to_destination = false;
            op_state.last_distance_to_destination = 100;
            println!("at destination, set currentTask to NONE");
            self.current_instruction = None;
        }
        op_state
    }

    fn exec_calibrate_instruction(
        &mut self,
        instruction: &InstructionCalibrate,
        mut op_state: OpState,
    ) -> OpState {
        // should not happen
        if op_state.start_time.is_none() {
            self.current_instruction = None;
            return op_state;
        }
        self.set_state(MachineState::Calibrate);

        let elapsed = op_state.start_time.unwrap().elapsed().unwrap().as_micros() as u64;
        let pos = self.get_pos();
        let calibrate_pin_closed =
            self.z_calibrate.is_none() || self.z_calibrate.as_mut().unwrap().is_closed();

        op_state.calibrate_x = Self::exec_calibrate_axe(
            instruction.x.clone(),
            elapsed,
            &mut self.motor_x,
            op_state.calibrate_x,
            calibrate_pin_closed,
            pos.x,
        );

        op_state.calibrate_y = Self::exec_calibrate_axe(
            instruction.y.clone(),
            elapsed,
            &mut self.motor_y,
            op_state.calibrate_y,
            calibrate_pin_closed,
            pos.y,
        );

        op_state.calibrate_z = Self::exec_calibrate_axe(
            instruction.z.clone(),
            elapsed,
            &mut self.motor_z,
            op_state.calibrate_z,
            calibrate_pin_closed,
            pos.z,
        );

        if op_state.calibrate_x.complete
            && op_state.calibrate_y.complete
            && op_state.calibrate_z.complete
        {
            op_state.reset_calibrate();
            self.current_instruction = None;
        } else {
            thread::sleep(Duration::from_micros(1_000));
        }

        op_state
    }

    fn exec_calibrate_axe(
        calibration_type: CalibrateType,
        elapsed: u64,
        motor: &mut Motor,
        mut calibrate: CalibrateData,
        calibrate_pin_closed: bool,
        pos: i64,
    ) -> CalibrateData {
        if calibrate.complete {
            return calibrate;
        }

        match (calibration_type, calibrate.phase) {
            (CalibrateType::Min, _) if elapsed > calibrate.steps_done * 1_000 => {
                if motor.step(Direction::Left).is_err() {
                    calibrate.complete = true;
                }
            }
            (CalibrateType::Max, _) if elapsed > calibrate.steps_done * 1_000 => {
                if motor.step(Direction::Right).is_err() {
                    calibrate.complete = true;
                }
            }
            (CalibrateType::Middle, 0) if elapsed > calibrate.steps_done * 1_000 => {
                // move to min
                if motor.step(Direction::Left).is_err() {
                    calibrate.pos_1 = pos;
                    calibrate.phase = 1;
                }
            }
            (CalibrateType::Middle, 1) if elapsed > calibrate.steps_done * 1_000 => {
                // move to max
                if motor.step(Direction::Right).is_err() {
                    let delta = (pos - calibrate.pos_1) / 2;
                    calibrate.pos_1 += delta;
                    calibrate.phase = 2;
                }
            }
            (CalibrateType::Middle, 2) if elapsed > calibrate.steps_done * 1_000 => {
                // move back to middle
                if calibrate.pos_1 <= pos || motor.step(Direction::Left).is_err() {
                    calibrate.complete = true;
                }
            }
            (CalibrateType::ContactPin, _) if elapsed > calibrate.steps_done * 3_000 => {
                if calibrate_pin_closed || motor.step(Direction::Right).is_err() {
                    calibrate.complete = true;
                }
            }
            _ => calibrate.complete = true,
        }
        calibrate
    }

    fn exec_delay(&mut self, delay: f64, op_state: OpState) -> OpState {
        thread::sleep(Duration::from_secs_f64(delay));
        op_state
    }

    fn exec_wait_for(&mut self, instruction: InstructionWaitFor, mut op_state: OpState) -> OpState {
        if op_state.wait_for.is_some() && op_state.wait_for.unwrap() != instruction {
            match instruction {
                InstructionWaitFor::ToolChanged(tool, length) => {
                    self.send(HardwareFeedback::RequireToolChange(tool, length))
                }
            }
            self.set_state(MachineState::WaitForInput);
            op_state.wait_for = Some(instruction);
        }
        op_state
    }

    #[allow(dead_code)]
    fn executer_on(&mut self, op_state: OpState) -> OpState {
        println!("switch on now");
        if let Ok(secs) = self.executer.resume() {
            thread::sleep(Duration::from_secs_f64(secs));
        } else {
            println!("switch on failed");
        }
        op_state
    }

    fn executer_change_speed(&mut self, speed: f64, cw: bool, op_state: OpState) -> OpState {
        println!("setSpeed");
        if let Ok(secs) = self.executer.on(speed, cw) {
            thread::sleep(Duration::from_secs_f64(secs));
        } else {
            println!("set speed failed");
        }
        op_state
    }

    fn executer_off(&mut self, op_state: OpState) -> OpState {
        println!("switch off now");
        if let Ok(secs) = self.executer.off() {
            thread::sleep(Duration::from_secs_f64(secs));
        } else {
            println!("switch off failed");
        }
        op_state
    }

    fn is_executer_on(&self) -> bool {
        self.executer.is_on()
    }
}
