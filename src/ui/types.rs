#![allow(clippy::too_many_arguments)]
use crate::settings::SettingsMotor;
use crate::types::Location;
use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};
use std::{fs, time::SystemTime};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<UiMessages>,
    pub self_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Idle,
    Manual,
    Program,
    Calibrate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConnectedMessage {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InfoLvl {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiInfoMessage {
    pub lvl: InfoLvl,
    pub message: String,
}

impl UiInfoMessage {
    pub fn new(lvl: InfoLvl, message: String) -> UiInfoMessage {
        UiInfoMessage { lvl, message }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionMessage {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl UiPositionMessage {
    pub fn new(x: f64, y: f64, z: f64) -> UiPositionMessage {
        UiPositionMessage { x, y, z }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiControllerMessage {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub freeze_x: bool,
    pub freeze_y: bool,
    pub slow: bool,
}
impl UiControllerMessage {
    pub fn new(
        pos: &Location<f64>,
        freeze_x: bool,
        freeze_y: bool,
        slow: bool,
    ) -> UiControllerMessage {
        UiControllerMessage {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            freeze_x,
            freeze_y,
            slow,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiStatusMessage {
    pub mode: Mode,
    pub dev_mode: bool,
    pub in_opp: bool,
    pub current_prog: Option<String>,
    pub calibrated: bool,
    pub steps_todo: i64,
    pub steps_done: i64,
    pub is_switched_on: bool,
}
impl UiStatusMessage {
    pub fn new(
        mode: Mode,
        dev_mode: bool,
        in_opp: bool,
        current_prog: Option<String>,
        calibrated: bool,
        steps_todo: i64,
        steps_done: i64,
        is_switched_on: bool,
    ) -> UiStatusMessage {
        UiStatusMessage {
            mode,
            dev_mode,
            in_opp,
            current_prog,
            calibrated,
            steps_todo,
            steps_done,
            is_switched_on,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub lines_of_code: u32,
    pub create_date_ts: u64,
    pub modified_date_ts: u64,
}

impl ProgramInfo {
    pub fn from_string(name: String) -> ProgramInfo {
        let metadata = fs::metadata(name.clone()).unwrap();
        ProgramInfo {
            name,
            path: String::from(""),
            size: metadata.len(),
            lines_of_code: 0,
            create_date_ts: metadata
                .created()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_date_ts: metadata
                .modified()
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct UiAvailableProgramsMessage {
    pub progs: Vec<ProgramInfo>,
    pub input_dir: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum UiReplyMessage {
    AvailablePrograms(UiAvailableProgramsMessage),
    #[serde(rename_all = "camelCase")]
    LoadProgram {
        program_name: String,
        program: String,
        invert_z: bool,
        scale: f64,
    },
    #[serde(rename_all = "camelCase")]
    SaveProgram {
        program_name: String,
        ok: bool,
    },
    #[serde(rename_all = "camelCase")]
    DeleteProgram {
        program_name: String,
        ok: bool,
    },
    #[serde(rename_all = "camelCase")]
    StartProgram {
        program_name: String,
    },
    CancelProgram {
        ok: bool,
    },
    #[serde(rename_all = "camelCase")]
    RuntimeSettings {
        input_dir: Vec<String>,
        input_update_reduce: u32,
        default_speed: f64,
        rapid_speed: f64,
        scale: f64,
        invert_z: bool,
        show_console_output: bool,
        console_pos_update_reduce: u32,
    },
    RuntimeSettingsSaved {
        ok: bool,
    },
    #[serde(rename_all = "camelCase")]
    SystemSettings {
        dev_mode: bool,
        motor_x: SettingsMotor,
        motor_y: SettingsMotor,
        motor_z: SettingsMotor,
        #[serde(skip_serializing_if = "Option::is_none")]
        calibrate_z_gpio: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        on_off_gpio: Option<u8>,
        switch_on_off_delay: f64,
    },
    SystemSettingsSaved {
        ok: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, Message)]
#[serde(rename_all = "camelCase", tag = "type")]
#[rtype(result = "()")]
pub enum UiMessages {
    Connected(UiConnectedMessage),
    Info(UiInfoMessage),
    Position(UiPositionMessage),
    ProgsUpdate(UiAvailableProgramsMessage),
    Controller(UiControllerMessage),
    Status(UiStatusMessage),
    Reply { to: Uuid, msg: UiReplyMessage },
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct UiCommandsFrom(pub Uuid, pub UiCommands);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "cmd")]
pub enum UiCommands {
    Program(UiCommandProgram),
    Control(UiCommandControl),
    Controller(UiCommandController),
    Settings(UiCommandSettings),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum UiCommandProgram {
    Get,
    #[serde(rename_all = "camelCase")]
    Load {
        program_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Save {
        program_name: String,
        program: String,
    },
    #[serde(rename_all = "camelCase")]
    Delete {
        program_name: String,
    },
    #[serde(rename_all = "camelCase")]
    Start {
        program_name: String,
        invert_z: bool,
        scale: f64,
    },
    Cancel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum UiCommandControl {
    // Move { direction: string, speed: f64},
    OnOff { on: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum UiCommandController {
    FreezeX { freeze: bool },
    FreezeY { freeze: bool },
    Slow { slow: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum UiCommandSettings {
    GetSystem,
    #[serde(rename_all = "camelCase")]
    SetSystem(UiCommandSettingsSetSystemSettings),
    GetRuntime,
    SetRuntime(UiCommandSettingsSetRuntimeSettings),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCommandSettingsSetSystemSettings {
    pub dev_mode: bool,
    pub motor_x: SettingsMotor,
    pub motor_y: SettingsMotor,
    pub motor_z: SettingsMotor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibrate_z_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_off_gpio: Option<u8>,
    pub switch_on_off_delay: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCommandSettingsSetRuntimeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_dir: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_update_reduce: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rapid_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invert_z: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_console_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_pos_update_reduce: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_input_enabled: Option<bool>,
}
