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

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::ThreadPool;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::{fs, thread, time::Duration};

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
    ui_data_sender: Sender<WsMessages>,
    ui_cmd_receiver: Receiver<WsCommands>,

    pub program_select_cursor: i32,
    pub input_reduce: u32,
    pub last_control: Location<f64>,
    pub display_counter: u32,
}

impl App {
    fn start() {
        let pool = ThreadPool::new().expect("Failed to build pool");
        let settings = Settings::from_file("./settings.yaml");

        let gilrs = Gilrs::new()
            .map_err(|_| "gamepad not valid")
            .expect("controller is missing");

        if !App::gamepad_connected(&gilrs) {
            panic!("no gamepad connected");
        }

        // init UI connection channel
        let (ui_data_sender, ui_data_receiver) = unbounded::<WsMessages>();
        let (ui_cmd_sender, ui_cmd_receiver) = unbounded::<WsCommands>();

        // return tuple with app and ui channel
        let mut app = App {
            available_progs: App::read_available_progs(&settings.input_dir),
            cnc: App::create_cnc_from_settings(&settings),
            pool,
            settings,
            gilrs,
            in_opp: false,
            current_mode: Mode::Manual,
            prog: None,
            calibrated: false,
            selected_program: None,
            ui_data_sender,
            ui_cmd_receiver,
            program_select_cursor: 0,
            input_reduce: 0,
            last_control: Location::default(),
            display_counter: 0,
        };
        app.run(ui_data_receiver, ui_cmd_sender);
    }
    fn create_cnc_from_settings(settings: &Settings) -> MotorController {
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

        // create cnc MotorController
        MotorController::new(motor_x, motor_y, motor_z, z_calibrate)
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
                    .filter(|name| {
                        name.ends_with(".gcode") || name.ends_with(".ngc") || name.ends_with(".nc")
                    })
            })
            .collect::<Vec<String>>()
    }
    pub fn update_available_progs(
        input_dir: &Vec<String>,
        available_progs: &Vec<String>,
    ) -> (bool, Vec<String>) {
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
                        Ok(RawEvent {
                            path: Some(path),
                            op: Ok(notify::op::CLOSE_WRITE),
                            ..
                        })
                        | Ok(RawEvent {
                            path: Some(path),
                            op: Ok(notify::op::REMOVE),
                            ..
                        }) => {
                            println!("{:?}", path);
                            publish_update = true;
                        }
                        _ => {
                            thread::sleep(Duration::new(1, 0));
                            if publish_update {
                                let (changed, ap) =
                                    App::update_available_progs(&path_vec, &known_progs);
                                if changed {
                                    known_progs = ap.clone();
                                    send_new_progs.send(ap).unwrap();
                                }
                                publish_update = false;
                            }
                        }
                    }
                }
                for path in path_vec.iter() {
                    watcher.unwatch(path).unwrap();
                }
            }
        });
        (send_path_changed, receiver_new_progs)
    }
    fn run(&mut self, data_receiver: Receiver<WsMessages>, cmd_sender: Sender<WsCommands>) {
        let (_update_path, new_progs) = self.start_file_watcher();

        // create Http-Server for the UI
        let pos_msg = WsPositionMessage::new(0.0f64, 0.0f64, 0.0f64);
        let status_msg = self.get_status_msg();
        self.pool.spawn_ok(async {
            ui_main(cmd_sender, data_receiver, pos_msg, status_msg).expect("could not start WS-server");
        });

        // initial output
        println!("rusty cnc controller started\n access the UI with http://localhost:1706");
        if self.settings.show_console_output {
            println!("Found programs in you input_path:");
            for (i, p) in self.available_progs.iter().enumerate() {
                println!("{}: {}", i, p);
            }
        }

        let mut last = Location::default();
        'running: loop {
            thread::sleep(Duration::new(0, 5_000_000));

            self.display_counter += 1;
            if self.display_counter >= self.settings.console_pos_update_reduce {
                let pos = self.cnc.get_pos();
                if last != pos {
                    self.send_pos_msg(&pos);
                    if self.settings.show_console_output {
                        println!("  {{ x: {}, y: {}, z: {} }},", pos.x, pos.y, pos.z);
                    }
                    last = pos;
                }
                self.display_counter = 0;
            }

            let ok = match self.current_mode {
                Mode::Manual => {
                    if let Ok(p) = new_progs.try_recv() {
                        println!("new progs {:?}", p);
                    }
                    self.manual_mode()
                },
                Mode::Program => self.program_mode(),
                Mode::Calibrate => self.calibrate_mode(),
            };
            if !ok {
                println!("terminate program");
                break 'running;
            };
        }
    }
}

impl App {
    pub fn get_status_msg(&self) -> WsStatusMessage {
        WsStatusMessage::new(
            self.current_mode.clone(),
            self.settings.dev_mode.clone(),
            self.in_opp.clone(),
            self.selected_program.clone(),
            self.calibrated.clone(),
        )
    }
    pub fn send_status_msg(&self) {
        self.ui_data_sender
            .send(WsMessages::Status(self.get_status_msg()))
            .unwrap();
    }

