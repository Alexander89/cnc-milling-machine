mod motor;
mod program;
mod switch;
use crate::motor::{CommandOwner, Direction, Motor};
use crate::program::Program;

use gilrs::{Axis, Button, Event, EventType, Gilrs};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    MANUAL,
    PROGRAM,
}

fn main() {
    let mut gilrs = Gilrs::new()
        .map_err(|_| "gamepad not valid")
        .expect("controler is missing");
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let step_size = 0.01f64; // 2mm per Round / 1.8° per step (0.02 Um per step) => 0.01 mm per step

    let mut motor_x = Motor::new(17, 24, None, None, None, 5000, step_size);
    let mut motor_y = Motor::new(18, 23, None, Some(26), None, 5000, step_size);
    let mut motor_z = Motor::new(16, 25, None, None, None, 5000, step_size);

    // motor.manual_move(Direction::LEFT, 1000.0f32).expect("should move");
    let selected_program = "./example.gcode";
    let mut current_mode: Mode = Mode::MANUAL;
    let mut input_reduce: u8 = 0;
    let mut prog: Option<program::Program> = None;

    'running: loop {
        // engine driver
        {
            let _ = motor_x.poll();
            let _ = motor_y.poll();
            let _ = motor_z.poll();
        }

        if current_mode == Mode::MANUAL {
            // controller just every 10th tick
            input_reduce += 1;
            if input_reduce < 10 {
                continue 'running;
            }
            input_reduce = 0;

            println!(
                "x {} y {} z {}",
                motor_x.get_pos(),
                motor_y.get_pos(),
                motor_z.get_pos()
            );
            // map GamePad events to update the manual program or start a programm
            while let Some(Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonReleased(Button::Select, _) => {
                        // remove later to avoid killing the machine by mistake
                        break 'running;
                    }
                    EventType::ButtonReleased(Button::Start, _)
                    | EventType::ButtonReleased(Button::Mode, _) => {
                        if let Ok(load_prog) = Program::new(selected_program) {
                            prog = Some(load_prog);
                            current_mode = Mode::PROGRAM;
                        } else {
                            println!("program is not able to load")
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                        if value > 0.07f32 {
                            println!("left {}", value);
                            let speed: f32 = value * 5000.0f32;
                            motor_y.manual_move(Direction::LEFT, speed).unwrap();
                        } else if value < -0.07f32 {
                            println!("right {}", value);
                            let speed: f32 = value * -5000.0f32;
                            motor_y.manual_move(Direction::RIGHT, speed).unwrap();
                        } else {
                            println!("stop");
                            motor_y.cancel_task(&CommandOwner::MANUAL).unwrap();
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                        if value > 0.07f32 {
                            println!("left {}", value);
                            let speed: f32 = value * 5000.0f32;
                            motor_x.manual_move(Direction::LEFT, speed).unwrap();
                        } else if value < -0.07f32 {
                            println!("right {}", value);
                            let speed: f32 = value * -5000.0f32;
                            motor_x.manual_move(Direction::RIGHT, speed).unwrap();
                        } else {
                            println!("stop");
                            motor_x.cancel_task(&CommandOwner::MANUAL).unwrap();
                        }
                    }
                    EventType::AxisChanged(Axis::RightStickY, value, _) => {
                        if value > 0.07f32 {
                            println!("left {}", value);
                            let speed: f32 = value * 1000.0f32;
                            motor_z.manual_move(Direction::LEFT, speed).unwrap();
                        } else if value < -0.07f32 {
                            println!("right {}", value);
                            let speed: f32 = value * -1000.0f32;
                            motor_z.manual_move(Direction::RIGHT, speed).unwrap();
                        } else {
                            println!("stop");
                            motor_z.cancel_task(&CommandOwner::MANUAL).unwrap();
                        }
                    }
                    EventType::ButtonPressed(Button::North, _) => {}
                    EventType::ButtonPressed(Button::South, _) => {}
                    // add cross to select a programm
                    _ => {}
                }
            }
        } else {
            if let Some(p) = prog.as_mut() {
                for next_movement in p {
                    match next_movement {
                        Some(movement) => {
                            if let Some(x) = movement.x {
                                motor_x.exec_task(x).expect("fix me in proc");
                            }
                            if let Some(y) = movement.y {
                                motor_y.exec_task(y).expect("fix me in proc");
                            }
                            if let Some(z) = movement.z {
                                motor_z.exec_task(z).expect("fix me in proc");
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}
