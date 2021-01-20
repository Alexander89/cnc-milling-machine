#![allow(dead_code)]
pub mod mock_motor;
pub mod motor_controller;
pub mod motor_controller_thread;
pub mod step_motor;
pub mod task;

use crate::types::Direction;
use log::{max_level, LevelFilter};
use std::{
    fmt::Debug,
    result,
    sync::Mutex,
    sync::{
        atomic::{AtomicI64, Ordering::Relaxed},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

pub use mock_motor::MockMotor;
pub use step_motor::StepMotor;
pub type Result<T> = result::Result<T, &'static str>;

pub trait Driver: std::fmt::Debug {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction>;
    fn get_step_size(&self) -> f64;
}

#[derive(Debug)]
pub struct MotorInner {
    name: String,
    max_step_speed: u32, // steps per second
    driver: Box<dyn Driver + Send>,
}

#[derive(Debug)]
pub struct Motor {
    pos: Arc<AtomicI64>,
    inner: Arc<Mutex<MotorInner>>,
    step_size: f64, // mm per step
    last_step: SystemTime,
    pref_delta_t: f64,
    speed: u64,
}

impl Motor {
    pub fn new(name: String, max_step_speed: u32, driver: Box<dyn Driver + Send>) -> Self {
        Motor {
            pos: Arc::new(AtomicI64::new(0)),
            step_size: driver.get_step_size(),
            inner: Arc::new(Mutex::new(MotorInner {
                name,
                max_step_speed, // steps per second
                driver,
            })),
            last_step: SystemTime::now(),
            pref_delta_t: 1.0,
            speed: 1000,
        }
    }
    pub fn step(&mut self, direction: Direction) -> Result<()> {
        // block motor to have a smooth ramp
        // this wil slow slow down the complete program, because all motors run in one thread
        // the slowest motor do also slow down the rest

        // first step after a brake could be done immediately
        if self.last_step.elapsed().unwrap() > Duration::from_millis(50) {
            self.speed = 1000;
        } else {
            self.speed += (6000 - self.speed) / (self.speed / 50);
            // println!(
            //     "{} {}",
            //     self.speed,
            //     self.last_step.elapsed().unwrap().as_nanos()
            // );
            let req_delay = Duration::from_micros(5000 - self.speed.min(5000));
            let delta_t = self.last_step.elapsed().unwrap();
            if delta_t < req_delay {
                let open_delay = req_delay.as_nanos() - delta_t.as_nanos();
                thread::sleep(Duration::new(0, open_delay as u32));
            };
        }
        self.pref_delta_t = self.last_step.elapsed().unwrap().as_secs_f64();
        self.last_step = SystemTime::now();

        match (*self.inner.lock().unwrap().driver).do_step(&direction) {
            Ok(Direction::Left) => {
                if max_level() == LevelFilter::Debug {
                    print!("-");
                }
                (*self.pos).fetch_sub(1, Relaxed);
                Ok(())
            }
            Ok(Direction::Right) => {
                if max_level() == LevelFilter::Debug {
                    print!("+");
                }
                (*self.pos).fetch_add(1, Relaxed);
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }
    pub fn get_pos_ref(&self) -> Arc<AtomicI64> {
        self.pos.clone()
    }
    pub fn get_step_size(&self) -> f64 {
        self.step_size
    }
}
