#![allow(dead_code)]
use super::{
    task::{Direction, Task},
    AutonomeMotor, CommandOwner, Motor,
};
use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, SystemTime},
};

#[derive(Debug)]
pub struct MockMotor {
    name: String,
    max_step_speed: u32, // steps per second
    step_pos: i32,       // + - from the reset point
    step_size: f64,      // mm per step
    current_task: Option<Task>,
    tasks: Vec<Task>,
    current_direction: Direction,
}

impl MockMotor {
    pub fn new(name: String, max_step_speed: u32, step_size: f64) -> Arc<RwLock<MockMotor>> {
        let motor_ref = Arc::new(RwLock::new(MockMotor {
            name: name,
            step_pos: 0i32,
            max_step_speed: max_step_speed,
            current_task: None,
            tasks: vec![],
            current_direction: Direction::LEFT,
            step_size: step_size,
        }));
        MockMotor::start(motor_ref)
    }
    fn start(motor_ref: Arc<RwLock<MockMotor>>) -> Arc<RwLock<MockMotor>> {
        let inner_motor = motor_ref.clone();
        let name = motor_ref.read().unwrap().name.clone();
        std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::new(0, 1000));
                let driver = inner_motor.read().unwrap();
                match driver.current_task {
                    None => {
                        if driver.tasks.len() > 0 {
                            drop(driver);
                            let mut driver_write = inner_motor.write().unwrap();
                            driver_write.current_task = Some(driver_write.tasks.remove(0));
                            match driver_write.current_task.as_mut().unwrap() {
                                Task::PROGRAM(p) => {
                                    println!(
                                        "{} {} - start new task {} {:?}",
                                        name, p.sqn, p.destination, p.duration
                                    );
                                    p.start_time = SystemTime::now();
                                }
                                Task::NOOP(sqn, d) => {
                                    println!("{} {} - sleep for noop {:?}", name, sqn, d);
                                    thread::sleep(d.clone());
                                    driver_write.current_task = None;
                                }
                                _ => {}
                            }
                        } else {
                            drop(driver);
                            // unlock driver and continue after rest
                            thread::sleep(Duration::new(0, 100_000));
                        }
                    }
                    Some(_) => {
                        drop(driver);
                        let mut driver_write = inner_motor.write().unwrap();
                        let _ = driver_write.poll();
                    }
                };
            }
        });
        motor_ref
    }
}
impl Motor for MockMotor {
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
                task.step_done();
                match dir {
                    Direction::LEFT => self.step_pos -= 1,
                    Direction::RIGHT => self.step_pos += 1,
                }
            } else {
                match task {
                    Task::PROGRAM(p) if p.start_time.elapsed().unwrap() >= p.duration => {
                        self.current_task = None
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl AutonomeMotor for MockMotor {
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
        if let Some(current_task) = &self.current_task {
            match interrupter {
                CommandOwner::PROGRAM => {
                    self.current_task = None; //@todo: check this
                    Ok(())
                }
                CommandOwner::MANUAL => {
                    if let Task::MANUAL(_) = current_task {
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
