mod motor;
mod program;
mod switch;
mod types;

use crate::motor::{MockMotor, Motor, MotorController};
use crate::program::{NextInstruction, Program};
use crate::types::{Location, MachineState};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use std::{thread, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    MANUAL,
    PROGRAM,
}

fn main() {
    let mut gilrs = Gilrs::new()
        .map_err(|_| "gamepad not valid")
        .expect("controler is missing");
    let mut gamepad_found = false;
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
        gamepad_found = true;
    }
    if !gamepad_found {
        panic!("no gamepad connected");
    }

    let step_size = 0.01f64; // 2mm per Round / 1.8° per step (0.02 Um per step) => 0.01 mm per step
    let speed = 5_000; // 2mm per Round / 1.8° per step (0.02 Um per step) => 0.01 mm per step

    // let motor_x = StepMotor::new(17, 24, None, None, None, 5000, step_size);
    // let motor_y = StepMotor::new(18, 23, None, Some(26), None, 5000, step_size);
    // let motor_z = StepMotor::new(16, 25, None, None, None, 5000, step_size);

    let motor_x = Motor::new(
        "x".to_string(),
        step_size,
        speed,
        Box::new(MockMotor::new()),
    );
    let motor_y = Motor::new(
        "y".to_string(),
        step_size,
        speed,
        Box::new(MockMotor::new()),
    );
    let motor_z = Motor::new(
        "z".to_string(),
        step_size,
        speed,
        Box::new(MockMotor::new()),
    );

    let mut cnc = MotorController::new(motor_x, motor_y, motor_z);
    let mut in_prog = false;

    // motor.manual_move(Direction::LEFT, 1000.0f32).expect("should move");
    let available_progs = vec![
        "./example.gcode",
        "./unknown.gcode",
        "./notThere.gcode",
        "linearExample.gcode",
        "one.gcode",
        "longtest.gcode",
    ];
    let mut selected_program = "one.gcode";
    let mut program_select_cursor = 4;
    let mut current_mode: Mode = Mode::MANUAL;
    let mut input_reduce: u8 = 0;
    let mut prog: Option<program::Program> = None;

    let mut last = Location::default();
    let mut control = Location::default();
    let mut last_control = control.clone();
    let mut display_counter = 0;

    'running: loop {
        thread::sleep(Duration::new(0, 10_000_000));
        display_counter += 1;
        if display_counter >= 30 {
            let pos = cnc.get_pos();
            if last != pos {
                println!("x {} y {} z {}", pos.x, pos.y, pos.z);
                last = pos;
            }
            display_counter = 0;
        }
        if current_mode == Mode::MANUAL {
            // controller just every 10th tick
            input_reduce += 1;
            if input_reduce < 10 {
                continue 'running;
            }
            input_reduce = 0;

            // map GamePad events to update the manual program or start a programm
            while let Some(Event { event, .. }) = gilrs.next_event() {
                match event {
                    EventType::ButtonReleased(Button::Select, _) => {
                        // remove later to avoid killing the machine by mistake
                        break 'running;
                    }
                    EventType::ButtonReleased(Button::Start, _)
                    | EventType::ButtonReleased(Button::Mode, _) => {
                        if let Ok(load_prog) = Program::new(selected_program, 10.0) {
                            prog = Some(load_prog);
                            current_mode = Mode::PROGRAM;
                        } else {
                            println!("program is not able to load")
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                        if value > 0.1 {
                            control.x = (value as f64 - 0.1) / 9.0 * -10.0;
                        } else if value < -0.1 {
                            control.x = (value as f64 + 0.1) / 9.0 * -10.0;
                        } else {
                            control.x = 0.0;
                        }
                    }
                    EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                        if value > 0.1 {
                            control.y = (value as f64 - 0.1) / 9.0 * 10.0;
                        } else if value < -0.1 {
                            control.y = (value as f64 + 0.1) / 9.0 * 10.0;
                        } else {
                            control.y = 0.0;
                        }
                    }
                    EventType::AxisChanged(Axis::RightStickY, value, _) => {
                        if value > 0.1 {
                            control.z = (value as f64 - 0.1) / 9.0 * 10.0;
                        } else if value < -0.1 {
                            control.z = (value as f64 + 0.1) / 9.0 * 10.0;
                        } else {
                            control.z = 0.0;
                        }
                    }
                    EventType::ButtonPressed(Button::North, _) => {
                        println!("reset");
                        cnc.reset();
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
            if last_control != control {
                println!("set manual move");
                cnc.manual_move(control.x, control.y, control.z, 5.0);
                last_control = control.clone();
            }
        } else {
            if let Some(p) = prog.as_mut() {
                'progLoop: for next_instruction in p {
                    while let Some(Event { event, .. }) = gilrs.next_event() {
                        if let EventType::ButtonReleased(Button::Select, _) = event {
                            break 'progLoop;
                        }
                    }

                    match next_instruction {
                        NextInstruction::Movement(next_movement) => {
                            //println!("Movement: {:?}", next_movement);
                            cnc.query_task(next_movement);
                        }
                        NextInstruction::Miscellaneous(task) => {
                            println!("Miscellaneous {:?}", task);
                        }
                        NextInstruction::NotSupported(err) => {
                            println!("NotSupported {:?}", err);
                            //writeln!(err.to_owned());
                        }
                        NextInstruction::InternalInstruction(err) => {
                            println!("InternalInstruction {:?}", err);
                            //writeln!(err.to_owned());
                        }
                        _ => {}
                    };
                }
                match (cnc.get_state(), in_prog) {
                    (MachineState::Idle, true) => {
                        current_mode = Mode::MANUAL;
                        in_prog = false;
                    }
                    (MachineState::ProgrammTask, false) => {
                        in_prog = true;
                    }
                    _ => (),
                }
            }
        }
    }
}
