mod run;
mod setters;
pub mod settings;
mod ui_communication;

use crate::gnc::Gnc;
use crate::io::{Actor, Switch};
use crate::motor::{
    motor_controller::{ExternalInput, ExternalInputRequest, MotorController},
    Driver, MockMotor, Motor, StepMotor,
};
use crate::types::Location;
use crate::ui::types::{Mode, WsCommandsFrom, WsMessages};

use settings::Settings;

use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::ThreadPool;
use gilrs::Gilrs;
use notify::{raw_watcher, RawEvent, RecursiveMode, Watcher};
use std::{boxed::Box, fs, sync::mpsc, thread, time::Duration};

const SETTINGS_PATH: &str = "./settings.yaml";

pub struct App {
    pub available_progs: Vec<String>,
    pub pool: ThreadPool,
    pub settings: Settings,
    pub gilrs: Gilrs,
    pub in_opp: bool,
    pub cnc: MotorController,
    pub current_mode: Mode,
    pub prog: Option<Gnc>,
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
    pub external_input_enabled: bool,
    pub external_input_sender: mpsc::Sender<ExternalInput>,
    pub external_input_request_receiver: mpsc::Receiver<ExternalInputRequest>,
}

impl App {
    pub fn start() {
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

        // init external_input channel
        let (external_input_sender, external_input_receiver) = mpsc::channel::<ExternalInput>();
        let (external_input_request_sender, external_input_request_receiver) =
            mpsc::channel::<ExternalInputRequest>();

        // return tuple with app and ui channel
        let mut app = App {
            available_progs: App::read_available_progs(&settings.input_dir),
            external_input_enabled: settings.external_input_enabled,
            cnc: App::create_cnc_from_settings(
                &settings,
                external_input_receiver,
                external_input_request_sender,
            ),
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
            external_input_sender,
            external_input_request_receiver,
        };
        app.run(ui_data_receiver, ui_cmd_sender);
    }
    fn create_cnc_from_settings(
        settings: &Settings,
        external_input_receiver: mpsc::Receiver<ExternalInput>,
        external_input_request_sender: mpsc::Sender<ExternalInputRequest>,
    ) -> MotorController {
        let on_off = settings
            .on_off_gpio
            .map(|pin| Actor::new(pin, false, false));
        let z_calibrate = settings.calibrate_z_gpio.map(|pin| Switch::new(pin, false));

        let driver_x: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new(settings.motor_x.step_size))
        } else {
            Box::new(StepMotor::from_settings(settings.motor_x.clone()))
        };
        let driver_y: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new(settings.motor_y.step_size))
        } else {
            Box::new(StepMotor::from_settings(settings.motor_y.clone()))
        };
        let driver_z: Box<dyn Driver + Send> = if settings.dev_mode {
            Box::new(MockMotor::new(settings.motor_z.step_size))
        } else {
            Box::new(StepMotor::from_settings(settings.motor_z.clone()))
        };

        let motor_x = Motor::new(
            "x".to_string(),
            settings.motor_x.max_step_speed.into(),
            settings.motor_x.acceleration,
            settings.motor_x.acceleration_damping,
            settings.motor_x.free_step_speed,
            driver_x,
        );
        let motor_y = Motor::new(
            "y".to_string(),
            settings.motor_y.max_step_speed.into(),
            settings.motor_y.acceleration,
            settings.motor_y.acceleration_damping,
            settings.motor_y.free_step_speed,
            driver_y,
        );
        let motor_z = Motor::new(
            "z".to_string(),
            settings.motor_z.max_step_speed.into(),
            settings.motor_z.acceleration,
            settings.motor_z.acceleration_damping,
            settings.motor_z.free_step_speed,
            driver_z,
        );

        // create cnc MotorController
        MotorController::new(
            on_off,
            settings.switch_on_off_delay,
            motor_x,
            motor_y,
            motor_z,
            z_calibrate,
            settings.external_input_enabled,
            external_input_receiver,
            external_input_request_sender,
        )
    }
    fn gamepad_connected(gilrs: &Gilrs) -> bool {
        let mut gamepad_found = false;
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
            gamepad_found = true;
        }
        gamepad_found
    }
    fn read_available_progs(input_dir: &[String]) -> Vec<String> {
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
        input_dir: &[String],
        available_progs: &[String],
    ) -> (bool, Vec<String>) {
        let new_content = App::read_available_progs(input_dir);

        if new_content.len() != available_progs.len() {
            (true, new_content)
        } else {
            // check if both arrays contain the same content
            let match_count = new_content
                .iter()
                .zip(available_progs)
                .filter(|(a, b)| a == b)
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
            let mut watcher = raw_watcher(tx_fs_changed).unwrap();
            let mut publish_update = false;
            let mut known_progs = vec![];
            loop {
                for path in path_vec.iter() {
                    watcher.watch(path, RecursiveMode::Recursive).unwrap();
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
                            path: Some(_),
                            op: Ok(notify::op::CLOSE_WRITE),
                            ..
                        })
                        | Ok(RawEvent {
                            path: Some(_),
                            op: Ok(notify::op::REMOVE),
                            ..
                        }) => {
                            //println!("{:?}", path);
                            publish_update = true;
                        }
                        _ => {
                            thread::sleep(Duration::new(0, 100_000_000));
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
}
