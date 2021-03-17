use super::{Driver, Result};
use crate::{io::{Switch, Actor}, settings::StepperDriverSettings};
use crate::types::Direction;
use std::fmt::Debug;

#[derive(Debug)]
pub struct StepMotor {
    name: String,
    pull: Actor,
    direction: Actor,
    invert_dir: bool,
    enable: Option<Actor>,
    step_pos: i32, // + - from the reset point
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
    ) -> Self {
        let name = format!("Stepper p:{} d:{} inv:{}", pull, dir, invert_dir);
        let ena_gpio = ena.map(|pin| Actor::new(pin, false, false));
        let left = end_left.map(|pin| Switch::new(pin, true));
        let right = end_right.map(|pin| Switch::new(pin, true));

        StepMotor {
            name,
            pull: Actor::new(pull, false, false),
            direction: Actor::new(dir, false, false),
            invert_dir,
            enable: ena_gpio,
            step_pos: 0i32,
            end_switch_left: left,
            end_switch_right: right,
            current_direction: Direction::Left,
        }
    }
    pub fn from_settings(settings: StepperDriverSettings) -> Self {
        StepMotor::new(
            settings.pull_gpio,      // 18,
            settings.dir_gpio,       // 27,
            settings.invert_dir,     // false,
            settings.ena_gpio,       // None,
            settings.end_left_gpio,  // Some(21),
            settings.end_right_gpio, // Some(20),
        )
    }
    fn is_blocked(&mut self) -> bool {
        let switch_opt = match (self.current_direction, self.invert_dir) {
            (Direction::Left, false) => self.end_switch_left.as_mut(),
            (Direction::Left, true) => self.end_switch_right.as_mut(),
            (Direction::Right, false) => self.end_switch_right.as_mut(),
            (Direction::Right, true) => self.end_switch_left.as_mut(),
        };
        switch_opt.map(|switch| switch.is_closed()).unwrap_or(false)
    }
}

impl Driver for StepMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            match (direction, self.invert_dir, self.direction.is_high()) {
                (Direction::Left, false, true) => self.direction.set_low(),
                (Direction::Left, true, false) => self.direction.set_high(),
                (Direction::Right, false, false) => self.direction.set_high(),
                (Direction::Right, true, true) => self.direction.set_low(),
                _ => (),
            }
            self.current_direction = *direction;
        }
        if self.is_blocked() {
            Err("is blocked at end")
        } else {
            self.pull.toggle();
            Ok(self.current_direction)
        }
    }
    fn is_blocked(&mut self) -> Option<Direction> {
        if let Some(switch) = self.end_switch_left.as_mut() {
            match (switch.is_closed(), self.invert_dir) {
                (true, false) => return Some(Direction::Left),
                (true, true) => return Some(Direction::Right),
                (_, _) => (),
            };
        }
        if let Some(switch) = self.end_switch_right.as_mut() {
            match (switch.is_closed(), self.invert_dir) {
                (true, false) => return Some(Direction::Right),
                (true, true) => return Some(Direction::Left),
                (_, _) => (),
            };
        }
        None
    }
}
