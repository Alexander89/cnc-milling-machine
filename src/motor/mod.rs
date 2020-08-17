#![allow(dead_code)]
use crate::program::Next3dMovement;
use crate::switch::Switch;
use crate::types::{
    CircleDirection, CircleMovement, CircleStep, CircleStepCCW, CircleStepCW, CircleStepDir,
    Direction, LinearMovement, Location, MachineState, MoveType, SteppedCircleMovement,
    SteppedLinearMovement, SteppedMoveType,
};
use log::{debug, max_level, LevelFilter};
use rppal::gpio::{Gpio, OutputPin};
use std::{
    fmt::Debug,
    result,
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering::Relaxed},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

pub type Result<T> = result::Result<T, &'static str>;

const INNER_MANUAL_DISTANCE: f64 = 3_000_000_000_000.0; //(1_000_000.0f64 * 1_000_000.0f64 * 3.0f64).sqrt();
#[derive(Debug)]
pub struct InnerTaskProduction {
    start_time: SystemTime,
    destination: Location<i64>,
    duration: Duration,
    move_type: SteppedMoveType,
}
#[derive(Debug)]
pub enum CalibrateType {
    None,
    Min,
    Max,
    Middle,
    ContactPin,
}
#[derive(Debug)]
pub struct InnerTaskCalibrate {
    x: CalibrateType,
    y: CalibrateType,
    z: CalibrateType,
}

#[derive(Debug)]
pub enum InnerTask {
    Production(InnerTaskProduction),
    Calibrate(InnerTaskCalibrate),
}
impl InnerTask {
    /**
     * convert task to InnerTask
     *
     * - **t** Task to convert
     * - **current_pos** current position in steps
     * - **step_size** mm per step
     * - **max_speed** mm per sec
     */
    pub fn from_task(
        t: Task,
        current_pos: Location<i64>,
        step_sizes: Location<f64>,
        max_speed: f64,
    ) -> InnerTask {
        match t {
            Task::Manual(task) => {
                let x = task.move_x_speed * -1000000.0f64;
                let y = task.move_y_speed * -1000000.0f64;
                let z = task.move_z_speed * -1000000.0f64;

                let pos_f64: Location<f64> = current_pos.into();
                let delta = Location::new(x, y, z).into();
                let destination = pos_f64 + Location::new(x, y, z);
                let distance_vec = destination.clone() / step_sizes;
                let distance = distance_vec.distance();

                InnerTask::Production(InnerTaskProduction {
                    start_time: SystemTime::now(),
                    destination: destination.into(),
                    duration: Duration::new(120, 0),
                    move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                        delta: delta,
                        distance: distance,
                    }),
                })
            }
            Task::Program(Next3dMovement {
                speed,
                move_type,
                to,
                ..
            }) => match move_type {
                MoveType::Linear(LinearMovement { distance, delta }) => {
                    let delta_in_steps: Location<i64> = (delta / step_sizes).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        destination: delta_in_steps.clone() - current_pos,
                        duration: Duration::from_secs_f64(distance / speed.unwrap_or(max_speed)),
                        move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                            delta: delta_in_steps,
                            distance: distance,
                        }),
                    })
                }
                MoveType::Rapid(LinearMovement { distance, delta }) => {
                    let delta_in_steps = delta / step_sizes;

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        destination: (delta_in_steps.clone() - current_pos.into()).into(),
                        duration: Duration::from_secs_f64(distance / speed.unwrap_or(max_speed)),
                        move_type: SteppedMoveType::Rapid(SteppedLinearMovement {
                            delta: delta_in_steps.into(),
                            distance: distance,
                        }),
                    })
                }
                MoveType::Circle(CircleMovement {
                    center,
                    turn_direction,
                    radius_sq,
                }) => {
                    println!("{}", to);
                    let destination = to / step_sizes.clone();

                    let step_speed =
                        speed.unwrap_or(2.0).min(max_speed).max(0.05) / step_sizes.max();
                    let step_center = (center.clone() / step_sizes.clone()).into();
                    // println!(
                    //     "{}  * {} = {} into {}",
                    //     center,
                    //     step_sizes,
                    //     center.clone() * step_sizes.clone(),
                    //     step_center
                    // );

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        destination: destination.into(),
                        duration: Duration::from_secs_f64(0.0f64),
                        move_type: SteppedMoveType::Circle(SteppedCircleMovement {
                            center: step_center,
                            radius_sq: radius_sq,
                            step_sizes: step_sizes,
                            turn_direction: turn_direction,
                            speed: step_speed,
                        }),
                    })
                }
            },

            Task::Calibrate(x, y, z) => {
                InnerTask::Calibrate(InnerTaskCalibrate { x: x, y: y, z: z })
            }
        }
    }
}

