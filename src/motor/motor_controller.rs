use super::{motor_controller_thread::MotorControllerThread, task::CalibrateType, Motor};

use super::{
    task::{ManualInstruction, ManualTask, Task},
    Result,
};
use crate::gnc::{Next3dMovement, NextMiscellaneous};
use crate::io::{Actor, Switch};
use crate::types::{Location, MachineState};
use std::{
    fmt::Debug,
    sync::Mutex,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering::Relaxed},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};
#[derive(Debug, PartialEq)]
pub enum ExternalInput {
    ToolChanged,
    NewStock,
    StockTurned,
    SpeedChanged,
}

#[derive(Debug, PartialEq)]
pub enum ExternalInputRequest {
    ChangeTool(i32),
    ChangeSpeed(f64),
}

#[derive(Debug)]
pub struct MotorController {
    thread: thread::JoinHandle<()>,
    cancel_task: Arc<AtomicBool>,
    state: Arc<AtomicU32>,
    task_query: Arc<Mutex<Vec<Task>>>,
    manual_instruction_sender: Sender<ManualInstruction>,
    step_sizes: Location<f64>,
    steps_todo: Arc<AtomicI64>,
    steps_done: Arc<AtomicI64>,
    x: Arc<AtomicI64>,
    y: Arc<AtomicI64>,
    z: Arc<AtomicI64>,
    on_off_state: Arc<AtomicBool>,
}

#[allow(clippy::too_many_arguments)]
impl MotorController {
    pub fn new(
        on_off: Option<Actor>,
        switch_on_off_delay: f64,
        motor_x: Motor,
        motor_y: Motor,
        motor_z: Motor,
        z_calibrate: Option<Switch>,

        external_input_enabled: bool,
        external_input_receiver: Receiver<ExternalInput>,
        external_input_request_sender: Sender<ExternalInputRequest>,
    ) -> Self {
        let cancel_task = Arc::new(AtomicBool::new(false));
        let state = Arc::new(AtomicU32::new(0));
        let steps_todo = Arc::new(AtomicI64::new(0));
        let steps_done = Arc::new(AtomicI64::new(0));
        let on_off_state = Arc::new(AtomicBool::new(false));

        let task_query = Arc::new(Mutex::new(Vec::new()));
        let step_sizes = Location {
            x: motor_x.get_step_size(),
            y: motor_y.get_step_size(),
            z: motor_z.get_step_size(),
        };

        let (manual_instruction_sender, receive_manual_instruction) =
            channel::<ManualInstruction>();
        let x = motor_x.get_pos_ref();
        let y = motor_y.get_pos_ref();
        let z = motor_z.get_pos_ref();
        let state_inner = state.clone();
        let steps_todo_inner = steps_todo.clone();
        let steps_done_inner = steps_done.clone();
        let on_off_state_inner = on_off_state.clone();
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
                receive_manual_instruction,
                external_input_enabled,
                external_input_receiver,
                external_input_request_sender,
                on_off_state_inner,
                on_off,
                switch_on_off_delay,
            );

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
            manual_instruction_sender,
            on_off_state,
        }
    }
    pub fn query_g_task(&mut self, task: Next3dMovement) {
        self.task_query
            .lock()
            .unwrap()
            .push(Task::ProgramMovement(task));
    }
    pub fn query_m_task(&mut self, task: NextMiscellaneous) {
        self.task_query
            .lock()
            .unwrap()
            .push(Task::ProgramMiscellaneous(task));
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
        self.steps_todo.load(Relaxed)
    }
    pub fn get_steps_done(&self) -> i64 {
        self.steps_done.load(Relaxed)
    }
    pub fn is_switched_on(&self) -> bool {
        self.on_off_state.load(Relaxed)
    }
    pub fn manual_move(&mut self, x: f64, y: f64, z: f64, speed: f64) {
        if self
            .manual_instruction_sender
            .send(ManualInstruction::Movement(ManualTask {
                move_x_speed: x,
                move_y_speed: y,
                move_z_speed: z,
                speed,
            }))
            .is_err()
        {
            println!("can't send manual move");
        }
    }
    pub fn manual_miscellaneous(&mut self, task: NextMiscellaneous) {
        if self
            .manual_instruction_sender
            .send(ManualInstruction::Miscellaneous(task))
            .is_err()
        {
            println!("can't send manual miscellaneous task");
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
