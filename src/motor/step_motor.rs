use super::{
    task::{Direction, Task},
    AutonomeMotor, CommandOwner, Motor,
};
use crate::switch::Switch;
use rppal::gpio::{Gpio, OutputPin};

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct StepMotor {
    pull: OutputPin,
    direction: OutputPin,
    enable: Option<OutputPin>,
    max_step_speed: u32, // steps per second
    step_pos: i32,       // + - from the reset point
    step_size: f64,      // mm per step
    end_switch_left: Option<Switch>,
    end_switch_right: Option<Switch>,
    current_task: Option<Task>,
    tasks: Vec<Task>,
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
    ) -> Arc<Mutex<StepMotor>> {
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

        let mut direction = Gpio::new().unwrap().get(dir).unwrap().into_output();
        direction.set_low();

        let motor_ref = Arc::new(Mutex::new(StepMotor {
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction: direction,
            enable: ena_gpio,
            step_pos: 0i32,
            max_step_speed: max_step_speed,
            end_switch_left: left,
            end_switch_right: right,
            current_task: None,
            tasks: vec![],
            current_direction: Direction::LEFT,
            step_size: step_size,
        }));
        StepMotor::start(motor_ref)
    }
    fn start(motor_ref: Arc<Mutex<StepMotor>>) -> Arc<Mutex<StepMotor>> {
        let inner_motor = motor_ref.clone();
        std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::new(0, 1));
                let mut driver = inner_motor.lock().unwrap();
                if driver.current_task.is_none() {
                    if driver.tasks.len() > 0 {
                        driver.current_task = driver.tasks.pop();
                    } else {
                        drop(driver);
                        // unlock driver and continue after rest
                        thread::sleep(Duration::new(0, 1000));
                    }
                } else {
                    let _ = driver.poll();
                }
            }
        });
        motor_ref
    }
}

impl Motor for StepMotor {
    fn reset(&mut self) -> &mut Self {
        self.step_pos = 0;
        self
    }
    fn get_pos(&self) -> f64 {
        let step_float: f64 = self.step_pos.into();
        step_float * self.step_size
    }

    fn poll(&mut self) -> Result<(), ()> {
        if let Some(task) = self.current_task.as_mut() {
            if let Some(dir) = task.is_step_required(self.step_size) {
                fn step_done(step_pos: &mut i32, task: &mut Task, dir: &Direction) -> () {
                    task.step_done();
                    match dir {
                        Direction::LEFT => *step_pos -= 1,
                        Direction::RIGHT => *step_pos += 1,
                    }
                }
                let mut end_switch = match dir {
                    Direction::LEFT => {
                        if self.direction.is_set_high() {
                            self.direction.set_low();
                            thread::sleep(Duration::new(0, 3_000));
                        }
                        self.end_switch_left.as_mut()
                    }
                    Direction::RIGHT => {
                        if self.direction.is_set_low() {
                            self.direction.set_high();
                            thread::sleep(Duration::new(0, 3_000));
                        }
                        self.end_switch_right.as_mut()
                    }
                };

                let res = if let Some(switch) = end_switch.as_mut() {
                    if switch.is_closed() {
                        Err(())
                    } else {
                        self.pull.toggle();
                        step_done(&mut self.step_pos, task, &dir);
                        Ok(())
                    }
                } else {
                    self.pull.toggle();
                    step_done(&mut self.step_pos, task, &dir);
                    Ok(())
                };

                res
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl AutonomeMotor for StepMotor {
    fn exec_task(&mut self, task: Task) -> Result<(), ()> {
        if self.current_task.is_none() {
            self.current_task = Some(task);
            Ok(())
        } else {
            Err(())
        }
    }
    fn query_task(&mut self, new_task: Task) -> () {
        self.tasks.push(new_task);
    }
    fn manual_move(&mut self, direction: Direction, speed: f32) -> Result<(), ()> {
        self.cancel_task(&CommandOwner::MANUAL)?;
        self.current_task = Some(Task::manual(direction, speed));
        Ok(())
    }
    fn cancel_task(&mut self, interrupter: &CommandOwner) -> Result<(), ()> {
        if let Some(task) = &self.current_task {
            match interrupter {
                CommandOwner::PROGRAM => {
                    self.current_task = None; //@todo: check this
                    Ok(())
                }
                CommandOwner::MANUAL => {
                    if let Task::MANUAL(_) = task {
                        self.current_task = None; //@todo: check this
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
