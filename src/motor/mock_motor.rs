use super::{Driver, Result};
use crate::types::Direction;
use std::fmt::Debug;

#[derive(Debug)]
pub struct MockMotor {
    current_direction: Direction,
    step_size: f64,
}

impl MockMotor {
    pub fn new(step_size: f64) -> Self {
        MockMotor {
            current_direction: Direction::Left,
            step_size,
        }
    }
}

impl Driver for MockMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            self.current_direction = direction.clone();
        };

        Ok(direction.clone())
    }
    fn get_step_size(&self) -> f64 {
        self.step_size
    }
    fn is_blocked(&mut self) -> Option<Direction> {
        None
    }
}
