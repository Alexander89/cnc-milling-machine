use super::App;

use crate::gnc::NextMiscellaneous;
use crate::types::Location;
use crate::ui::types::{
    InfoLvl, ProgramInfo, WsAvailableProgramsMessage, WsCommandControl, WsCommandController,
    WsCommandProgram, WsCommandSettings, WsCommands, WsCommandsFrom, WsControllerMessage,
    WsInfoMessage, WsMessages, WsPositionMessage, WsReplyMessage, WsStatusMessage,
};
use std::{
    fs::{remove_file, File, OpenOptions},
    io::prelude::*,
    sync::mpsc::Sender,
    thread,
    time::Duration,
};
use uuid::Uuid;

impl App {
    // handle incoming commands
    pub fn handle_network_commands(&mut self, update_path: &Sender<Vec<String>>) {
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
                WsCommands::Control(WsCommandControl::OnOff { on }) => {
                    if self.cnc.is_switched_on() != on {
                        println!("switch to {}", on);
                        self.cnc.manual_miscellaneous(if on {
                            NextMiscellaneous::SwitchOn
                        } else {
                            NextMiscellaneous::SwitchOff
                        });
                        thread::sleep(Duration::from_secs_f64(0.2f64));
                        self.send_status_msg();
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
                WsCommands::Settings(WsCommandSettings::SetRuntime(settings)) => {
                    match self.set_runtime_settings(settings, &update_path) {
                        Ok(()) => self.send_runtime_settings_saved_reply_message(uuid, true),
                        Err(_) => self.send_runtime_settings_saved_reply_message(uuid, false),
                    };
                }
                WsCommands::Settings(WsCommandSettings::GetSystem) => {
                    self.send_system_settings_reply_message(uuid);
                }
                WsCommands::Settings(WsCommandSettings::SetSystem(settings)) => {
                    match self.set_system_settings(settings) {
                        Ok(()) => self.send_system_settings_saved_reply_message(uuid, true),
                        Err(_) => self.send_system_settings_saved_reply_message(uuid, false),
                    };
                } //_ => (),
            };
        };
    }
}

impl App {
    pub fn get_status_msg(&self) -> WsStatusMessage {
        WsStatusMessage::new(
            self.current_mode.clone(),
            self.settings.dev_mode,
            self.in_opp,
            self.selected_program.clone(),
            self.calibrated,
            self.steps_todo,
            self.steps_done,
            self.cnc.is_switched_on(),
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
                msg: WsReplyMessage::RuntimeSettingsSaved { ok },
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
                    on_off_gpio: self.settings.on_off_gpio.clone(),
                    switch_on_off_delay: self.settings.switch_on_off_delay,
                },
            })
            .unwrap();
    }
    pub fn send_system_settings_saved_reply_message(&self, to: Uuid, ok: bool) {
        self.ui_data_sender
            .send(WsMessages::Reply {
                to,
                msg: WsReplyMessage::SystemSettingsSaved { ok },
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
