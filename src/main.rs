mod motor;
mod program;
mod switch;
use crate::motor::{AutonomeMotor, CommandOwner, Direction, MockMotor, Motor, StepMotor};
use crate::program::Program;
use std::{ops::DerefMut, thread, time::Duration};

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

    let motor_x = StepMotor::new(17, 24, None, None, None, 5000, step_size);
    let motor_y = StepMotor::new(18, 23, None, Some(26), None, 5000, step_size);
    let motor_z = StepMotor::new(16, 25, None, None, None, 5000, step_size);

    // let motor_x = MockMotor::new(5000, step_size);
    // let motor_y = MockMotor::new(5000, step_size);
    // let motor_z = MockMotor::new(5000, step_size);

    // motor.manual_move(Direction::LEFT, 1000.0f32).expect("should move");
    let available_progs = vec!["./example.gcode", "./unknown.gcode", "./notThere.gcode"];
    let mut selected_program = "";
    let mut program_select_cursor = 0usize;
    let mut current_mode: Mode = Mode::MANUAL;
    let mut input_reduce: u8 = 0;
    let mut prog: Option<program::Program> = None;

    let mut last = (0.0f64, 0.0f64, 0.0f64);

    'running: loop {
        thread::sleep(Duration::new(0, 100_000_000));
        if current_mode == Mode::MANUAL {
            // controller just every 10th tick
            input_reduce += 1;
            if input_reduce < 10 {
                continue 'running;
            }
            input_reduce = 0;

            let x = motor_x.lock().unwrap().deref_mut().get_pos();
            let y = motor_y.lock().unwrap().deref_mut().get_pos();
            let z = motor_z.lock().unwrap().deref_mut().get_pos();
            if last.0 != x || last.1 != y || last.2 != z {
                println!("x {} y {} z {}", x, y, z,);
                last = (x, y, z);
            }

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
                        if value > 0.08f32 {
                            motor_y
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::LEFT, (value - 0.08) * 5000.0f32)
                                .unwrap();
                        } else if value < -0.08f32 {
                            motor_y
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::RIGHT, (value + 0.08) * -5000.0f32)
                                .unwrap();
                        } else {
                            motor_y
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .cancel_task(&CommandOwner::MANUAL)
                                .unwrap();
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                        if value > 0.08f32 {
                            motor_x
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::LEFT, (value - 0.08) * 5000.0f32)
                                .unwrap();
                        } else if value < -0.08f32 {
                            motor_x
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::RIGHT, (value + 0.08) * -5000.0f32)
                                .unwrap();
                        } else {
                            motor_x
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .cancel_task(&CommandOwner::MANUAL)
                                .unwrap();
                        }
                    }
                    EventType::AxisChanged(Axis::RightStickY, value, _) => {
                        if value > 0.08f32 {
                            println!("left {}", value);
                            motor_z
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::LEFT, (value - 0.08) * 5000.0f32)
                                .unwrap();
                        } else if value < -0.08f32 {
                            println!("right {}", value);
                            motor_z
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .manual_move(Direction::RIGHT, (value + 0.08) * -5000.0f32)
                                .unwrap();
                        } else {
                            motor_z
                                .lock()
                                .unwrap()
                                .deref_mut()
                                .cancel_task(&CommandOwner::MANUAL)
                                .unwrap();
                        }
                    }
                    EventType::ButtonPressed(Button::North, _) => {
                        println!("reset");
                        motor_x.lock().unwrap().deref_mut().reset();
                        motor_y.lock().unwrap().deref_mut().reset();
                        motor_z.lock().unwrap().deref_mut().reset();
                    }
                    EventType::ButtonPressed(Button::DPadUp, _) => {
                        if program_select_cursor <= 0 {
                            program_select_cursor = available_progs.len() - 1;
                        } else {
                            program_select_cursor -= 1;
                        }
                        println!(
                            "select {} {}",
                            program_select_cursor,
                            available_progs.get(program_select_cursor).unwrap()
                        );
                    }
                    EventType::ButtonPressed(Button::DPadDown, _) => {
                        program_select_cursor += 1;

                        if program_select_cursor >= available_progs.len() {
                            program_select_cursor = 0;
                        }
                        println!(
                            "select {} {}",
                            program_select_cursor,
                            available_progs.get(program_select_cursor).unwrap()
                        );
                    }
                    EventType::ButtonPressed(Button::South, _) => {
                        selected_program = available_progs.get(program_select_cursor).unwrap();
                        println!("select {}", selected_program);
                    }
                    // add cross to select a programm
                    _ => {}
                }
            }
        } else {
            if let Some(p) = prog.as_mut() {
                'progLoop: for next_movement in p {
                    while let Some(Event { event, .. }) = gilrs.next_event() {
                        if let EventType::ButtonReleased(Button::Select, _) = event {
                            break 'progLoop;
                        }
                    }
                    match next_movement {
                        Some(movement) => {
                            if let Some(x) = movement.x {
                                motor_x
                                    .lock()
                                    .unwrap()
                                    .deref_mut()
                                    .exec_task(x)
                                    .expect("fix me in proc");
                            }
                            if let Some(y) = movement.y {
                                motor_y
                                    .lock()
                                    .unwrap()
                                    .deref_mut()
                                    .exec_task(y)
                                    .expect("fix me in proc");
                            }
                            if let Some(z) = movement.z {
                                motor_z
                                    .lock()
                                    .unwrap()
                                    .deref_mut()
                                    .exec_task(z)
                                    .expect("fix me in proc");
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}