#[derive(Debug)]
pub struct ManualTask {
    /** x speed from -1.0 to 1.0 */
    move_x_speed: f64,
    /** y speed from -1.0 to 1.0 */
    move_y_speed: f64,
    /** z speed from -1.0 to 1.0 */
    move_z_speed: f64,
    /** move speed [mm/sec] */
    speed: f64,
}

#[derive(Debug)]
pub enum Task {
    Program(Next3dMovement),
    Manual(ManualTask),
    Calibrate(CalibrateType, CalibrateType, CalibrateType),
}

impl Task {
    pub fn machine_state(&self) -> MachineState {
        match self {
            Task::Program(_) => MachineState::ProgramTask,
            Task::Manual(_) => MachineState::ManualTask,
            Task::Calibrate(_, _, _) => MachineState::Calibrate,
        }
    }
}

#[derive(Debug)]
struct MotorControllerThread {
    motor_x: Motor,
    motor_y: Motor,
    motor_z: Motor,

    z_calibrate: Option<Switch>,

    x_step: Arc<AtomicI64>,
    y_step: Arc<AtomicI64>,
    z_step: Arc<AtomicI64>,

    current_task: Option<(InnerTask, Location<i64>, SystemTime)>,
    current_location: Location<f64>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    task_query: Arc<Mutex<Vec<Task>>>,
}

