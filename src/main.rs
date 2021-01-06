mod motor;
mod program;
mod settings;
mod switch;
mod types;
mod ui;

use crate::motor::{CalibrateType, MockMotor, Motor, MotorController, StepMotor};
use crate::program::{NextInstruction, Program};
use crate::settings::Settings;
use crate::switch::Switch;
use crate::types::{Location, MachineState};
use crate::ui::{
    types::{Mode, WsCommands, WsMessages, WsPositionMessage, WsStatusMessage},
    ui_main,
};

use crossbeam_channel::unbounded;
use futures::executor::ThreadPool;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use notify::{Watcher, raw_watcher, RawEvent, RecursiveMode};
use std::{fs, thread, time::Duration};
use std::sync::mpsc;

struct App {
    pub available_progs: Vec<String>,
    pub pool: ThreadPool,
    pub settings: Settings,
    pub gilrs: Gilrs,
    pub in_opp: bool,
    pub cnc: MotorController,
    pub current_mode: Mode,
    pub prog: Option<program::Program>,
    pub calibrated: bool,
    pub selected_program: Option<String>,
}

impl App {
    fn init() -> App {
        let pool = ThreadPool::new().expect("Failed to build pool");
        let settings = Settings::from_file("./settings.yaml");

        let gilrs = Gilrs::new()
            .map_err(|_| "gamepad not valid")
            .expect("controller is missing");

        if !App::gamepad_connected(&gilrs) {
            panic!("no gamepad connected");
        }

        let (z_calibrate, motor_x, motor_y, motor_z) = if settings.dev_mode {
            (
                None,
                Motor::new(
                    "x".to_string(),
                    settings.motor_x.max_step_speed,
                    Box::new(MockMotor::new(settings.motor_x.step_size)),
                ),
                Motor::new(
                    "y".to_string(),
                    settings.motor_y.max_step_speed,
                    Box::new(MockMotor::new(settings.motor_y.step_size)),
                ),
                Motor::new(
                    "z".to_string(),
                    settings.motor_z.max_step_speed,
                    Box::new(MockMotor::new(settings.motor_z.step_size)),
                ),
            )
        } else {
            (
                if let Some(pin) = settings.calibrate_z_gpio {
                    Some(Switch::new(pin, false))
                } else {
                    None
                },
                Motor::new(
                    "x".to_string(),
                    settings.motor_x.max_step_speed,
                    Box::new(StepMotor::new(
                        settings.motor_x.pull_gpio,      //18,
                        settings.motor_x.dir_gpio,       //27,
                        settings.motor_x.ena_gpio,       //None,
                        settings.motor_x.end_left_gpio,  //Some(21),
                        settings.motor_x.end_right_gpio, //Some(20),
                        settings.motor_x.max_step_speed, //speed,
                        settings.motor_x.step_size,      //0.004f64,
                    )),
                ),
                Motor::new(
                    "y".to_string(),
                    settings.motor_y.max_step_speed,
                    Box::new(StepMotor::new(
                        settings.motor_y.pull_gpio,      // 22,
                        settings.motor_y.dir_gpio,       // 23,
                        settings.motor_y.ena_gpio,       // None,
                        settings.motor_y.end_left_gpio,  // Some(19),
                        settings.motor_y.end_right_gpio, // Some(26),
                        settings.motor_y.max_step_speed, // speed,
                        settings.motor_y.step_size,      // 0.004f64,
                    )),
                ),
                Motor::new(
                    "z".to_string(),
                    settings.motor_z.max_step_speed,
                    Box::new(StepMotor::new(
                        settings.motor_z.pull_gpio,      //25,
                        settings.motor_z.dir_gpio,       //24,
                        settings.motor_z.ena_gpio,       //None,
                        settings.motor_z.end_left_gpio,  //Some(5),
                        settings.motor_z.end_right_gpio, //Some(6),
                        settings.motor_z.max_step_speed, //speed,
                        settings.motor_z.step_size,      //0.004f64,
                    )),
                ),
            )
        };

        let cnc = MotorController::new(motor_x, motor_y, motor_z, z_calibrate);

        App {
            available_progs: App::read_available_progs(&settings.input_dir),
            pool,
            settings,
            gilrs,
            in_opp: false,
            cnc,
            current_mode: Mode::Manual,
            prog: None,
            calibrated: false,
            selected_program: None,
        }
    }
    fn gamepad_connected(gilrs: &Gilrs) -> bool {
        let mut gamepad_found = false;
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
            gamepad_found = true;
        }
        gamepad_found
    }
    fn read_available_progs(input_dir: &Vec<String>) -> Vec<String> {
        input_dir
            .iter()
            .flat_map(|path| {
                fs::read_dir(path)
                    .unwrap()
                    .map(|res| res.expect("ok").path().to_str().unwrap().to_owned())
                    .filter(|name| name.ends_with(".gcode") || name.ends_with(".ngc") || name.ends_with(".nc"))
            })
            .collect::<Vec<String>>()
    }
    pub fn update_available_progs(input_dir: &Vec<String>, available_progs: &Vec<String>) -> (bool, Vec<String>) {
        let new_content = App::read_available_progs(input_dir);

        if new_content.len() != available_progs.len() {
            (true, new_content)
        } else {
            // check if both arrays contain the same content
            let match_count = new_content
                .iter()
                .zip(available_progs.clone())
                .filter(|(a, b)| *a == b)
                .count();
            (match_count != available_progs.len(), new_content)
        }
    }

    fn start_file_watcher(&mut self) -> (mpsc::Sender<Vec<String>>, mpsc::Receiver<Vec<String>>) {
        let (send_path_changed, receiver_path_changed) = mpsc::channel::<Vec<String>>();
        let (send_new_progs, receiver_new_progs) = mpsc::channel::<Vec<String>>();
        let mut path_vec = self.settings.input_dir.clone();
        thread::spawn(move || {
            let (tx_fs_changed, rx_fs_changed) = mpsc::channel();
            let mut watcher = raw_watcher(tx_fs_changed.clone()).unwrap();
            let mut publish_update = false;
            let mut known_progs = vec![];
            loop {
                for path in path_vec.iter() {
                    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
                }

                'watch: loop {
                    if let Ok(p) = receiver_path_changed.try_recv() {
                        path_vec = p;
                        break 'watch;
                    };
                    match rx_fs_changed.try_recv() {
                        Ok(RawEvent{path: Some(path), op: Ok(notify::op::CLOSE_WRITE), ..}) |
                        Ok(RawEvent{path: Some(path), op: Ok(notify::op::REMOVE), ..}) => {
                            println!("{:?}", path);
                            publish_update = true;
                        },
                        _ => {
                            thread::sleep(Duration::new(1, 0));
                            if publish_update {
                                let (changed, ap) = App::update_available_progs(&path_vec, &known_progs);
                                if changed {
                                    known_progs = ap.clone();
                                    send_new_progs.send(ap);
                                }
                                publish_update = false;
                            }
                        },
                     }
                };
                for path in path_vec.iter() {
                    watcher.unwatch(path).unwrap();
                }
            };
        });
        (send_path_changed, receiver_new_progs)
    }

    fn run(&mut self) {
        let (update_path, new_progs) = self.start_file_watcher();
        thread::spawn(move || {
            loop {
                if let Ok(p) = new_progs.recv() {
                    println!("new progs {:?}", p);
                }
            }
        });
    }
}

