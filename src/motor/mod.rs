use std::time::{Duration, SystemTime};
use rppal::gpio::{Gpio, OutputPin};
use super::switch::Switch;

#[derive(Debug, Clone)]
pub enum Direction {
    LEFT,
    RIGHT,
}

#[derive(Debug, Clone)]
struct ProgramTask {
    direction: Direction,
    start: u32,
    destrination: u32,
    steps_done: u32,
    start_time: SystemTime,
    duration: Duration,
}
#[derive(Debug, Clone)]
struct ManualTask {
    direction: Direction,
    start_time: SystemTime,
    speed: f32, // steps per second
    step_count: f32, // steps done in this task
}
#[derive(Debug, Clone)]
enum Task {
    PROGRAM(ProgramTask),
    MANUAL(ManualTask),
}
pub enum CommandOwner {
    PROGRAM,
    MANUAL
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
    pub fn is_step_required(&self) -> Option<Direction> {
        match self {
            Task::PROGRAM(_task) => {
                None
            },
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
            },
        }
    }
    pub fn step_done(&mut self) {
        match self {
            Task::PROGRAM(_task) => {
                return
            },
            Task::MANUAL(task) => {
                task.step_count = task.step_count + 1.0f32;
            },
        }
    }
}


#[derive(Debug)]
pub struct Motor {
    pull: OutputPin,
    direction: OutputPin,
    enable: Option<OutputPin>,
    max_step_speed: u32, // steps per second
    step_pos: i32,      // + - from the reset point
    end_switch_left: Option<Switch>,
    end_switch_right: Option<Switch>,
    task: Option<Task>,
}

impl Motor {
    pub fn new(pull: u8, dir: u8, ena: Option<u8>, end_left: Option<u8>, end_right: Option<u8>, max_step_speed: u32) -> Motor {
        let ena_gpio = if let Some(ena_pin) = ena {
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

        Motor {
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction: Gpio::new().unwrap().get(dir).unwrap().into_output(),
            enable: ena_gpio,
            step_pos: 0i32,
            max_step_speed: max_step_speed,
            end_switch_left: left,
            end_switch_right: right,
            task: None,
        }
    }
    pub fn manual_move(&mut self, direction: Direction, speed: f32) -> Result<(), ()> {
        self.cancle_task(&CommandOwner::MANUAL)?;
        self.task = Some(Task::manual(direction, speed));
        Ok(())
    }
    pub fn cancle_task(&mut self, interupter: &CommandOwner) -> Result<(), ()> {
        if let Some(task) = &self.task {
            match interupter {
                CommandOwner::PROGRAM => {
                    self.task = None; //@todo: check this
                    Ok(())
                },
                CommandOwner::MANUAL => {
                    if let Task::MANUAL(_) = task {
                        self.task = None; //@todo: check this
                        Ok(())
                    } else {
                        Err(())
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Motor {
    pub fn poll(&mut self) -> Result<(), ()> {
        if let Some(task) = self.task.as_mut() {
            if let Some(dir) = task.is_step_required() {
                let mut end_switch = match dir {
                    Direction::LEFT => {
                        self.direction.set_low();
                        self.end_switch_left.as_mut()
                    },
                    Direction::RIGHT => {
                        self.direction.set_high();
                        self.end_switch_right.as_mut()
                    },
                };

                if let Some(switch) = end_switch.as_mut() {
                    if switch.is_closed() {
                        Err(())
                    } else {
                        self.pull.toggle();
                        task.step_done();
                        Ok(())
                    }
                } else {
                    self.pull.toggle();
                    task.step_done();
                    Ok(())
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