impl MotorControllerThread {
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
    pub fn run(&mut self) -> () {
        let mut curve_last_step = SystemTime::now();
        let mut curve_close_to_destination = false;
        let mut last_distance_to_destination = 100;
        loop {
            if self.cancel_task.load(Relaxed) == true {
                self.current_task = None;
                self.cancel_task.store(false, Relaxed);
            }
            match &self.current_task {
                Some((
                    InnerTask::Production(InnerTaskProduction {
                        duration,
                        move_type,
                        destination,
                        ..
                    }),
                    start_pos,
                    start,
                )) => match move_type {
                    SteppedMoveType::Linear(SteppedLinearMovement { delta, .. })
                    | SteppedMoveType::Rapid(SteppedLinearMovement { delta, .. }) => {
                        if let Ok(elapsed) = start.elapsed() {
                            let runtime = elapsed.as_micros() as u64;
                            let job_runtime = duration.as_micros() as u64;

                            let move_in_task = (self.get_pos() - start_pos.clone()).abs();

                            let delta_u64: Location<i64> = delta.clone().into();
                            let (x, y, z) = delta_u64.split();
                            debug!(
                                "loop: start {} move_in_task {} delta {}",
                                start_pos, move_in_task, delta
                            );
                            if (delta_u64.abs() - move_in_task.clone() == Location::default())
                                || (x == 0 && y == 0 && z == 0)
                            {
                                self.current_task = None;
                            } else {
                                if x != 0
                                    && x.abs() as u64 != move_in_task.x
                                    && runtime > job_runtime / x.abs() as u64 * move_in_task.x
                                {
                                    let dir = if x > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    self.motor_x.step(dir).expect("Step failed X");
                                }

                                if y != 0
                                    && y.abs() as u64 != move_in_task.y
                                    && runtime > job_runtime / y.abs() as u64 * move_in_task.y
                                {
                                    let dir = if y > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    self.motor_y.step(dir).expect("Step failed Y");
                                }

                                if z != 0
                                    && z.abs() as u64 != move_in_task.z
                                    && runtime > job_runtime / z.abs() as u64 * move_in_task.z
                                {
                                    let dir = if z > 0 {
                                        Direction::Right
                                    } else {
                                        Direction::Left
                                    };
                                    self.motor_z.step(dir).expect("Step failed Z");
                                }
                            }
                        } else {
                            self.current_task = None;
                        }
                    }
                    SteppedMoveType::Circle(SteppedCircleMovement {
                        turn_direction,
                        center,
                        radius_sq,
                        step_sizes,
                        speed,
                        ..
                    }) => {
                        if curve_last_step.elapsed().unwrap().as_secs_f64() <= 1.0 / *speed {
                            continue;
                        }
                        curve_last_step = SystemTime::now();

                        let abs_center: Location<i64> = start_pos.clone() + center.clone().into();
                        let rel_to_center = self.get_pos() - abs_center.clone();
                        // let abs_center_f: Location<f64> = abs_center.clone().into();
                        // println!("{}", abs_center_f / step_sizes.clone());
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
                            CircleStepDir::Right => {
                                self.motor_x.step(Direction::Right).expect("Step failed X")
                            }
                            CircleStepDir::Down => {
                                self.motor_y.step(Direction::Left).expect("Step failed Y")
                            }
                            CircleStepDir::Left => {
                                self.motor_x.step(Direction::Left).expect("Step failed X")
                            }
                            CircleStepDir::Up => {
                                self.motor_y.step(Direction::Right).expect("Step failed Y")
                            }
                        }
                        let pos_before_move = self.get_pos();
                        let pos_after_move = pos_before_move.clone()
                            + match step_dir.opt {
                                CircleStepDir::Right => Location::<i64>::new(1, 0, 0),
                                CircleStepDir::Down => Location::<i64>::new(0, -1, 0),
                                CircleStepDir::Left => Location::<i64>::new(-1, 0, 0),
                                CircleStepDir::Up => Location::<i64>::new(0, 1, 0),
                            };

                        let delta_before_op: Location<f64> =
                            (pos_before_move - abs_center.clone()).into();
                        let delta_before_op_step_correct =
                            delta_before_op.clone() * step_sizes.clone();
                        let delta_radius_before_op =
                            radius_sq - delta_before_op_step_correct.distance_sq();

                        let delta_after_op: Location<f64> =
                            (pos_after_move - abs_center.clone()).into();
                        let delta_after_op_step_correct =
                            delta_after_op.clone() * step_sizes.clone();
                        let delta_radius_after_op =
                            radius_sq - delta_after_op_step_correct.distance_sq();

                        if delta_radius_before_op.abs() > delta_radius_after_op.abs() {
                            match step_dir.opt {
                                CircleStepDir::Right => {
                                    self.motor_x.step(Direction::Right).expect("Step failed X")
                                }
                                CircleStepDir::Down => {
                                    self.motor_y.step(Direction::Left).expect("Step failed Y")
                                }
                                CircleStepDir::Left => {
                                    self.motor_x.step(Direction::Left).expect("Step failed X")
                                }
                                CircleStepDir::Up => {
                                    self.motor_y.step(Direction::Right).expect("Step failed Y")
                                }
                            };
                        }
                        let dist_destination = destination.clone() - self.get_pos();
                        let dist_to_dest = dist_destination.clone().distance_sq();
                        //println!("{} < {}", dist_destination, step_sizes.distance_sq() as i64);
                        if dist_to_dest < 10 {
                            curve_close_to_destination = true;
                        }

                        if curve_close_to_destination && dist_to_dest > last_distance_to_destination
                        {
                            self.current_task = Some((
                                InnerTask::Production(InnerTaskProduction {
                                    destination: dist_destination.clone(),
                                    duration: Duration::from_secs_f64(0.1),
                                    start_time: SystemTime::now(),
                                    move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                                        delta: dist_destination.clone(),
                                        distance: dist_to_dest as f64,
                                    }),
                                }),
                                self.get_pos(),
                                SystemTime::now(),
                            ));
                            curve_close_to_destination = false;
                            last_distance_to_destination = 100;
                        } else {
                            last_distance_to_destination = dist_to_dest
                        }

                        if dist_destination.distance_sq() == 0 {
                            curve_close_to_destination = false;
                            last_distance_to_destination = 100;
                            self.current_task = None;
                        }
                    }
                },
                Some((InnerTask::Calibrate(InnerTaskCalibrate { z, .. }), start_pos, start)) => {
                    let runtime = start.elapsed().unwrap().as_micros() as u64;
                    let move_in_task = (self.get_pos() - start_pos.clone()).abs();
                    let z_steps = move_in_task.z;

                    if runtime > z_steps * 4_000 {
                        match z {
                            CalibrateType::Min => {
                                if let Err(_) = self.motor_z.step(Direction::Left) {
                                    self.current_task = None;
                                }
                            }
                            CalibrateType::Max => {
                                if let Err(_) = self.motor_z.step(Direction::Right) {
                                    self.current_task = None;
                                }
                            }
                            CalibrateType::Middle => {
                                println!("impl missing");
                                self.current_task = None;
                            }
                            CalibrateType::ContactPin => {
                                if let Err(_) = self.motor_z.step(Direction::Right) {
                                    self.current_task = None;
                                }
                                if self.z_calibrate.is_none()
                                    || self.z_calibrate.as_mut().unwrap().is_closed()
                                {
                                    self.current_task = None;
                                }
                            }
                            CalibrateType::None => (),
                        }
                    }
                }
                None => match self.task_query.lock() {
                    Ok(ref mut lock) if lock.len() > 0 => {
                        let next = lock.remove(0);
                        self.state.store(next.machine_state().into(), Relaxed);
                        self.current_task = Some((
                            InnerTask::from_task(next, self.get_pos(), self.get_step_sizes(), 10.0),
                            self.get_pos(),
                            SystemTime::now(),
                        ));
                        println!("start task {:?}", self.current_task.as_ref().unwrap().0);
                    }
                    Ok(lock) => {
                        if self.state.load(Relaxed) != MachineState::Idle.into() {
                            self.state.store(MachineState::Idle.into(), Relaxed);
                        }
                        drop(lock);
                        thread::sleep(Duration::new(0, 10_000));
                    }
                    _ => {
                        thread::sleep(Duration::new(0, 10_000));
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub struct MotorController {
    thread: thread::JoinHandle<()>,
    ref_location: Location<i64>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    task_query: Arc<Mutex<Vec<Task>>>,
    step_sizes: Location<f64>,
    x: Arc<AtomicI64>,
    y: Arc<AtomicI64>,
    z: Arc<AtomicI64>,
}

impl MotorController {
    pub fn new(
        motor_x: Motor,
        motor_y: Motor,
        motor_z: Motor,
        z_calibrate: Option<Switch>,
    ) -> Self {
        let cancel_task = Arc::new(AtomicBool::new(false));
        let state = Arc::new(AtomicU32::new(0));

        let task_query = Arc::new(Mutex::new(Vec::new()));
        let step_sizes = Location {
            x: motor_x.get_step_size(),
            y: motor_y.get_step_size(),
            z: motor_z.get_step_size(),
        };

        let x = motor_x.get_pos_ref();
        let y = motor_y.get_pos_ref();
        let z = motor_z.get_pos_ref();
        let state_inner = state.clone();
        let cancel_task_inner = cancel_task.clone();
        let task_query_inner = task_query.clone();
        let thread = std::thread::spawn(move || {
            let mut inner = MotorControllerThread {
                x_step: motor_x.get_pos_ref(),
                y_step: motor_y.get_pos_ref(),
                z_step: motor_z.get_pos_ref(),
                motor_x: motor_x,
                motor_y: motor_y,
                motor_z: motor_z,
                z_calibrate: z_calibrate,
                current_task: None,
                current_location: Location::default(),
                state: state_inner,
                cancel_task: cancel_task_inner,
                task_query: task_query_inner,
            };

            inner.run();
        });

        MotorController {
            thread: thread,
            state: state,
            ref_location: Location { x: 0, y: 0, z: 0 },
            cancel_task: cancel_task,
            step_sizes: step_sizes,
            x: x,
            y: y,
            z: z,
            task_query: task_query,
        }
    }
    pub fn query_task(&mut self, task: Next3dMovement) -> () {
        self.task_query.lock().unwrap().push(Task::Program(task));
    }
    pub fn calibrate(&mut self, x: CalibrateType, y: CalibrateType, z: CalibrateType) -> () {
        self.task_query
            .lock()
            .unwrap()
            .push(Task::Calibrate(x, y, z));
    }
    pub fn get_state(&self) -> MachineState {
        self.state.load(Relaxed).into()
    }
    pub fn manual_move(&mut self, x: f64, y: f64, z: f64, speed: f64) -> () {
        self.task_query.lock().unwrap().clear();
        self.cancel_task().expect("ok");
        self.task_query
            .lock()
            .unwrap()
            .push(Task::Manual(ManualTask {
                move_x_speed: x,
                move_y_speed: y,
                move_z_speed: z,
                speed: speed,
            }));
    }
    pub fn cancel_task(&mut self) -> Result<()> {
        self.cancel_task.store(true, Relaxed);
        Ok(())
    }
    pub fn reset(&mut self) -> () {
        self.ref_location = self.motor_pos();
    }
    pub fn set_pos(&mut self, pos: Location<f64>) -> () {
        self.ref_location = self.motor_pos() + (pos / self.step_sizes.clone()).into();
    }
    pub fn motor_pos(&self) -> Location<i64> {
        Location {
            x: self.x.load(Relaxed),
            y: self.y.load(Relaxed),
            z: self.z.load(Relaxed),
        }
    }
    pub fn get_pos(&self) -> Location<f64> {
        let relative: Location<f64> = (self.motor_pos() - self.ref_location.clone()).into();
        relative * self.step_sizes.clone()
    }
}

pub trait Driver: std::fmt::Debug {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction>;
    fn get_step_size(&self) -> f64;
}

#[derive(Debug)]
pub struct MotorInner {
    name: String,
    max_step_speed: u32, // steps per second
    driver: Box<dyn Driver + Send>,
}

#[derive(Debug)]
pub struct Motor {
    pos: Arc<AtomicI64>,
    inner: Arc<Mutex<MotorInner>>,
    step_size: f64, // mm per step
}

impl Motor {
    pub fn new(name: String, max_step_speed: u32, driver: Box<dyn Driver + Send>) -> Self {
        Motor {
            pos: Arc::new(AtomicI64::new(0)),
            step_size: driver.get_step_size(),
            inner: Arc::new(Mutex::new(MotorInner {
                name: name,
                max_step_speed: max_step_speed, // steps per second
                driver: driver,
            })),
        }
    }
    pub fn step(&mut self, direction: Direction) -> Result<()> {
        match (*self.inner.lock().unwrap().driver).do_step(&direction) {
            Ok(Direction::Left) => {
                if max_level() == LevelFilter::Debug {
                    print!("-");
                }
                (*self.pos).fetch_sub(1, Relaxed);
                Ok(())
            }
            Ok(Direction::Right) => {
                if max_level() == LevelFilter::Debug {
                    print!("+");
                }
                (*self.pos).fetch_add(1, Relaxed);
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }
    pub fn get_pos_ref(&self) -> Arc<AtomicI64> {
        self.pos.clone()
    }
    pub fn get_step_size(&self) -> f64 {
        self.step_size
    }
}

#[derive(Debug)]
pub struct MockMotor {
    current_direction: Direction,
    step_size: f64,
}

impl MockMotor {
    pub fn new(step_size: f64) -> Self {
        MockMotor {
            current_direction: Direction::Left,
            step_size: step_size,
        }
    }
}

impl Driver for MockMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            self.current_direction = direction.clone();
        };

        Ok(direction.clone())
    }
    fn get_step_size(&self) -> f64 {
        self.step_size
    }
}

#[derive(Debug)]
pub struct StepMotor {
    name: String,
    pull: OutputPin,
    direction: OutputPin,
    enable: Option<OutputPin>,
    max_step_speed: u32, // steps per second
    step_pos: i32,       // + - from the reset point
    step_size: f64,      // mm per step
    end_switch_left: Option<Switch>,
    end_switch_right: Option<Switch>,
    current_direction: Direction,
}

impl StepMotor {
    pub fn new(
        pull: u8,
        dir: u8,
        ena: Option<u8>,
        end_left: Option<u8>,
        end_right: Option<u8>,
        max_step_speed: u32,
        step_size: f64,
    ) -> Self {
        let mut name = format!("Stepper p{} d{} ", pull, dir);
        let ena_gpio = if let Some(ena_pin) = ena {
            name = format!("{} e{} ", name, ena_pin);
            Some(Gpio::new().unwrap().get(ena_pin).unwrap().into_output())
        } else {
            None
        };

        let left = if let Some(pin) = end_left {
            Some(Switch::new(pin, true))
        } else {
            None
        };

        let right = if let Some(pin) = end_right {
            Some(Switch::new(pin, true))
        } else {
            None
        };

        let mut direction = Gpio::new().unwrap().get(dir).unwrap().into_output();
        direction.set_low();

        StepMotor {
            name: name,
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction: direction,
            enable: ena_gpio,
            step_pos: 0i32,
            max_step_speed: max_step_speed,
            end_switch_left: left,
            end_switch_right: right,
            current_direction: Direction::Left,
            step_size: step_size,
        }
    }
    fn is_blocked(&mut self) -> bool {
        let switch_opt = match self.current_direction {
            Direction::Left => self.end_switch_left.as_mut(),
            Direction::Right => self.end_switch_right.as_mut(),
        };
        if let Some(switch) = switch_opt {
            switch.is_closed()
        } else {
            false
        }
    }
}

impl Driver for StepMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            match direction {
                Direction::Left => {
                    if self.direction.is_set_high() {
                        self.direction.set_low();
                        thread::sleep(Duration::new(0, 3));
                    }
                }
                Direction::Right => {
                    if self.direction.is_set_low() {
                        self.direction.set_high();
                        thread::sleep(Duration::new(0, 3));
                    }
                }
            }
            self.current_direction = direction.clone();
        }
        if self.is_blocked() {
            Err("is blocked at end")
        } else {
            self.pull.toggle();
            Ok(self.current_direction.clone())
        }
    }
    fn get_step_size(&self) -> f64 {
        self.step_size
    }
}
