mod motor;
mod program;
mod switch;
mod types;

use crate::motor::{CalibrateType, MockMotor, Motor, MotorController, StepMotor};
use crate::program::{NextInstruction, Program};
use crate::switch::Switch;
use crate::types::{Location, MachineState};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use std::{env, fs, thread, time::Duration};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    Manual,
    Program,
    CalibrateZ,
}

struct Settings {
    dev_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { dev_mode: false }
    }
}
impl From<env::Args> for Settings {
    fn from(args: env::Args) -> Settings {
        let mut s = Settings::default();
        for arg in args {
            if arg == String::from("dev_mode") {
                s.dev_mode = true;
            }
        }

        s
    }
}

fn main() {
    let settings = Settings::from(env::args());

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

    let speed = 1_000; // 2mm per Round / 1.8° per step (0.02 Um per step) => 0.01 mm per step

    let (z_calibrate, motor_x, motor_y, motor_z) = if settings.dev_mode {
        (
            None,
            Motor::new("x".to_string(), speed, Box::new(MockMotor::new(0.0040f64))),
            Motor::new("y".to_string(), speed, Box::new(MockMotor::new(0.0042f64))),
            Motor::new("z".to_string(), speed, Box::new(MockMotor::new(0.0042f64))),
        )
    } else {
        (
            Some(Switch::new(16, false)),
            Motor::new(
                "x".to_string(),
                speed,
                Box::new(StepMotor::new(
                    18,
                    27,
                    None,
                    Some(21),
                    Some(20),
                    speed,
                    0.0040f64,
                )),
            ),
            Motor::new(
                "y".to_string(),
                speed,
                Box::new(StepMotor::new(
                    22,
                    23,
                    None,
                    Some(19),
                    Some(26),
                    speed,
                    0.0042f64,
                )),
            ),
            Motor::new(
                "z".to_string(),
                speed,
                Box::new(StepMotor::new(
                    25,
                    24,
                    None,
                    Some(5),
                    Some(6),
                    speed,
                    0.0042f64,
                )),
            ),
        )
    };

    let mut cnc = MotorController::new(motor_x, motor_y, motor_z, z_calibrate);
    let mut in_opp = false;

    let available_progs = fs::read_dir(".")
        .unwrap()
        .map(|res| res.expect("ok").path().to_str().unwrap().to_owned())
        .filter(|name| name.ends_with(".gcode"))
        .collect::<Vec<String>>();
    for (i, p) in available_progs.iter().enumerate() {
        println!("{}: {}", i, p);
    }

    let mut selected_program = available_progs.get(0);
    let mut program_select_cursor = 0;
    let mut current_mode: Mode = Mode::Manual;
    let mut input_reduce: u8 = 0;
    let mut prog: Option<program::Program> = None;

    let mut last = Location::default();
    let mut control = Location::default();
    let mut last_control = control.clone();
    let mut display_counter = 0;
    let mut calibrated = false;

    'running: loop {
        thread::sleep(Duration::new(0, 5_000_000));
        display_counter += 1;
        if display_counter >= 100 {
            let pos = cnc.get_pos();
            if last != pos {
                println!("  {{ x: {}, y: {}, z: {} }},", pos.x, pos.y, pos.z);
                last = pos;
            }
            display_counter = 0;
        }
        match current_mode {
            Mode::Manual => {
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
                            if !calibrated {
                                println!("Warning: start program without calibration")
                            }
                            if let Some(sel_prog) = selected_program {
                                if let Ok(load_prog) =
                                    Program::new(sel_prog, 5.0, 10.0, 1.0, cnc.get_pos(), false)
                                {
                                    prog = Some(load_prog);
                                    current_mode = Mode::Program;
                                } else {
                                    println!("program is not able to load")
                                }
                            } else {
                                println!("No Program selected")
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
                        // add cross to select a program
                        EventType::ButtonPressed(dir @ Button::DPadDown, _)
                        | EventType::ButtonPressed(dir @ Button::DPadUp, _) => {
                            match dir {
                                Button::DPadUp => {
                                    if program_select_cursor <= 0 {
                                        program_select_cursor = available_progs.len() - 1;
                                    } else {
                                        program_select_cursor -= 1;
                                    }
                                }
                                Button::DPadDown => {
                                    program_select_cursor += 1;

                                    if program_select_cursor >= available_progs.len() {
                                        program_select_cursor = 0;
                                    }
                                }
                                _ => (),
                            };
                            println!(
                                "select {} {}",
                                program_select_cursor,
                                available_progs.get(program_select_cursor).unwrap()
                            );
                        }
                        EventType::ButtonPressed(Button::South, _) => {
                            selected_program = available_progs.get(program_select_cursor);
                            println!("select {:?}", selected_program);
                        }
                        EventType::ButtonPressed(Button::North, _) => {
                            let pos = cnc.get_pos();
                            if pos.x == 0.0 && pos.y == 0.0 {
                                println!("reset all (x, y, z)");
                                cnc.reset();
                            } else {
                                println!("reset only plane move (x, y)\nReset again without moving to reset the z axis as well");
                                cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                            }
                        }
                        EventType::ButtonPressed(Button::West, _) => {
                            println!("calibrate");
                            cnc.calibrate(
                                CalibrateType::None,
                                CalibrateType::None,
                                CalibrateType::ContactPin,
                            );
                            current_mode = Mode::CalibrateZ;
                        }
                        _ => {}
                    }
                }
                if last_control != control {
                    println!("set manual move");
                    cnc.manual_move(control.x, control.y, control.z, 5.0);
                    last_control = control.clone();
                }
            }
            Mode::Program => {
                while let Some(Event { event, .. }) = gilrs.next_event() {
                    if let EventType::ButtonReleased(Button::Select, _) = event {
                        current_mode = Mode::Manual;
                        cnc.manual_move(0.0, 0.0, 0.0, 1.0);
                    }
                }
                if let Some(p) = prog.as_mut() {
                    for next_instruction in p {
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
                    match (cnc.get_state(), in_opp) {
                        (MachineState::Idle, true) => {
                            current_mode = Mode::Manual;
                            in_opp = false;
                        }
                        (MachineState::ProgramTask, false) => {
                            calibrated = false;
                            in_opp = true;
                        }
                        _ => (),
                    }
                }
            }
            Mode::CalibrateZ => {
                while let Some(Event { event, .. }) = gilrs.next_event() {
                    if let EventType::ButtonReleased(Button::Select, _) = event {
                        current_mode = Mode::Manual;
                        cnc.manual_move(0.0, 0.0, 0.0, 1.0);
                    }
                }
                match (cnc.get_state(), in_opp) {
                    (MachineState::Idle, true) => {
                        let calibrate_hight = Location {
                            x: 0.0f64,
                            y: 0.0f64,
                            z: 20.0f64,
                        };
                        cnc.set_pos(calibrate_hight);
                        current_mode = Mode::Manual;
                        calibrated = true;
                        in_opp = false;
                    }
                    (MachineState::Calibrate, false) => {
                        in_opp = true;
                    }
                    _ => (),
                }
            }
        }
    }
}
