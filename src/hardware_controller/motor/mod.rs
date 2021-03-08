pub mod mock_motor;
pub mod step_motor;

use crate::types::Direction;
use log::{max_level, LevelFilter};
use std::{fmt::Debug, result};

pub use mock_motor::MockMotor;
pub use step_motor::StepMotor;
pub type Result<T> = result::Result<T, &'static str>;

#[derive(Clone, Debug)]
pub struct SettingsMotor {
    pub pull_gpio: u8,
    pub dir_gpio: u8,
    pub invert_dir: bool,
    pub ena_gpio: Option<u8>,
    pub end_left_gpio: Option<u8>,
    pub end_right_gpio: Option<u8>,
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
    pub fn new(name: String, settings: SettingsMotor, driver: Box<dyn Driver + Send>) -> Self {
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
