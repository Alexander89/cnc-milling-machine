use crate::types::MachineState;

#[derive(Debug)]
pub struct PosData {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}
#[derive(Debug)]
pub enum HardwareFeedback {
    Pos(PosData),
    State(MachineState),
    // todo, done
    Progress(u32, u32),
    RequireToolChange(i32, Option<f64>),
}
