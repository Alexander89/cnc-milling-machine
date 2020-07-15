#![allow(dead_code)]
use super::{
    task::{Direction, ProgramTask, Task},
    AutonomeMotor, CommandOwner, Motor,
};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug)]
pub struct MockMotor {
    max_step_speed: u32, // steps per second
    step_pos: i32,       // + - from the reset point
    step_size: f64,      // mm per step
    current_task: Option<Task>,
    tasks: Vec<Task>,
    current_direction: Direction,
}

impl MockMotor {
    pub fn new(max_step_speed: u32, step_size: f64) -> Arc<Mutex<MockMotor>> {
        let motor_ref = Arc::new(Mutex::new(MockMotor {
            step_pos: 0i32,
            max_step_speed: max_step_speed,
            current_task: None,
            tasks: vec![],
            current_direction: Direction::LEFT,
            step_size: step_size,
        }));
        MockMotor::start(motor_ref)
    }
    fn start(motor_ref: Arc<Mutex<MockMotor>>) -> Arc<Mutex<MockMotor>> {
        let inner_motor = motor_ref.clone();
        std::thread::spawn(move || {
            loop {
                thread::sleep(Duration::new(0, 1000));
                let mut driver = inner_motor.lock().unwrap();
                if driver.current_task.is_none() {
                    if driver.tasks.len() > 0 {
                        driver.current_task = driver.tasks.pop();
                    } else {
                        drop(driver);
                        // unlock driver and continue after rest
                        thread::sleep(Duration::new(0, 100_000));
                    }
                } else {
                    driver.poll();
                }
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
            if let Some(dir) = task.is_step_required() {
                task.step_done();
                match dir {
                    Direction::LEFT => self.step_pos -= 1,
                    Direction::RIGHT => self.step_pos += 1,
                }
            }
        }
        Ok(())
    }
}

impl AutonomeMotor for MockMotor {
    fn exec_task(&mut self, task: ProgramTask) -> Result<(), ()> {
        if self.current_task.is_none() {
            self.current_task = Some(Task::PROGRAM(task));
            Ok(())
        } else {
            Err(())
        }
    }
    fn query_task(&mut self, new_task: ProgramTask) -> () {
        self.tasks.push(Task::PROGRAM(new_task));
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
