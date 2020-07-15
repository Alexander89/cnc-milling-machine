use super::Motor;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveType {
    LINEAR,
    SINUS,
    COSINE,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Idle,
    Running,
    Complete,
    Failed,
    Canceled,
}

#[derive(Debug, Clone)]
pub struct ProgramTask {
    direction: Direction,
    start: u32,
    destination: u32,
    steps_done: f64,
    start_time: SystemTime,
    duration: Duration,
    move_type: MoveType,

    state: TaskState,
}

#[derive(Debug, Clone)]
pub struct ManualTask {
    direction: Direction,
    start_time: SystemTime,
    speed: f32,      // steps per second
    step_count: f32, // steps done in this task
}
#[derive(Debug, Clone)]
pub enum Task {
    PROGRAM(ProgramTask),
    MANUAL(ManualTask),
}

impl Task {
    pub fn manual(direction: Direction, speed: f32) -> Task {
        Task::MANUAL(ManualTask {
            direction: direction,
            speed: speed,
            start_time: SystemTime::now(),
            step_count: 0f32,
        })
    }
    pub fn program(
        direction: Direction,
        start: u32,
        destination: u32,
        duration: Duration,
        move_type: MoveType,
    ) -> Task {
        Task::PROGRAM(ProgramTask {
            direction: direction,
            start: start,
            destination: destination,
            start_time: SystemTime::now(),
            steps_done: 0f64,
            duration: duration,
            move_type: move_type,
            state: TaskState::Idle,
        })
    }
    pub fn is_step_required(&self) -> Option<Direction> {
        match self {
            Task::PROGRAM(_task) => None,
            Task::MANUAL(task) => {
                if let Ok(elapsed) = task.start_time.elapsed() {
                    let duration = 1.0f32 / task.speed;
                    if elapsed.as_secs_f32() > duration * task.step_count {
                        Some(task.direction.to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
    pub fn step_done(&mut self) {
        match self {
            Task::PROGRAM(_task) => return,
            Task::MANUAL(task) => {
                task.step_count = task.step_count + 1.0f32;
            }
        }
    }
}
