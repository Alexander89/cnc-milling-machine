use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Manual,
    Program,
    Calibrate,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsPositionMessage {
    x: u64,
    y: u64,
    z: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsStatusMessage {
    mode: Mode,
    dev_mode: bool,
    speed: u32,
    in_opp: bool,
    current_prog: Option<String>,
    calibrated: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum WsMessages {
    Position(WsPositionMessage),
    Status(WsStatusMessage),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "cmd")]
pub enum WsCommands {
    Prog(WsCommandProg),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum WsCommandProg {
    Load(String),
}
