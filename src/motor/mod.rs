#![allow(dead_code)]
use crate::program::Next3dMovement;
use crate::switch::Switch;
use crate::types::{
    CircleDirection, CircleMovement, CircleStep, CircleStepCCW, CircleStepCW, CircleStepDir,
    Direction, LinearMovement, Location, MachineState, MoveType, SteppedCircleMovement,
    SteppedLinearMovement, SteppedMoveType,
};
use log::{max_level, LevelFilter};
use rppal::gpio::{Gpio, OutputPin};
use std::{
    fmt::Debug,
    result,
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicI32, Ordering::Relaxed},
        mpsc::{channel, Receiver, Sender},
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
    from: Location<i64>,
    destination: Location<i64>,
    move_type: SteppedMoveType,
}
#[derive(Debug, Clone)]
pub enum CalibrateType {
    None,
    Min,
    Max,
    Middle,
    ContactPin,
}
#[derive(Debug)]
pub struct InnerTaskCalibrate {
    start_time: SystemTime,
    from: Location<i64>,
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
                let input = Location::new(task.move_x_speed, task.move_y_speed, task.move_z_speed);
                let speed = input.clone().distance() * task.speed;

                let move_vec: Location<f64> = input.clone() * 10000.0f64;
                let delta: Location<i64> = (move_vec.clone() / step_sizes).into(); // [steps] (10m more than the table into i64 steps)

                let destination = current_pos.clone() + delta.clone();
                let distance = move_vec.distance();

