mod motor;
mod switch;
mod program;
use crate::motor::{Motor, Direction, CommandOwner};
use crate::program::{Program};

use gilrs::{Gilrs, Axis, Event, Button, EventType};

fn main() {
    let mut gilrs = Gilrs::new().map_err(|_| "gamepad not valid").expect("controler is missing");
    // Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut motor = Motor::new(18, 23, None, Some(26), None, 5000);

    motor.manual_move(Direction::LEFT, 1000.0f32);
    let prog = Program::new("./example.gcode").expect("failed to load file");
    prog.run_program();
    'running: loop {
        motor.poll();
        // map GamePad events to drone
        while let Some(Event { event, .. }) = gilrs.next_event() {
            match event {
                EventType::ButtonReleased(Button::Mode, _) => {
                    break 'running;
                }    
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    if value > 0.07f32 {
                        println!("left {}", value);
                        let speed: f32 = value * 5000.0f32;
                        motor.manual_move(Direction::LEFT, speed);
                    } else if value < -0.07f32 {
                        println!("right {}", value);
                        let speed: f32 = value * -5000.0f32;
                        motor.manual_move(Direction::RIGHT, speed);
                    } else {
                        println!("stop");
                        motor.cancle_task(&CommandOwner::MANUAL);             
                    }
                }
                _ => {}
            }
        };
    }
}
