#![allow(dead_code)]

pub mod mock_motor;
pub mod step_motor;
pub mod task;

pub use mock_motor::MockMotor;
pub use step_motor::StepMotor;
pub use task::{Direction, MoveType, ProgramTask, Task};

pub enum CommandOwner {
    PROGRAM,
    MANUAL,
}

pub trait Motor {
    fn reset(&mut self) -> &mut Self;
    fn get_pos(&self) -> f64;
    fn poll(&mut self) -> Result<(), ()>;
}

pub trait AutonomeMotor {
    fn exec_task(&mut self, task: Task) -> Result<(), ()>;
    fn query_task(&mut self, task: Task) -> ();
    fn manual_move(&mut self, direction: Direction, speed: f32) -> Result<(), ()>;
    fn cancel_task(&mut self, interrupter: &CommandOwner) -> Result<(), ()>;
}
