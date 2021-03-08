use crate::types::Location;
use std::time::SystemTime;

use super::InstructionWaitFor;

pub struct CalibrateData {
    pub steps_done: u64,
    pub phase: i32,
    pub pos_1: i64,
    pub complete: bool,
}
impl Default for CalibrateData {
    fn default() -> Self {
        Self {
            steps_done: 0,
            phase: 0,
            pos_1: 0,
            complete: false,
        }
    }
}

pub struct OpState {
    pub shutdown: bool,
    pub last_data_send: SystemTime,

    pub wait_for: Option<InstructionWaitFor>,
    // current tool length
    pub tool_id: i32,
    pub tool_length: f64,

    // start_infos for job
    pub start_time: Option<SystemTime>,
    pub start_pos: Location<i64>,

    // run curve
    pub curve_close_to_destination: bool,
    pub last_distance_to_destination: u32,
    pub curve_steps_done: u64,

    // calibrate
    pub calibrate_x: CalibrateData,
    pub calibrate_y: CalibrateData,
    pub calibrate_z: CalibrateData,
}

impl Default for OpState {
    fn default() -> Self {
        Self {
            shutdown: false,
            last_data_send: SystemTime::UNIX_EPOCH,
            wait_for: None,
            tool_id: 1,
            tool_length: 0.0,
            start_time: None,
            start_pos: Location::default(),
            curve_close_to_destination: false,
            last_distance_to_destination: 100,
            curve_steps_done: 0,
            calibrate_x: CalibrateData::default(),
            calibrate_y: CalibrateData::default(),
            calibrate_z: CalibrateData::default(),
        }
    }
}
impl OpState {
    pub fn reset_calibrate(&mut self) {
        self.calibrate_x = CalibrateData::default();
        self.calibrate_y = CalibrateData::default();
        self.calibrate_z = CalibrateData::default();
    }
}
