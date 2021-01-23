use super::{Driver, Result};
use crate::io::Switch;
use crate::types::Direction;
use rppal::gpio::{Gpio, OutputPin};
use std::fmt::Debug;

#[derive(Debug)]
pub struct StepMotor {
    name: String,
    pull: OutputPin,
    direction: OutputPin,
    invert_dir: bool,
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
        invert_dir: bool,
        ena: Option<u8>,
        end_left: Option<u8>,
        end_right: Option<u8>,
        max_step_speed: u32,
        step_size: f64,
    ) -> Self {
        let name = format!("Stepper p{} d{} ", pull, dir);
        let ena_gpio = ena.map(|pin| Gpio::new().unwrap().get(pin).unwrap().into_output());
        let left = end_left.map(|pin| Switch::new(pin, true));
        let right = end_right.map(|pin| Switch::new(pin, true));

        let mut direction = Gpio::new().unwrap().get(dir).unwrap().into_output();
        direction.set_low();

        StepMotor {
            name,
            pull: Gpio::new().unwrap().get(pull).unwrap().into_output(),
            direction,
            invert_dir,
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
        switch_opt.map(|switch| switch.is_closed()).unwrap_or(false)
    }
}

impl Driver for StepMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            match (direction, self.invert_dir, self.direction.is_set_high()) {
                (Direction::Left, false, true) => self.direction.set_low(),
                (Direction::Left, true, false) => self.direction.set_high(),
                (Direction::Right, false, false) => self.direction.set_high(),
                (Direction::Right, true, true) => self.direction.set_low(),
                _ => (),
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
