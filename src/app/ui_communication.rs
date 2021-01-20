use super::App;

use crate::types::Location;
use crate::ui::types::{
    InfoLvl, ProgramInfo, WsAvailableProgramsMessage, WsControllerMessage, WsInfoMessage,
    WsMessages, WsPositionMessage, WsReplyMessage, WsStatusMessage,
};

use std::{fs::File, io::prelude::*};
use uuid::Uuid;

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