mod motor;
mod switch;
use crate::motor::{Motor, Direction};

fn main() {
    let mut motor = Motor::new(18, 23, None, Some(26), None, 1500);

    motor.manual_move(Direction::LEFT, 800.0f32);
    loop {
        motor.poll();
    }
}
