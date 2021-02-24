#![allow(dead_code)]
pub mod mock_motor;
pub mod motor_controller;
pub mod motor_controller_thread;
pub mod step_motor;
pub mod task;

use crate::types::Direction;
use log::{max_level, LevelFilter};
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorSettings {
    pub max_step_speed: u32,
    pub pull_gpio: u8,
    pub dir_gpio: u8,
    pub invert_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ena_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_left_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_right_gpio: Option<u8>,
    pub step_size: f64,
    // max acceleration constance
    pub acceleration: f64,
    // reduce the acceleration on higher speed,
    pub acceleration_damping: f64,
    // speed that requires no acceleration. It just runs with that.
    pub free_step_speed: f64,
    // value to adjust UI Graph
    pub acceleration_time_scale: f64,
}

pub trait Driver: std::fmt::Debug {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction>;
    fn get_step_size(&self) -> f64;
    fn is_blocked(&mut self) -> Option<Direction>;
}

#[derive(Debug)]
pub struct MotorInner {
    name: String,
    max_step_speed: u64, // steps per second
    driver: Box<dyn Driver + Send>,
}

#[derive(Debug)]
pub struct Motor {
    pos: Arc<AtomicI64>,
    step_size: f64,
    inner: Arc<Mutex<MotorInner>>,
    // for speed
    t_pre_last: SystemTime,
    t_last: SystemTime,
    // values to ramp up the motor speed
    max_step_speed: f64, // [step / sec]
    acceleration: f64,
    acceleration_damping: f64,
    free_step_speed: f64, // [step / sec]
}

impl Motor {
    pub fn new(
        name: String,
        max_step_speed: f64,
        acceleration: f64,
        acceleration_damping: f64,
        free_step_speed: f64,
        driver: Box<dyn Driver + Send>,
    ) -> Self {
        Motor {
            pos: Arc::new(AtomicI64::new(0)),
            step_size: driver.get_step_size(),
            inner: Arc::new(Mutex::new(MotorInner {
                name,
                max_step_speed: max_step_speed as u64, // steps per second
                driver,
            })),

            t_pre_last: SystemTime::now()
                .checked_sub(Duration::from_secs(5))
                .unwrap_or(SystemTime::UNIX_EPOCH),
            t_last: SystemTime::now(),

            max_step_speed,
            acceleration,
            acceleration_damping,
            free_step_speed,
        }
    }
    /**
     * @return The time the motor was blocked.
     */
    pub fn step(&mut self, direction: Direction) -> f64 {
        // block motor to have a smooth ramp
        // this will slow slow down all motors, because all motors run in one thread

        // delta T from last to pre-last step
        let d_t_last = self
            .t_last
            .duration_since(self.t_pre_last)
            .unwrap()
            .as_secs_f64();
        // current_speed_st_p_s to messure the max next speed (min free_step_speed as offset for max next speed)
        // graph: const v = Math.max(startMotorSpeed, vLast)
        let current_speed_st_p_s = (1.0f64 / d_t_last).max(self.free_step_speed);
        // calc max speed:
        // graph: const maxSpeed = speedCurrent + acceleration - speedCurrent * damping
        let max_speed = current_speed_st_p_s + self.acceleration
            - (current_speed_st_p_s * self.acceleration_damping);
        // upper bound; how long the the motor have to wait at least
        let min_delta_t = 1.0f64 / max_speed;

        // delta T to last step (speed the motor/program request to run. (needs to be decelerated if it is faster than allowed))
        let d_t = self.t_last.elapsed().unwrap().as_secs_f64();

        // block if the required wait time is larger the the elapsed time
        let blocked = if min_delta_t > d_t {
            let required_wait_for = min_delta_t - d_t;
            thread::sleep(Duration::from_secs_f64(required_wait_for));
            required_wait_for
        } else {
            0.0f64
        };
        self.t_pre_last = self.t_last;
        self.t_last = SystemTime::now();

        // do step now

        match (*self.inner.lock().unwrap().driver).do_step(&direction) {
            Ok(Direction::Left) => {
                if max_level() == LevelFilter::Debug {
                    print!("-");
                }
                (*self.pos).fetch_sub(1, Relaxed);
            }
            Ok(Direction::Right) => {
                if max_level() == LevelFilter::Debug {
                    print!("+");
                }
                (*self.pos).fetch_add(1, Relaxed);
            }
            Err(_) => (),
        };
        blocked
    }
    pub fn is_blocked(&mut self) -> Option<Direction> {
        (*self.inner.lock().unwrap().driver).is_blocked()
    }
    pub fn get_pos_ref(&self) -> Arc<AtomicI64> {
        self.pos.clone()
    }
    pub fn get_step_size(&self) -> f64 {
        self.step_size
    }
}