                InnerTask::Production(InnerTaskProduction {
                    start_time: SystemTime::now(),
                    from: current_pos,
                    destination,
                    move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                        delta,
                        distance,
                        speed,
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
                        from: current_pos.clone(),
                        destination: delta_in_steps.clone() - current_pos,
                        move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                            delta: delta_in_steps,
                            distance,
                            speed: speed.min(max_speed),
                        }),
                    })
                }
                MoveType::Rapid(LinearMovement { distance, delta }) => {
                    let delta_in_steps: Location<i64> = (delta / step_sizes).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        from: current_pos.clone(),
                        destination: delta_in_steps.clone() - current_pos,
                        move_type: SteppedMoveType::Rapid(SteppedLinearMovement {
                            delta: delta_in_steps,
                            distance,
                            speed: speed.min(max_speed),
                        }),
                    })
                }
                MoveType::Circle(CircleMovement {
                    center,
                    turn_direction,
                    radius_sq,
                }) => {
                    let destination = to / step_sizes.clone();

                    let step_delay = step_sizes.max() / speed.min(max_speed).max(0.05);
                    let step_center = (center / step_sizes.clone()).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        from: current_pos,
                        destination: destination.into(),
                        move_type: SteppedMoveType::Circle(SteppedCircleMovement {
                            center: step_center,
                            radius_sq,
                            step_sizes,
                            turn_direction,
                            speed,
                            step_delay,
                        }),
                    })
                }
            },

            Task::Calibrate(x, y, z) => InnerTask::Calibrate(InnerTaskCalibrate {
                start_time: SystemTime::now(),
                from: current_pos,
                x,
                y,
                z,
            }),
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

    current_task: Option<InnerTask>,
    current_location: Location<f64>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    steps_todo: Arc<AtomicI32>,
    steps_done: Arc<AtomicI32>,
    task_query: Arc<Mutex<Vec<Task>>>,
    manual_task_receiver: Receiver<ManualTask>,
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
    pub fn run(&mut self) {
        let mut last_step: Option<SystemTime> = None; //SystemTime::now();
        let mut curve_close_to_destination = false;
        let mut last_distance_to_destination = 100;
        let mut q_ptr = 0;
        loop {
            if let Ok(next_task) = self.manual_task_receiver.try_recv() {
                let max_speed = next_task.speed;
                let task = Task::Manual(next_task);
                self.state.store(task.machine_state().into(), Relaxed);
                self.current_task = Some(InnerTask::from_task(
                    task,
                    self.get_pos(),
                    self.get_step_sizes(),
                    max_speed,
                ));
            };
            if self.cancel_task.load(Relaxed) {
                self.cancel_task.store(false, Relaxed);
                self.current_task = None;
                self.steps_todo.store(0, Relaxed);
                self.steps_done.store(0, Relaxed);
                println!("MotorControllerThread: cancel task");
            };
            match &self.current_task {
                Some(InnerTask::Production(InnerTaskProduction {
                    start_time,
                    move_type,
                    from,
                    destination,
                    ..
                })) => match move_type {
                    SteppedMoveType::Linear(SteppedLinearMovement {
                        delta,
                        speed,
                        distance,
                        ..
                    })
                    | SteppedMoveType::Rapid(SteppedLinearMovement {
                        delta,
                        speed,
                        distance,
                        ..
                    }) => {
                        if let Ok(elapsed) = start_time.elapsed() {
                            // println!(
                            //     "loop: speed {} distance {} delta {}",
                            //     speed, distance, delta
                            // );
                            if *speed == 0.0f64 || *distance == 0.0f64 {
                                self.current_task = None;
                                continue;
                            }
                            let duration = Duration::from_secs_f64(*distance / *speed);
                            let runtime = elapsed.as_micros() as u64;
                            let job_runtime = duration.as_micros() as u64;

                            let move_in_task = (self.get_pos() - from.clone()).abs();

                            let (x, y, z) = delta.split();

                            if (delta.abs() - move_in_task.clone() == Location::default())
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
                            println!("failed at start_time.elapsed()");
                        }
                    }
                    SteppedMoveType::Circle(SteppedCircleMovement {
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
                                move_type: SteppedMoveType::Linear(SteppedLinearMovement {
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
                    ..
                })) => {
                    let runtime = start_time.elapsed().unwrap().as_micros() as u64;
                    let move_in_task = (self.get_pos() - from.clone()).abs();
                    let z_steps = move_in_task.z;

                    if runtime > z_steps * 4_000 {
                        match z {
                            CalibrateType::Min => {
                                if self.motor_z.step(Direction::Left).is_err() {
                                    self.current_task = None;
                                }
                            }
                            CalibrateType::Max => {
                                if self.motor_z.step(Direction::Right).is_err() {
                                    self.current_task = None;
                                }
                            }
                            CalibrateType::Middle => {
                                println!("impl missing");
                                self.current_task = None;
                            }
                            CalibrateType::ContactPin => {
                                if self.motor_z.step(Direction::Right).is_err() {
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
                    Ok(ref mut lock) if lock.len() > q_ptr => {
                        let next = lock[q_ptr].clone();
                        q_ptr += 1;
                        self.state.store(next.machine_state().into(), Relaxed);
                        self.steps_done.store(q_ptr as i32, Relaxed);
                        self.steps_todo.store((lock.len() - q_ptr) as i32, Relaxed);
                        println!("next {:?} {:?}", q_ptr, lock.len() - q_ptr);
                        self.current_task = Some(InnerTask::from_task(
                            next,
                            self.get_pos(),
                            self.get_step_sizes(),
                            40.0f64,
                        ));
                    }
                    Ok(ref mut lock) => {
                        lock.clear();
                        q_ptr = 0;
                        let idle: u32 = MachineState::Idle.into();

                        if self.state.load(Relaxed) != idle {
                            self.state.store(idle, Relaxed);
                        }
                        #[allow(clippy::drop_ref)]
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
    fn max_speed(&self) -> f64 {
        let x_speed: f64 = self.motor_x.speed as f64 * self.motor_x.step_size;
        let y_speed: f64 = self.motor_y.speed as f64 * self.motor_y.step_size;
        let z_speed: f64 = self.motor_z.speed as f64 * self.motor_z.step_size;
        x_speed.min(y_speed).min(z_speed)
    }
}

#[derive(Debug)]
pub struct MotorController {
    thread: thread::JoinHandle<()>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    task_query: Arc<Mutex<Vec<Task>>>,
    manual_task_sender: Sender<ManualTask>,
    step_sizes: Location<f64>,
    steps_todo: Arc<AtomicI32>,
    steps_done: Arc<AtomicI32>,
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
        let steps_todo = Arc::new(AtomicI32::new(0));
        let steps_done = Arc::new(AtomicI32::new(0));

        let task_query = Arc::new(Mutex::new(Vec::new()));
        let step_sizes = Location {
            x: motor_x.get_step_size(),
            y: motor_y.get_step_size(),
            z: motor_z.get_step_size(),
        };

        let (manual_task_sender, receive_manual_task) = channel::<ManualTask>();
        let x = motor_x.get_pos_ref();
        let y = motor_y.get_pos_ref();
        let z = motor_z.get_pos_ref();
        let state_inner = state.clone();
        let steps_todo_inner = steps_todo.clone();
        let steps_done_inner = steps_done.clone();
        let cancel_task_inner = cancel_task.clone();
        let task_query_inner = task_query.clone();
        let thread = std::thread::spawn(move || {
            let mut inner = MotorControllerThread {
                x_step: motor_x.get_pos_ref(),
                y_step: motor_y.get_pos_ref(),
                z_step: motor_z.get_pos_ref(),
                motor_x,
                motor_y,
                motor_z,
                z_calibrate,
                current_task: None,
                current_location: Location::default(),
                state: state_inner,
                steps_todo: steps_todo_inner,
                steps_done: steps_done_inner,
                cancel_task: cancel_task_inner,
                task_query: task_query_inner,
                manual_task_receiver: receive_manual_task,
            };

            inner.run();
        });

        MotorController {
            thread,
            state,
            steps_todo,
            steps_done,
            cancel_task,
            step_sizes,
            x,
            y,
            z,
            task_query,
            manual_task_sender,
        }
    }
    pub fn query_task(&mut self, task: Next3dMovement) {
        self.task_query.lock().unwrap().push(Task::Program(task));
    }
    pub fn calibrate(&mut self, x: CalibrateType, y: CalibrateType, z: CalibrateType) {
        self.task_query
            .lock()
            .unwrap()
            .push(Task::Calibrate(x, y, z));
    }

    pub fn get_state(&self) -> MachineState {
        self.state.load(Relaxed).into()
    }
    pub fn get_steps_todo(&self) -> i64 {
        self.steps_todo.load(Relaxed).into()
    }
    pub fn get_steps_done(&self) -> i64 {
        self.steps_done.load(Relaxed).into()
    }

    pub fn manual_move(&mut self, x: f64, y: f64, z: f64, speed: f64) {
        if self
            .manual_task_sender
            .send(ManualTask {
                move_x_speed: x,
                move_y_speed: y,
                move_z_speed: z,
                speed,
            })
            .is_err()
        {
            println!("cant set manual move");
        }
    }
    pub fn cancel_task(&mut self) -> Result<()> {
        self.task_query.lock().unwrap().clear();
        self.cancel_task.store(true, Relaxed);
        while self.cancel_task.load(Relaxed) {
            thread::sleep(Duration::from_nanos(100));
        }
        Ok(())
    }
    pub fn reset(&mut self) {
        self.set_pos(Location::default())
    }
    pub fn set_pos(&mut self, pos: Location<f64>) {
        let steps: Location<i64> = (pos / self.step_sizes.clone()).into();
        self.x.store(steps.x, Relaxed);
        self.y.store(steps.y, Relaxed);
        self.z.store(steps.z, Relaxed);
    }
    pub fn motor_pos(&self) -> Location<i64> {
        Location {
            x: self.x.load(Relaxed),
            y: self.y.load(Relaxed),
            z: self.z.load(Relaxed),
        }
    }
    pub fn get_pos(&self) -> Location<f64> {
        let pos: Location<f64> = self.motor_pos().into();
        pos * self.step_sizes.clone()
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
    last_step: SystemTime,
    pref_delta_t: f64,
    speed: u64,
}

impl Motor {
    pub fn new(name: String, max_step_speed: u32, driver: Box<dyn Driver + Send>) -> Self {
        Motor {
            pos: Arc::new(AtomicI64::new(0)),
            step_size: driver.get_step_size(),
            inner: Arc::new(Mutex::new(MotorInner {
                name,
                max_step_speed, // steps per second
                driver,
            })),
            last_step: SystemTime::now(),
            pref_delta_t: 1.0,
            speed: 1000,
        }
    }
    pub fn step(&mut self, direction: Direction) -> Result<()> {
        // block motor to have a smooth ramp
        // this wil slow slow down the complete program, because all motors run in one thread
        // the slowest motor do also slow down the rest

        // first step after a brake could be done immediately
        if self.last_step.elapsed().unwrap() > Duration::from_millis(50) {
            self.speed = 1000;
        } else {
            self.speed += (6000 - self.speed) / (self.speed / 50);
            // println!(
            //     "{} {}",
            //     self.speed,
            //     self.last_step.elapsed().unwrap().as_nanos()
            // );
            let req_delay = Duration::from_micros(5000 - self.speed.min(5000));
            let delta_t = self.last_step.elapsed().unwrap();
            if delta_t < req_delay {
                let open_delay = req_delay.as_nanos() - delta_t.as_nanos();
                thread::sleep(Duration::new(0, open_delay as u32));
            };
        }
        self.pref_delta_t = self.last_step.elapsed().unwrap().as_secs_f64();
        self.last_step = SystemTime::now();

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
            step_size,
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
            name,
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction,
            enable: ena_gpio,
            step_pos: 0i32,
            max_step_speed,
            end_switch_left: left,
            end_switch_right: right,
            current_direction: Direction::Left,
            step_size,
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
                        //thread::sleep(Duration::new(0, 3));
                    }
                }
                Direction::Right => {
                    if self.direction.is_set_low() {
                        self.direction.set_high();
                        //thread::sleep(Duration::new(0, 3));
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
