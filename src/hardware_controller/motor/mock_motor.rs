use super::{Driver, Result};
use crate::types::Direction;
use std::fmt::Debug;

#[derive(Debug)]
pub struct MockMotor {
    current_direction: Direction,
}

impl MockMotor {
    pub fn new() -> Self {
        MockMotor {
            current_direction: Direction::Left,
        }
    }
}

impl Driver for MockMotor {
    fn do_step(&mut self, direction: &Direction) -> Result<Direction> {
        if self.current_direction != *direction {
            self.current_direction = direction.clone();
        };

        Ok(*direction)
    }
    fn is_blocked(&mut self) -> Option<Direction> {
        None
    }
}
