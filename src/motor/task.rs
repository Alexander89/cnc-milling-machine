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
    pub sqn: u32,
    direction: Direction,
    pub destination: f64,
    steps_done: u64,
    pub start_time: SystemTime,
    pub duration: Duration,
    move_type: MoveType,

    state: TaskState,
}

#[derive(Debug, Clone)]
pub struct ManualTask {
    direction: Direction,
    start_time: SystemTime,
    speed: f32,      // steps per second
    step_count: u32, // steps done in this task
}
#[derive(Debug, Clone)]
pub enum Task {
    PROGRAM(ProgramTask),
    MANUAL(ManualTask),
    NOOP(u32, Duration),
}

impl Task {
    pub fn manual(direction: Direction, speed: f32) -> Task {
        Task::MANUAL(ManualTask {
            direction: direction,
            speed: speed,
            start_time: SystemTime::now(),
            step_count: 0,
        })
    }
    pub fn program(
        sqn: u32,
        direction: Direction,
        destination: f64,
        duration: Duration,
        move_type: MoveType,
    ) -> Task {
        Task::PROGRAM(ProgramTask {
            sqn: sqn,
            direction: direction,
            destination: destination,
            start_time: SystemTime::now(),
            steps_done: 0,
            duration: duration,
            move_type: move_type,
            state: TaskState::Idle,
        })
    }
    pub fn is_step_required(&self, step_size: f64) -> Option<Direction> {
        match self {
            Task::PROGRAM(task) => {
                if let Ok(elapsed) = task.start_time.elapsed() {
                    let steps_to_do = task.destination / step_size;
                    let steps_left = steps_to_do.round() as u64 - task.steps_done;
                    if steps_left == 0 {
                        return None;
                    }
                    let proc = elapsed.as_secs_f64() / task.duration.as_secs_f64();
                    let scheduled_steps = steps_to_do * proc;

                    if task.steps_done < scheduled_steps.round() as u64 {
                        Some(task.direction.to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Task::MANUAL(task) => {
                if let Ok(elapsed) = task.start_time.elapsed() {
                    let duration = 1.0f32 / task.speed;
                    if elapsed.as_secs_f32() > duration * task.step_count as f32 {
                        Some(task.direction.to_owned())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Task::NOOP(_, _) => None,
        }
    }
    pub fn step_done(&mut self) {
        match self {
            Task::PROGRAM(task) => {
                task.steps_done = task.steps_done + 1;
            }
            Task::MANUAL(task) => {
                task.step_count = task.step_count + 1;
            }
            Task::NOOP(_, _) => {}
        }
    }
}
