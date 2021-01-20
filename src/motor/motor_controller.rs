use super::{motor_controller_thread::MotorControllerThread, task::CalibrateType, Motor};

use super::{
    task::{ManualTask, Task},
    Result,
};
use crate::io::{Actor, Switch};
use crate::program::Next3dMovement;
use crate::types::{Location, MachineState};
use std::{
    fmt::Debug,
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, Ordering::Relaxed},
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct MotorController {
    on_off: Option<Actor>,
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
        on_off: Option<Actor>,
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
            let mut inner = MotorControllerThread::new(
                motor_x.get_pos_ref(),
                motor_y.get_pos_ref(),
                motor_z.get_pos_ref(),
                motor_x,
                motor_y,
                motor_z,
                z_calibrate,
                None,
                Location::default(),
                state_inner,
                steps_todo_inner,
                steps_done_inner,
                cancel_task_inner,
                task_query_inner,
                receive_manual_task,
            );

            inner.run();
        });

        MotorController {
            on_off,
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

    pub fn switch_on(&mut self) {
        self.on_off.as_mut().map(|actor| actor.set_high());
    }
    pub fn switch_off(&mut self) {
        self.on_off.as_mut().map(|actor| actor.set_low());
    }
}