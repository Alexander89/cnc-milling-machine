mod console;
mod types;

//use futures::executor::ThreadPool;
use console::Console;

use crate::{
    app::{ConsoleOut, SystemPublisher, SystemSubscriber},
    settings::SettingsUi
};

pub struct Ui;

impl Ui {
    pub fn new(event_publish: SystemPublisher, event_subscribe: SystemSubscriber, out: ConsoleOut, settings: SettingsUi) -> Ui {

        /* if settings.web {
            let pool = ThreadPool::new().expect("Failed to build pool");
            pool.spawn_ok(async {
                ui_main(
                    cmd_sender,
                    data_receiver,
                    pos_msg,
                    status_msg,
                    controller_msg,
                )
                .expect("could not start WS-server");
            });
        } */
        if settings.console {
            Console::new(event_publish, event_subscribe, out);
        }

        // let ui_thread = thread::spawn(move || {
        //     let ui = UiState {
        //         pool,

        //         available_progs: Vec::new(),
        //         program_select_cursor: 0,
        //         selected_program: None,

        //         in_opp: false,
        //         current_mode: MachineState::Idle,
        //         calibrated: false,
        //         steps_todo: 0,
        //         steps_done: 0,
        //     };
        //     ui.poll();
        // });
        Ui {}
    }
}
/*
pub struct UiState {
    pool: ThreadPool,

    available_progs: Vec<String>,
    program_select_cursor: i32,
    selected_program: Option<String>,

    in_opp: bool,
    current_mode: MachineState,
    calibrated: bool,
    steps_todo: i64,
    steps_done: i64,
}

impl UiState {
    //fn create() -> () {
    //    let (ui_data_sender, ui_data_receiver) = unbounded::<UiMessages>();
    //    let (ui_cmd_sender, ui_cmd_receiver) = unbounded::<UiCommandsFrom>();
    //}


    pub fn handle_network_commands(&mut self, update_path: &Sender<Vec<String>>) {
        if let Ok(UiCommandsFrom(uuid, cmd)) = self.ui_cmd_receiver.try_recv() {
            match cmd {
                UiCommands::Controller(UiCommandController::FreezeX { freeze }) => {
                    if self.freeze_x != freeze {
                        self.freeze_x = freeze;
                        self.send_controller_msg();
                    }
                }
                UiCommands::Controller(UiCommandController::FreezeY { freeze }) => {
                    if self.freeze_y != freeze {
                        self.freeze_y = freeze;
                        self.send_controller_msg();
                    }
                }
                UiCommands::Controller(UiCommandController::Slow { slow }) => {
                    if self.slow_control != slow {
                        self.slow_control = slow;
                        self.send_controller_msg();
                    }
                }
                UiCommands::Control(UiCommandControl::OnOff { on }) => {
                    if self.cnc.is_switched_on() != on {
                        println!("switch to {}", on);
                        self.cnc.(if on {
                            NextMiscellaneous::SwitchOn
                        } else {
                            NextMiscellaneous::SwitchOff
                        });
                        thread::sleep(Duration::from_secs_f64(0.2f64));
                        self.send_status_msg();
                    }
                }
                UiCommands::Program(UiCommandProgram::Get) => {
                    self.send_available_programs_msg(uuid)
                }
                UiCommands::Program(UiCommandProgram::Load { program_name }) => {
                    self.send_program_data_msg(uuid, program_name)
                }
                UiCommands::Program(UiCommandProgram::Start {
                    program_name,
                    invert_z,
                    scale,
                }) => {
                    if self.start_program(&program_name, invert_z, scale) {
                        self.send_start_reply_message(uuid, program_name)
                    }
                }
                UiCommands::Program(UiCommandProgram::Cancel) => {
                    self.cancel_program();
                    self.send_cancel_reply_message(uuid, true);
                }
                UiCommands::Program(UiCommandProgram::Save {
                    program_name,
                    program,
                }) => {
                    match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(program_name.clone())
                    {
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
                                Ok(_) => self.send_save_reply_message(uuid, program_name, true),
                            };
                        }
                    }
                }
                UiCommands::Program(UiCommandProgram::Delete { program_name }) => {
                    self.send_delete_reply_message(
                        uuid,
                        program_name.clone(),
                        remove_file(program_name).is_ok(),
                    );
                }
                UiCommands::Settings(UiCommandSettings::GetRuntime) => {
                    self.send_runtime_settings_reply_message(uuid);
                }
                UiCommands::Settings(UiCommandSettings::SetRuntime(settings)) => {
                    match self.set_runtime_settings(settings, &update_path) {
                        Ok(()) => self.send_runtime_settings_saved_reply_message(uuid, true),
                        Err(_) => self.send_runtime_settings_saved_reply_message(uuid, false),
                    };
                }
                UiCommands::Settings(UiCommandSettings::GetSystem) => {
                    self.send_system_settings_reply_message(uuid);
                }
                UiCommands::Settings(UiCommandSettings::SetSystem(settings)) => {
                    match self.set_system_settings(settings) {
                        Ok(()) => self.send_system_settings_saved_reply_message(uuid, true),
                        Err(_) => self.send_system_settings_saved_reply_message(uuid, false),
                    };
                } //_ => (),
            };
        };
    }
}
 */