fn main() {
    let mut app: App = App::init();
    app.run();

    for (i, p) in app.available_progs.iter().enumerate() {
        println!("{}: {}", i, p);
    }

    let mut program_select_cursor: i32 = 0;
    let mut input_reduce: u8 = 0;

    let mut last = Location::default();
    let mut control = Location::default();
    let mut last_control = control.clone();
    let mut display_counter = 0;

    let (data_sender, data_receiver) = unbounded::<WsMessages>();
    let (cmd_sender, _cmd_receiver) = unbounded::<WsCommands>();
    let pos_msg = WsPositionMessage::new(0.0f64, 0.0f64, 0.0f64);
    let status_msg = WsStatusMessage::new(
        app.current_mode.clone(),
        app.settings.dev_mode.clone(),
        200,
        app.in_opp.clone(),
        app.selected_program.clone(),
        app.calibrated.clone(),
    );
    app.pool.spawn_ok(async {
        ui_main(cmd_sender, data_receiver, pos_msg, status_msg).expect("could not start WS-server");
    });

    'running: loop {
        thread::sleep(Duration::new(0, 5_000_000));
        display_counter += 1;
        if display_counter >= 50 {
            let pos = app.cnc.get_pos();
            if last != pos {
                data_sender
                    .send(WsMessages::Position(WsPositionMessage {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                    }))
                    .unwrap();
                println!("  {{ x: {}, y: {}, z: {} }},", pos.x, pos.y, pos.z);
                last = pos;
            }
            display_counter = 0;
        }
        match app.current_mode {
            Mode::Manual => {
                // controller just every 10th tick
                input_reduce += 1;
                if input_reduce < 10 {
                    continue 'running;
                }
                input_reduce = 0;

                // map GamePad events to update the manual program or start a program
                while let Some(Event { event, .. }) = app.gilrs.next_event() {
                    match event {
                        EventType::ButtonReleased(Button::Select, _) => {
                            // remove later to avoid killing the machine by mistake
                            break 'running;
                        }
                        EventType::ButtonReleased(Button::Start, _)
                        | EventType::ButtonReleased(Button::Mode, _) => {
                            if !app.calibrated {
                                println!("Warning: start program without calibration")
                            }
                            if let Some(ref sel_prog) = app.selected_program {
                                if let Ok(load_prog) =
                                    Program::new(sel_prog, 5.0, 50.0, 1.0, app.cnc.get_pos(), false)
                                {
                                    app.prog = Some(load_prog);
                                    app.current_mode = Mode::Program;

                                    data_sender
                                        .send(WsMessages::Status(WsStatusMessage::new(
                                            app.current_mode.clone(),
                                            app.settings.dev_mode.clone(),
                                            200,
                                            app.in_opp.clone(),
                                            app.selected_program.clone(),
                                            app.calibrated.clone(),
                                        )))
                                        .unwrap();
                                } else {
                                    println!("program is not able to load")
                                }
                            } else {
                                println!("No Program selected")
                            }
                        }
                        EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                            if value > 0.15 {
                                control.x = (value as f64 - 0.15) / 8.5 * -10.0;
                            } else if value < -0.15 {
                                control.x = (value as f64 + 0.15) / 8.5 * -10.0;
                            } else {
                                control.x = 0.0;
                            }
                        }
                        EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                            if value > 0.15 {
                                control.y = (value as f64 - 0.15) / 8.5 * 10.0;
                            } else if value < -0.15 {
                                control.y = (value as f64 + 0.15) / 8.5 * 10.0;
                            } else {
                                control.y = 0.0;
                            }
                        }
                        EventType::AxisChanged(Axis::RightStickY, value, _) => {
                            if value > 0.15 {
                                control.z = (value as f64 - 0.15) / 8.5 * 10.0;
                            } else if value < -0.15 {
                                control.z = (value as f64 + 0.15) / 8.5 * 10.0;
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
                                        program_select_cursor =
                                            app.available_progs.len() as i32 - 1;
                                    } else {
                                        program_select_cursor -= 1;
                                    }
                                }
                                Button::DPadDown => {
                                    program_select_cursor += 1;

                                    if program_select_cursor >= app.available_progs.len() as i32 {
                                        program_select_cursor = 0;
                                    }
                                }
                                _ => (),
                            };
                            for (i, p) in app.available_progs.iter().enumerate() {
                                println!("{}: {}", i, p);
                            }
                            println!(
                                "select {} {}",
                                program_select_cursor,
                                app.available_progs
                                    .get(
                                        program_select_cursor
                                            .min(app.available_progs.len() as i32)
                                            .max(0)
                                            as usize
                                    )
                                    .unwrap()
                            );
                        }
                        EventType::ButtonPressed(Button::South, _) => {
                            app.selected_program = if let Some(sel) = app.available_progs.get(
                                program_select_cursor
                                    .min(app.available_progs.len() as i32)
                                    .max(0) as usize,
                            ) {
                                Some(sel.to_owned())
                            } else {
                                None
                            };
                            println!("select {:?}", app.selected_program);
                        }
                        EventType::ButtonPressed(Button::North, _) => {
                            let pos = app.cnc.get_pos();
                            if pos.x == 0.0 && pos.y == 0.0 {
                                println!("reset all (x, y, z)");
                                app.cnc.reset();
                            } else {
                                println!("reset only plane move (x, y)\nReset again without moving to reset the z axis as well");
                                app.cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                            }
                        }
                        EventType::ButtonPressed(Button::West, _) => {
                            println!("calibrate");
                            app.cnc.calibrate(
                                CalibrateType::None,
                                CalibrateType::None,
                                CalibrateType::ContactPin,
                            );
                            app.current_mode = Mode::Calibrate;
                        }
                        _ => {}
                    }
                }
                if last_control != control {
                    app.cnc.manual_move(control.x, control.y, control.z, 20.0);
                    last_control = control.clone();
                }
            }
            Mode::Program => {
                while let Some(Event { event, .. }) = app.gilrs.next_event() {
                    if let EventType::ButtonReleased(Button::Select, _) = event {
                        app.current_mode = Mode::Manual;
                        if app.cnc.cancel_task().is_err() {
                            panic!("cancle did not work!");
                        };
                    }
                }
                if let Some(p) = app.prog.as_mut() {
                    for next_instruction in p {
                        match next_instruction {
                            NextInstruction::Movement(next_movement) => {
                                //println!("Movement: {:?}", next_movement);
                                app.cnc.query_task(next_movement);
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
                    match (app.cnc.get_state(), app.in_opp) {
                        (MachineState::Idle, true) => {
                            app.current_mode = Mode::Manual;
                            app.in_opp = false;
                        }
                        (MachineState::ProgramTask, false) => {
                            app.calibrated = false;
                            app.in_opp = true;
                        }
                        _ => (),
                    }
                }
            }
            Mode::Calibrate => {
                while let Some(Event { event, .. }) = app.gilrs.next_event() {
                    if let EventType::ButtonReleased(Button::Select, _) = event {
                        app.current_mode = Mode::Manual;
                        if app.cnc.cancel_task().is_err() {
                            panic!("cancle did not work!");
                        };
                    }
                }
                match (app.cnc.get_state(), app.in_opp) {
                    (MachineState::Idle, true) => {
                        let calibrate_hight = Location {
                            x: 0.0f64,
                            y: 0.0f64,
                            z: 20.0f64,
                        };
                        app.cnc.set_pos(calibrate_hight);
                        app.current_mode = Mode::Manual;
                        app.calibrated = true;
                        app.in_opp = false;
                    }
                    (MachineState::Calibrate, false) => {
                        app.in_opp = true;
                    }
                    _ => (),
                }
            }
        }
    }
}
