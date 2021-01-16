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
    types::{
        InfoLvl, Mode, ProgramInfo, WsAvailableProgramsMessage, WsCommandController,
        WsCommandProgram, WsCommandSettings, WsCommands, WsCommandsFrom, WsControllerMessage,
        WsInfoMessage, WsMessages, WsPositionMessage, WsReplyMessage, WsStatusMessage,
        WsCommandSettingsSetRuntimeSettings, WsCommandSettingsSetSystemSettings
    },
    ui_main,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::ThreadPool;
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
use std::sync::mpsc;
use std::{
    fs::{self, remove_file, File},
    io::prelude::*,
    path::Path,
    thread,
    time::Duration,
};
use uuid::Uuid;

const SETTINGS_PATH: &'static str = "./settings.yaml";

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
    ui_cmd_receiver: Receiver<WsCommandsFrom>,

    pub program_select_cursor: i32,
    pub input_reduce: u32,
    pub last_control: Location<f64>,
    pub freeze_x: bool,
    pub freeze_y: bool,
    pub slow_control: bool,
    pub display_counter: u32,
    pub steps_todo: i64,
    pub steps_done: i64,
}

impl App {
    fn start() {
        let pool = ThreadPool::new().expect("Failed to build pool");
        let settings = Settings::from_file(SETTINGS_PATH);

        let gilrs = Gilrs::new()
            .map_err(|_| "gamepad not valid")
            .expect("controller is missing");

        if !App::gamepad_connected(&gilrs) {
            panic!("no gamepad connected");
        }

        // init UI connection channel
        let (ui_data_sender, ui_data_receiver) = unbounded::<WsMessages>();
        let (ui_cmd_sender, ui_cmd_receiver) = unbounded::<WsCommandsFrom>();

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
            steps_todo: 0,
            steps_done: 0,
            freeze_x: false,
            freeze_y: false,
            slow_control: false,
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
                        println!("receive new Path {:?}", p);
                        for path in path_vec.iter() {
                            watcher.unwatch(path).unwrap();
                        }
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
                            thread::sleep(Duration::new(0, 250_000_000));
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
            }
        });
        (send_path_changed, receiver_new_progs)
    }
    fn run(&mut self, data_receiver: Receiver<WsMessages>, cmd_sender: Sender<WsCommandsFrom>) {
        let (update_path, new_progs) = self.start_file_watcher();

        // create Http-Server for the UI
        let pos_msg = WsPositionMessage::new(0.0f64, 0.0f64, 0.0f64);
        let status_msg = self.get_status_msg();
        let controller_msg = WsControllerMessage::new(&Location::default(), false, false, false);
        self.pool.spawn_ok(async {
            ui_main(
                cmd_sender,
                data_receiver,
                pos_msg,
                status_msg,
                controller_msg,
            )
            .expect("could not start WS-server");
        });

        // initial output
        println!("rusty cnc controller started\n access the UI with http://localhost:1506");
        if self.settings.show_console_output {
            println!("Found programs in you input_path:");
            for (i, p) in self.available_progs.iter().enumerate() {
                println!("{}: {}", i, p);
            }
        }

        let mut last = Location::default();
        'running: loop {
            thread::sleep(Duration::new(0, 5_000_000));

            // display position, or send it to the ws client
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

            // poll current mode of the cnc
            let ok = match self.current_mode {
                Mode::Program => self.program_mode(),
                Mode::Calibrate => self.calibrate_mode(),
                _ => self.manual_mode(),
            };
            if !ok {
                println!("terminate program");
                break 'running;
            }

            if let Ok(p) = new_progs.try_recv() {
                self.set_available_programs(p);
            }

            // handle incoming commands
            if let Ok(WsCommandsFrom(uuid, cmd)) = self.ui_cmd_receiver.try_recv() {
                match cmd {
                    WsCommands::Controller(WsCommandController::FreezeX { freeze }) => {
                        if self.freeze_x != freeze {
                            self.freeze_x = freeze;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Controller(WsCommandController::FreezeY { freeze }) => {
                        if self.freeze_y != freeze {
                            self.freeze_y = freeze;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Controller(WsCommandController::Slow { slow }) => {
                        if self.slow_control != slow {
                            self.slow_control = slow;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Get) => {
                        self.send_available_programs_msg(uuid)
                    }
                    WsCommands::Program(WsCommandProgram::Load { program_name }) => {
                        self.send_program_data_msg(uuid, program_name)
                    }
                    WsCommands::Program(WsCommandProgram::Start {
                        program_name,
                        invert_z,
                        scale,
                    }) => {
                        if self.start_program(&program_name, invert_z, scale) {
                            self.send_start_reply_message(uuid, program_name)
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Cancel) => {
                        self.cancel_program();
                        self.send_cancel_reply_message(uuid, true);
                    }
                    WsCommands::Program(WsCommandProgram::Save {
                        program_name,
                        program,
                    }) => {
                        if Path::new(&program_name).exists() {
                            match File::open(program_name.clone()) {
                                Err(why) => {
                                    self.info(format!("couldn't open {}: {}", program_name, why));
                                    self.send_save_reply_message(uuid, program_name, false);
                                }
                                Ok(mut file) => {
                                    match file.write_all(program.as_bytes()) {
                                        Err(why) => {
                                            self.info(format!(
                                                "couldn't write to {}: {}",
                                                program_name, why
                                            ));
                                            self.send_save_reply_message(uuid, program_name, false);
                                        }
                                        Ok(_) => {
                                            self.send_save_reply_message(uuid, program_name, true)
                                        }
                                    };
                                }
                            }
                        } else {
                            match File::create(&program_name) {
                                Err(why) => {
                                    self.info(format!(
                                        "couldn't write to {}: {}",
                                        program_name, why
                                    ));
                                    self.send_save_reply_message(uuid, program_name, true);
                                }
                                Ok(mut file) => self.send_save_reply_message(
                                    uuid,
                                    program_name,
                                    file.write_all(program.as_bytes()).is_ok(),
                                ),
                            }
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Delete { program_name }) => {
                        self.send_delete_reply_message(
                            uuid,
                            program_name.clone(),
                            remove_file(program_name).is_ok(),
                        );
                    }
                    WsCommands::Settings(WsCommandSettings::GetRuntime) => {
                        self.send_runtime_settings_reply_message(uuid);
                    }
                    WsCommands::Settings(WsCommandSettings::SetRuntime( settings )) => {
                        match self.set_runtime_settings(settings, &update_path) {
                            Ok(()) => self.send_runtime_settings_saved_reply_message(uuid, true),
                            Err(_) => self.send_runtime_settings_saved_reply_message(uuid, false),
                        };
                    }
                    WsCommands::Settings(WsCommandSettings::GetSystem) => {
                        self.send_system_settings_reply_message(uuid);
                    }
                    WsCommands::Settings(WsCommandSettings::SetSystem( settings )) => {
                        match self.set_system_settings(settings) {
                            Ok(()) => self.send_system_settings_saved_reply_message(uuid, true),
                            Err(_) => self.send_system_settings_saved_reply_message(uuid, false),
                        };
                    }
                    //_ => (),
                };
            }
        }
    }
}

impl App {
    pub fn set_prog_state(&mut self, todo: i64, done: i64) {
        if self.steps_todo != todo || self.steps_done != done {
            self.steps_todo = todo;
            self.steps_done = done;
            self.send_status_msg();
        }
    }
    pub fn set_current_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
        self.send_status_msg();
    }
    pub fn set_available_programs(&mut self, available_progs: Vec<String>) {
        self.available_progs = available_progs;
        self.send_available_program_msg();
    }
    pub fn set_selected_program(&mut self, selected_program: Option<String>) {
        self.selected_program = selected_program;
        self.send_status_msg();
    }
    pub fn set_runtime_settings(&mut self, settings: WsCommandSettingsSetRuntimeSettings, update_path: &mpsc::Sender<Vec<String>>) -> Result<(), String> {
        settings.input_dir.map(|v| {
            self.settings.input_dir = v.clone();
            update_path.send(v)
        });
        settings.input_update_reduce.map(|v| self.settings.input_update_reduce = v);
        settings.default_speed.map(|v| self.settings.default_speed = v);
        settings.rapid_speed.map(|v| self.settings.rapid_speed = v);
        settings.scale.map(|v| self.settings.scale = v);
        settings.invert_z.map(|v| self.settings.invert_z = v);
        settings.show_console_output.map(|v| self.settings.show_console_output = v);
        settings.console_pos_update_reduce.map(|v| self.settings.console_pos_update_reduce = v);

        self.settings.write_to_file(SETTINGS_PATH)
    }
    pub fn set_system_settings(&mut self, settings: WsCommandSettingsSetSystemSettings) -> Result<(), String> {
        self.settings.dev_mode = settings.dev_mode;
        self.settings.motor_x = settings.motor_x;
        self.settings.motor_y = settings.motor_y;
        self.settings.motor_z = settings.motor_z;
        self.settings.calibrate_z_gpio = settings.calibrate_z_gpio;

        self.settings.write_to_file(SETTINGS_PATH)
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
            self.steps_todo,
            self.steps_done,
        )
    }
    pub fn send_status_msg(&self) {
        self.ui_data_sender
            .send(WsMessages::Status(self.get_status_msg()))
            .unwrap();
    }
    pub fn send_available_program_msg(&self) {
        self.ui_data_sender
            .send(WsMessages::ProgsUpdate(WsAvailableProgramsMessage {
                progs: self
                    .available_progs
                    .iter()
                    .map(|name| name.to_owned())
                    .map(ProgramInfo::from_string)
                    .collect(),
                input_dir: self.settings.input_dir.clone(),
            }))
            .unwrap();
    }
    pub fn send_controller_msg(&self) {
        self.ui_data_sender
            .send(WsMessages::Controller(WsControllerMessage::new(
                &self.last_control,
                self.freeze_x,
                self.freeze_y,
                self.slow_control,
            )))
            .unwrap();
    }
    pub fn send_available_programs_msg(&self, to: Uuid) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::AvailablePrograms(WsAvailableProgramsMessage {
                    progs: self
                        .available_progs
                        .iter()
                        .map(|name| name.to_owned())
                        .map(ProgramInfo::from_string)
                        .collect(),
                    input_dir: self.settings.input_dir.clone(),
                }),
            })
            .unwrap();
    }
    pub fn send_program_data_msg(&self, to: Uuid, program_name: String) {
        let mut file = File::open(program_name.clone()).unwrap();
        let mut program = String::new();
        file.read_to_string(&mut program).unwrap();

        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::LoadProgram {
                    program,
                    program_name,
                    invert_z: self.settings.invert_z,
                    scale: self.settings.scale,
                },
            })
            .unwrap();
    }
    pub fn send_start_reply_message(&self, to: Uuid, program_name: String) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::StartProgram { program_name },
            })
            .unwrap();
    }
    pub fn send_cancel_reply_message(&self, to: Uuid, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::CancelProgram { ok },
            })
            .unwrap();
    }
    pub fn send_save_reply_message(&self, to: Uuid, program_name: String, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::SaveProgram { ok, program_name },
            })
            .unwrap();
    }
    pub fn send_delete_reply_message(&self, to: Uuid, program_name: String, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::DeleteProgram { ok, program_name },
            })
            .unwrap();
    }
    pub fn send_runtime_settings_reply_message(&self, to: Uuid) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::RuntimeSettings {
                    input_dir: self.settings.input_dir.to_owned(),
                    input_update_reduce: self.settings.input_update_reduce,
                    default_speed: self.settings.default_speed,
                    rapid_speed: self.settings.rapid_speed,
                    scale: self.settings.scale,
                    invert_z: self.settings.invert_z,
                    show_console_output: self.settings.show_console_output,
                    console_pos_update_reduce: self.settings.console_pos_update_reduce,
                },
            })
            .unwrap();
    }
    pub fn send_runtime_settings_saved_reply_message(&self, to: Uuid, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::RuntimeSettingsSaved{ ok },
            })
            .unwrap();
    }
    pub fn send_system_settings_reply_message(&self, to: Uuid) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::SystemSettings {
                    dev_mode: self.settings.dev_mode,
                    motor_x: self.settings.motor_x.clone(),
                    motor_y: self.settings.motor_y.clone(),
                    motor_z: self.settings.motor_z.clone(),
                    calibrate_z_gpio: self.settings.calibrate_z_gpio.clone(),
                },
            })
            .unwrap();
    }
    pub fn send_system_settings_saved_reply_message(&self, to: Uuid, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::SystemSettingsSaved{ ok },
            })
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
    pub fn info(&self, msg: String) {
        println!("INFO: {}", msg);
        self.ui_data_sender
            .send(WsMessages::Info(WsInfoMessage::new(InfoLvl::Info, msg)))
            .unwrap();
    }
    pub fn warning(&self, msg: String) {
        println!("WARN: {}", msg);
        self.ui_data_sender
            .send(WsMessages::Info(WsInfoMessage::new(InfoLvl::Warning, msg)))
            .unwrap();
    }
    pub fn error(&self, msg: String) {
        println!("ERROR: {}", msg);
        self.ui_data_sender
            .send(WsMessages::Info(WsInfoMessage::new(InfoLvl::Error, msg)))
            .unwrap();
    }
}

