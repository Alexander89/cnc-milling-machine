use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};
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
pub struct WsStatusMessage {
    pub mode: Mode,
    pub dev_mode: bool,
    pub speed: u32,
    pub in_opp: bool,
    pub current_prog: Option<String>,
    pub calibrated: bool,
}
impl WsStatusMessage {
    pub fn new(
        mode: Mode,
        dev_mode: bool,
        speed: u32,
        in_opp: bool,
        current_prog: Option<String>,
        calibrated: bool,
    ) -> WsStatusMessage {
        WsStatusMessage {
            mode,
            dev_mode,
            speed,
            in_opp,
            current_prog,
            calibrated,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Message)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
#[rtype(result = "()")]
pub enum WsMessages {
    Connected(WsConnectedMessage),
    Position(WsPositionMessage),
    Status(WsStatusMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "cmd")]
pub enum WsCommands {
    Prog(WsCommandProg),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum WsCommandProg {
    Load(String),
}
