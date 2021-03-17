pub mod mock_motor;
pub mod step_motor;

use crate::{settings::DriverType, types::Direction};
use log::{max_level, LevelFilter};
use std::{fmt::Debug, result};

pub use mock_motor::MockMotor;
pub use step_motor::StepMotor;
pub type Result<T> = result::Result<T, &'static str>;

#[derive(Clone, Debug)]
pub struct SettingsMotor {
    pub driver_settings: DriverType,
    // acceleration constance
    pub acceleration: f64,
    // deceleration constance
    pub deceleration: f64,
}

pub trait Driver: std::fmt::Debug {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction>;
    fn is_blocked(&mut self) -> Option<Direction>;
}

#[derive(Debug)]
pub struct Motor {
    settings: SettingsMotor,
    name: String,
    pos: i64,
    driver: Box<dyn Driver + Send>,
}

impl Motor {
    pub fn new(name: String, settings: SettingsMotor) -> Self {
        let driver: Box<dyn Driver + Send> = match settings.driver_settings.clone() {
            DriverType::Mock => Box::new(MockMotor::new()),
            DriverType::Stepper(settings) => Box::new(StepMotor::from_settings(settings))
        };
        println!("init motor {}", name);
        Motor {
            name,
            pos: 0,
            settings,
            driver,
        }
    }
    /**
     * @return The time the motor was blocked.
     */
    pub fn step(&mut self, direction: Direction) -> Result<()> {
        // block motor to have a smooth ramp
        // this will slow slow down all motors, because all motors run in one thread
        // do step now

        match (*self.driver).do_step(&direction) {
            Ok(Direction::Left) => {
                if max_level() == LevelFilter::Debug {
                    print!("-");
                }
                self.pos -= 1;
                Ok(())
            }
            Ok(Direction::Right) => {
                if max_level() == LevelFilter::Debug {
                    print!("+");
                }
                self.pos += 1;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    #[allow(dead_code)]
    pub fn is_blocked(&mut self) -> Option<Direction> {
        (*self.driver).is_blocked()
    }
    pub fn get_pos(&self) -> i64 {
        self.pos
    }
}
