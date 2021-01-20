use crate::io::Switch;
use crate::types::Direction;
use rppal::gpio::{Gpio, OutputPin};
use std::fmt::Debug;
use super::{Driver, Result};


#[derive(Debug)]
pub struct StepMotor {
    name: String,
    pull: OutputPin,
    direction: OutputPin,
    enable: Option<OutputPin>,
    max_step_speed: u32, // steps per second
    step_pos: i32,       // + - from the reset point
    step_size: f64,      // mm per step
    end_switch_left: Option<Switch>,
    end_switch_right: Option<Switch>,
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
    ) -> Self {
        let mut name = format!("Stepper p{} d{} ", pull, dir);
        let ena_gpio = if let Some(ena_pin) = ena {
            name = format!("{} e{} ", name, ena_pin);
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

        StepMotor {
            name,
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction,
            enable: ena_gpio,
            step_pos: 0i32,
            max_step_speed,
            end_switch_left: left,
            end_switch_right: right,
            current_direction: Direction::Left,
            step_size,
        }
    }
    fn is_blocked(&mut self) -> bool {
        let switch_opt = match self.current_direction {
            Direction::Left => self.end_switch_left.as_mut(),
            Direction::Right => self.end_switch_right.as_mut(),
        };
        if let Some(switch) = switch_opt {
            switch.is_closed()
        } else {
            false
        }
    }
}

impl Driver for StepMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            match direction {
                Direction::Left => {
                    if self.direction.is_set_high() {
                        self.direction.set_low();
                        //thread::sleep(Duration::new(0, 3));
                    }
                }
                Direction::Right => {
                    if self.direction.is_set_low() {
                        self.direction.set_high();
                        //thread::sleep(Duration::new(0, 3));
                    }
                }
            }
            self.current_direction = direction.clone();
        }
        if self.is_blocked() {
            Err("is blocked at end")
        } else {
            self.pull.toggle();
            Ok(self.current_direction.clone())
        }
    }
    fn get_step_size(&self) -> f64 {
        self.step_size
    }
}
