use crate::app::settings::MotorSettings;
use crate::types::Location;
use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};
use std::{fs, time::SystemTime};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsMessages>,
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
    Manual,
    Program,
    Calibrate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsConnectedMessage {
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
pub struct WsInfoMessage {
    pub lvl: InfoLvl,
    pub message: String,
}

impl WsInfoMessage {
    pub fn new(lvl: InfoLvl, message: String) -> WsInfoMessage {
        WsInfoMessage { lvl, message }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsPositionMessage {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl WsPositionMessage {
    pub fn new(x: f64, y: f64, z: f64) -> WsPositionMessage {
        WsPositionMessage { x, y, z }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsControllerMessage {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub freeze_x: bool,
    pub freeze_y: bool,
    pub slow: bool,
}
impl WsControllerMessage {
    pub fn new(
        pos: &Location<f64>,
        freeze_x: bool,
        freeze_y: bool,
        slow: bool,
    ) -> WsControllerMessage {
        WsControllerMessage {
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
pub struct WsStatusMessage {
    pub mode: Mode,
    pub dev_mode: bool,
    pub in_opp: bool,
    pub current_prog: Option<String>,
    pub calibrated: bool,
    pub steps_todo: i64,
    pub steps_done: i64,
    pub is_switched_on: bool,
}
impl WsStatusMessage {
    pub fn new(
        mode: Mode,
        dev_mode: bool,
        in_opp: bool,
        current_prog: Option<String>,
        calibrated: bool,
        steps_todo: i64,
        steps_done: i64,
        is_switched_on: bool,
    ) -> WsStatusMessage {
        WsStatusMessage {
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
            name: name.to_owned(),
            path: String::from(""),
            size: metadata.len(),
            lines_of_code: 0,
            create_date_ts: metadata
                .created()
                .unwrap_or(SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            modified_date_ts: metadata
                .modified()
                .unwrap_or(SystemTime::now())
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub struct WsAvailableProgramsMessage {
    pub progs: Vec<ProgramInfo>,
    pub input_dir: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum WsReplyMessage {
    AvailablePrograms(WsAvailableProgramsMessage),
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
        motor_x: MotorSettings,
        motor_y: MotorSettings,
        motor_z: MotorSettings,
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
pub enum WsMessages {
    Connected(WsConnectedMessage),
    Info(WsInfoMessage),
    Position(WsPositionMessage),
    ProgsUpdate(WsAvailableProgramsMessage),
    Controller(WsControllerMessage),
    Status(WsStatusMessage),
    Reply { to: Uuid, msg: WsReplyMessage },
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct WsCommandsFrom(pub Uuid, pub WsCommands);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "cmd")]
pub enum WsCommands {
    Program(WsCommandProgram),
    Control(WsCommandControl),
    Controller(WsCommandController),
    Settings(WsCommandSettings),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum WsCommandProgram {
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
pub enum WsCommandControl {
    // Move { direction: string, speed: f64},
    OnOff { on: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum WsCommandController {
    FreezeX { freeze: bool },
    FreezeY { freeze: bool },
    Slow { slow: bool },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "action")]
pub enum WsCommandSettings {
    GetSystem,
    #[serde(rename_all = "camelCase")]
    SetSystem(WsCommandSettingsSetSystemSettings),
    GetRuntime,
    SetRuntime(WsCommandSettingsSetRuntimeSettings),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsCommandSettingsSetSystemSettings {
    pub dev_mode: bool,
    pub motor_x: MotorSettings,
    pub motor_y: MotorSettings,
    pub motor_z: MotorSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibrate_z_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_off_gpio: Option<u8>,
    pub switch_on_off_delay: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsCommandSettingsSetRuntimeSettings {
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
}