    pub fn get_pos_msg(pos: &Location<f64>) -> WsPositionMessage {
        WsPositionMessage::new(pos.x, pos.y, pos.z)
    }
    pub fn send_pos_msg(&self, pos: &Location<f64>) {
        self.ui_data_sender
            .send(WsMessages::Position(App::get_pos_msg(pos)))
            .unwrap();
    }
}

impl App {
    fn manual_mode(&mut self) -> bool {
        // controller just every 10th tick
        self.input_reduce += 1;
        if self.input_reduce < self.settings.input_update_reduce {
            return true;
        }
        self.input_reduce = 0;

        let mut control = self.last_control.clone();
        // map GamePad events to update the manual program or start a program
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::ButtonReleased(Button::Select, _) => {
                    // remove later to avoid killing the machine by mistake
                    return false;
                }
                EventType::ButtonReleased(Button::Start, _)
                | EventType::ButtonReleased(Button::Mode, _) => {
                    if !self.calibrated {
                        println!("Warning: start program without calibration")
                    }
                    if let Some(ref sel_prog) = self.selected_program {
                        if let Ok(load_prog) =
                            Program::new(sel_prog, 5.0, 50.0, 1.0, self.cnc.get_pos(), false)
                        {
                            self.prog = Some(load_prog);
                            self.current_mode = Mode::Program;

                            self.send_status_msg();
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
                            if self.program_select_cursor <= 0 {
                                self.program_select_cursor =
                                    self.available_progs.len() as i32 - 1;
                            } else {
                                self.program_select_cursor -= 1;
                            }
                        }
                        Button::DPadDown => {
                            self.program_select_cursor += 1;

                            if self.program_select_cursor >= self.available_progs.len() as i32 {
                                self.program_select_cursor = 0;
                            }
                        }
                        _ => (),
                    };
                    if self.settings.show_console_output {
                        for (i, p) in self.available_progs.iter().enumerate() {
                            println!("{}: {}", i, p);
                        }
                        println!(
                            "select {} {}",
                            self.program_select_cursor,
                            self.available_progs
                                .get(
                                    self.program_select_cursor
                                        .min(self.available_progs.len() as i32)
                                        .max(0)
                                        as usize
                                )
                                .unwrap()
                        );
                    }
                }
                EventType::ButtonPressed(Button::South, _) => {
                    self.selected_program = if let Some(sel) = self.available_progs.get(
                        self.program_select_cursor
                            .min(self.available_progs.len() as i32)
                            .max(0) as usize,
                    ) {
                        Some(sel.to_owned())
                    } else {
                        None
                    };
                    println!("select {:?}", self.selected_program);
                }
                EventType::ButtonPressed(Button::North, _) => {
                    let pos = self.cnc.get_pos();
                    if pos.x == 0.0 && pos.y == 0.0 {
                        println!("reset all (x, y, z)");
                        self.cnc.reset();
                    } else {
                        println!("reset only plane move (x, y)\nReset again without moving to reset the z axis as well");
                        self.cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                    }
                }
                EventType::ButtonPressed(Button::West, _) => {
                    println!("calibrate");
                    self.cnc.calibrate(
                        CalibrateType::None,
                        CalibrateType::None,
                        CalibrateType::ContactPin,
                    );
                    self.current_mode = Mode::Calibrate;
                }
                _ => {}
            }
        }
        if self.last_control != control {
            self.cnc.manual_move(control.x, control.y, control.z, 20.0);
            self.last_control = control.clone();
        }

        true
    }
    fn program_mode(&mut self) -> bool{
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.current_mode = Mode::Manual;
                if self.cnc.cancel_task().is_err() {
                    panic!("cancle did not work!");
                };
            }
        }
        if let Some(p) = self.prog.as_mut() {
            for next_instruction in p {
                match next_instruction {
                    NextInstruction::Movement(next_movement) => {
                        //println!("Movement: {:?}", next_movement);
                        self.cnc.query_task(next_movement);
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
            match (self.cnc.get_state(), self.in_opp) {
                (MachineState::Idle, true) => {
                    self.current_mode = Mode::Manual;
                    self.in_opp = false;
                }
                (MachineState::ProgramTask, false) => {
                    self.calibrated = false;
                    self.in_opp = true;
                }
                _ => (),
            }
        }

        true
    }
    fn calibrate_mode(&mut self) -> bool {
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.current_mode = Mode::Manual;
                if self.cnc.cancel_task().is_err() {
                    panic!("cancle did not work!");
                };
            }
        }
        match (self.cnc.get_state(), self.in_opp) {
            (MachineState::Idle, true) => {
                let calibrate_hight = Location {
                    x: 0.0f64,
                    y: 0.0f64,
                    z: 20.0f64,
                };
                self.cnc.set_pos(calibrate_hight);
                self.current_mode = Mode::Manual;
                self.calibrated = true;
                self.in_opp = false;
            }
            (MachineState::Calibrate, false) => {
                self.in_opp = true;
            }
            _ => (),
        }

        true
    }
}

fn main() {
    App::start();
}