impl App {
    fn apply_control(&mut self, control: Location<f64>) {
        if self.last_control != control {
            self.cnc.manual_move(control.x, control.y, control.z, 20.0);
            self.last_control = control.clone();
            self.send_controller_msg();
        }
    }
    fn manual_mode(&mut self) -> bool {
        // controller just every n-th tick
        self.input_reduce += 1;
        if self.input_reduce < self.settings.input_update_reduce {
            return true;
        }
        self.input_reduce = 0;

        let mut control = self.last_control.clone();
        let speed = if self.slow_control { 2.5f64 } else { 10.0f64 };
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
                        self.warning(format!("start program without calibration"));
                    }
                    if let Some(ref sel_prog) = self.selected_program {
                        if let Ok(load_prog) =
                            Program::new(sel_prog, 5.0, 50.0, 1.0, self.cnc.get_pos(), false)
                        {
                            println!("commands found {:?}", load_prog.len());
                            self.prog = Some(load_prog);
                            self.set_current_mode(Mode::Program);
                        } else {
                            self.error(format!("program is not able to load"));
                        };
                    } else {
                        self.error(format!("No Program selected"));
                    }
                }
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    if !self.freeze_x && value > 0.15 {
                        control.x = (value as f64 - 0.15) / 8.5 * -speed;
                    } else if !self.freeze_x && value < -0.15 {
                        control.x = (value as f64 + 0.15) / 8.5 * -speed;
                    } else {
                        control.x = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    if !self.freeze_y && value > 0.15 {
                        control.y = (value as f64 - 0.15) / 8.5 * speed;
                    } else if !self.freeze_y && value < -0.15 {
                        control.y = (value as f64 + 0.15) / 8.5 * speed;
                    } else {
                        control.y = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::RightStickY, value, _) => {
                    if value > 0.15 {
                        control.z = (value as f64 - 0.15) / 8.5 * speed;
                    } else if value < -0.15 {
                        control.z = (value as f64 + 0.15) / 8.5 * speed;
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
                                self.program_select_cursor = self.available_progs.len() as i32 - 1;
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
                                        .max(0) as usize
                                )
                                .unwrap()
                        );
                    }
                }
                EventType::ButtonPressed(Button::South, _) => {
                    let selected = self
                        .available_progs
                        .get(
                            self.program_select_cursor
                                .min(self.available_progs.len() as i32)
                                .max(0) as usize,
                        )
                        .map(|p| p.to_owned());
                    self.set_selected_program(selected);
                    self.info(format!("select {:?}", self.selected_program));
                }
                EventType::ButtonPressed(Button::North, _) => {
                    let pos = self.cnc.get_pos();
                    if pos.x == 0.0 && pos.y == 0.0 {
                        self.info(format!("reset all (x, y, z)"));
                        self.cnc.reset();
                    } else {
                        self.info(format!("reset only plane move (x, y) -- Reset again without moving to reset the z axis as well"));
                        self.cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                    }
                }
                EventType::ButtonPressed(Button::West, _) => {
                    self.info(format!("calibrate"));
                    self.cnc.calibrate(
                        CalibrateType::None,
                        CalibrateType::None,
                        CalibrateType::ContactPin,
                    );
                    self.set_current_mode(Mode::Calibrate);
                }
                _ => {}
            }
        }

        self.apply_control(control);

        true
    }
    fn program_mode(&mut self) -> bool {
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.info(format!("Cancel current job"));
                self.set_current_mode(Mode::Manual);
                if self.cnc.cancel_task().is_err() {
                    self.error(format!("cancel did not work"));
                    panic!("cancel did not work!");
                };
            }
        }
        if let Some(ref mut prog) = self.prog {
            for next_instruction in prog {
                match next_instruction {
                    NextInstruction::Movement(next_movement) => {
                        self.cnc.query_task(next_movement);
                    }
                    NextInstruction::Miscellaneous(task) => {
                        println!("Miscellaneous {:?}", task);
                        //self.info(format!("Miscellaneous {:?}", task));
                    }
                    NextInstruction::NotSupported(err) => {
                        println!("NotSupported {:?}", err);
                        //self.warning(format!("NotSupported {:?}", err));
                    }
                    NextInstruction::InternalInstruction(err) => {
                        println!("InternalInstruction {:?}", err);
                        //self.info(format!("InternalInstruction {:?}", err));
                    }
                    _ => {}
                };
            }

            match (self.cnc.get_state(), self.in_opp) {
                (MachineState::Idle, true) => {
                    self.set_current_mode(Mode::Manual);
                    self.in_opp = false;
                }
                (MachineState::ProgramTask, false) => {
                    self.calibrated = false;
                    self.in_opp = true;
                }
                _ => (),
            }
        }

        self.set_prog_state(self.cnc.get_steps_todo(), self.cnc.get_steps_done());

        true
    }
    fn calibrate_mode(&mut self) -> bool {
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.set_current_mode(Mode::Manual);
                if self.cnc.cancel_task().is_err() {
                    self.error(format!("cancel did not work"));
                    panic!("cancel did not work!");
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
                self.set_current_mode(Mode::Manual);
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
    pub fn start_program(&mut self, program_name: &String, invert_z: bool, scale: f64) -> bool {
        if !self.calibrated {
            self.warning(format!("start program without calibration"));
        }
        self.set_selected_program(Some(program_name.to_owned()));
        if let Ok(load_prog) = Program::new(
            &program_name,
            5.0,
            50.0,
            scale,
            self.cnc.get_pos(),
            invert_z,
        ) {
            println!("commands found {:?}", load_prog.len());
            self.prog = Some(load_prog);
            self.set_current_mode(Mode::Program);
            true
        } else {
            self.error(format!("program is not able to load"));
            false
        }
    }
    pub fn cancel_program(&mut self) {
        self.set_selected_program(None);
        self.set_current_mode(Mode::Manual);
        if self.cnc.cancel_task().is_err() {
            self.error(format!("cancel did not work"));
            panic!("cancel did not work!");
        };
    }
}

fn main() {
    // let package = WsCommands::Program(WsCommandProgram::Load{program_name: String::from("name")});
    // println!("output {:?}", serde_json::to_string(&package));
    App::start();
}
