#![allow(dead_code)]
use crate::program::Next3dMovement;
use crate::switch::Switch;
use crate::types::{Direction, Location, MachineState, MoveType};
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
pub struct InnerTask {
    start_time: SystemTime,
    destination: Location<i64>,
    delta: Location<i64>,
    duration: Duration,
    distance: f64,
    move_type: MoveType,
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
        step_size: f64,
        max_speed: f64,
    ) -> InnerTask {
        match t {
            Task::Manual(task) => {
                let x = task.move_x_speed * -1_000.0f64;
                let y = task.move_y_speed * -1_000.0f64;
                let z = task.move_z_speed * -1_000.0f64;

                let pos_f64: Location<f64> = current_pos.into();
                let delta = Location { x: x, y: y, z: z };
                let destination = pos_f64 + Location { x: x, y: y, z: z };
                let distance_vec = destination.clone() / step_size;
                let distance = distance_vec.distance();

                InnerTask {
                    start_time: SystemTime::now(),
                    destination: destination.into(),
                    delta: delta.into(),
                    duration: Duration::new(120, 0),
                    distance: distance,
                    move_type: MoveType::Linear,
                }
            }
            Task::Program(Next3dMovement {
                delta,
                speed,
                move_type,
                distance,
                ..
            }) => {
                let pos_f64: Location<f64> = current_pos.into();
                let delta_in_steps = delta / step_size;
                let destination = delta_in_steps.clone() - pos_f64;

                let duration = Duration::from_secs_f64(distance / speed.unwrap_or(max_speed));

                InnerTask {
                    start_time: SystemTime::now(),
                    destination: destination.into(),
                    delta: delta_in_steps.into(),
                    duration: duration,
                    distance: distance,
                    move_type: move_type,
                }
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
}

impl Task {
    pub fn machine_state(&self) -> MachineState {
        match self {
            Task::Program(_) => MachineState::ProgrammTask,
            Task::Manual(_) => MachineState::ManualTask,
        }
    }
}

#[derive(Debug)]
struct MotorControllerThread {
    motor_x: Motor,
    motor_y: Motor,
    motor_z: Motor,

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
    pub fn run(&mut self) -> () {
        loop {
            if self.cancel_task.load(Relaxed) == true {
                self.current_task = None;
                self.cancel_task.store(false, Relaxed);
            }
            match &self.current_task {
                Some((
                    InnerTask {
                        duration, delta, ..
                    },
                    start_pos,
                    start,
                )) => {
                    if let Ok(elapsed) = start.elapsed() {
                        let runtime = elapsed.as_micros() as u64;
                        let job_runtime = duration.as_micros() as u64;

                        let move_in_task = (self.get_pos() - start_pos.clone()).abs();

                        let (x, y, z) = delta.split();
                        debug!(
                            "loop: start {} move_in_task {} delta {}",
                            start_pos, move_in_task, delta
                        );
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
                    }
                }
                None => match self.task_query.lock() {
                    Ok(ref mut lock) if lock.len() > 0 => {
                        let next = lock.remove(0);
                        self.state.store(next.machine_state().into(), Relaxed);
                        self.current_task = Some((
                            InnerTask::from_task(next, self.get_pos(), 0.01, 5.0),
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
    step_size: f64,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    task_query: Arc<Mutex<Vec<Task>>>,
    x: Arc<AtomicI64>,
    y: Arc<AtomicI64>,
    z: Arc<AtomicI64>,
}

impl MotorController {
    pub fn new(motor_x: Motor, motor_y: Motor, motor_z: Motor) -> Self {
        let cancel_task = Arc::new(AtomicBool::new(false));
        let state = Arc::new(AtomicU32::new(0));

        let task_query = Arc::new(Mutex::new(Vec::new()));
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
            step_size: 0.01, // mm
            cancel_task: cancel_task,
            x: x,
            y: y,
            z: z,
            task_query: task_query,
        }
    }
    pub fn query_task(&mut self, task: Next3dMovement) -> () {
        self.task_query.lock().unwrap().push(Task::Program(task));
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
    pub fn motor_pos(&self) -> Location<i64> {
        Location {
            x: self.x.load(Relaxed),
            y: self.y.load(Relaxed),
            z: self.z.load(Relaxed),
        }
    }
    pub fn get_pos(&self) -> Location<f64> {
        let relative: Location<f64> = (self.motor_pos() - self.ref_location.clone()).into();
        relative * self.step_size
    }
}

pub trait Driver: std::fmt::Debug {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction>;
}

#[derive(Debug)]
pub struct MotorInner {
    name: String,
    max_step_speed: u32, // steps per second
    driver: Box<dyn Driver + Send>,
    step_size: f64, // mm per step
}

#[derive(Debug)]
pub struct Motor {
    pos: Arc<AtomicI64>,
    inner: Arc<Mutex<MotorInner>>,
}

impl Motor {
    pub fn new(
        name: String,
        step_size: f64,
        max_step_speed: u32,
        driver: Box<dyn Driver + Send>,
    ) -> Self {
        Motor {
            pos: Arc::new(AtomicI64::new(0)),
            inner: Arc::new(Mutex::new(MotorInner {
                name: name,
                max_step_speed: max_step_speed, // steps per second
                driver: driver,
                step_size: step_size, // mm per step
            })),
        }
    }
    pub fn step(&mut self, direction: Direction) -> Result<()> {
        match (*self.inner.lock().unwrap().driver).do_step(&direction) {
            Ok(Direction::Left) => {
                if max_level() == LevelFilter::Debug {
                    print!("+");
                }
                (*self.pos).fetch_sub(1, Relaxed);
                Ok(())
            }
            Ok(Direction::Right) => {
                if max_level() == LevelFilter::Debug {
                    print!("-");
                }
                (*self.pos).fetch_add(1, Relaxed);
                Ok(())
            }
            e @ Err(_) => e.map(|_| ()),
        }
    }
    pub fn get_pos_ref(&self) -> Arc<AtomicI64> {
        self.pos.clone()
    }
}

#[derive(Debug)]
pub struct MockMotor {
    current_direction: Direction,
}

impl MockMotor {
    pub fn new() -> Self {
        MockMotor {
            current_direction: Direction::Left,
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
}
