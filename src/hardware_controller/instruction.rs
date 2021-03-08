use std::ops::{Div, Mul};
use std::time::Duration;

use super::SettingsHardwareController;
use crate::types::Location;

#[derive(Debug, Clone)]
pub enum Instruction {
    Condition(InstructionCondition),
    Settings(SettingsHardwareController),
    ManualMovement(InstructionManualMovement),
    Line(InstructionLine),
    Curve(InstructionCurve),
    Calibrate(InstructionCalibrate),
    MotorOn(InstructionMotorOn),
    MotorOff,
    SetSpeed(InstructionSpeed),
    Delay(f64),
    // all tasks, require user interaction need to be converted into WaitFor instructions
    WaitFor(InstructionWaitFor),
    // active User commands
    Start,
    Stop,
    Pause,
    Resume,
    Emergency,
    // user reply If machine wait for tool change
    ToolChanged(InstructionToolChanged),
    // shutdown the hardware thread
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum InstructionConditions {
    DifferentTool(i32),
    MotorOn,
    MotorOff,
}
#[derive(Debug, Clone)]
pub struct InstructionCondition {
    pub condition: InstructionConditions,
    pub invert: bool,
    pub terminate: bool,
    pub sub_instructions: Vec<Instruction>,
}
#[derive(Debug, Clone)]
pub struct InstructionLine {
    // start position
    p_start: Location<i64>,
    // speed at start [steps/ns]
    v_start: Location<f64>,
    // time for the ramp up from v_start to v_max [ns]
    t_ramp_up: f64,
    // accelerate per ns
    a_acc_t: Location<f64>,

    // speed as main phase [steps/ns]
    v_max: Location<f64>,
    // start point for v_max
    p_v_max_start: Location<i64>,
    // time to be on may speed [ns]
    t_at_max_speed: f64,

    // start point for v_end
    p_v_end_start: Location<i64>,
    // speed after ramp down [steps/ns]
    v_end: Location<f64>,
    // time for the ramp down from v_max to v_end [ns]
    t_ramp_down: f64,
    // decelerate per ns
    a_dec_t: Location<f64>,
    // end position
    p_end: Location<i64>,
    // total time to execute [ns]
    t_total: f64,
}

impl InstructionLine {
    pub fn new(
        p_start: Location<i64>,
        p_end: Location<i64>,

        v_start: Location<f64>,
        v_max: Location<f64>,
        v_end: Location<f64>,

        a_acc_t: Location<f64>,
        a_dec_t: Location<f64>,
    ) -> Self {
        let t_ramp_up_v: Location<f64> = (v_max.clone() - v_start.clone()).div(a_acc_t.clone());
        let t_ramp_up = t_ramp_up_v.max();
        let t_ramp_down_v: Location<f64> = (v_end.clone() - v_max.clone()).div(a_dec_t.clone());
        let t_ramp_down = t_ramp_down_v.max();

        let s_ramp_up = (v_max.clone() + v_start.clone())
            .div(2.0)
            .mul(t_ramp_up)
            .into();
        let s_ramp_down = (v_max.clone() + v_end.clone())
            .div(2.0)
            .mul(t_ramp_down)
            .into();

        let p_v_max_start = p_start.clone() + s_ramp_up;
        let p_v_end_start = p_end.clone() - s_ramp_down;

        let s_at_max_speed: Location<f64> = (p_v_end_start.clone() - p_v_max_start.clone()).into();
        let t_at_max_speed = (s_at_max_speed.div(v_max.clone())).max();

        // let s_v_max = p_v_end_start.clone() - p_v_max_start.clone();
        let t_total = t_at_max_speed + t_ramp_up + t_ramp_down;

        Self {
            a_acc_t,
            a_dec_t,
            p_start,
            v_start,
            t_ramp_up,
            v_max,
            t_at_max_speed,
            p_v_max_start,
            p_v_end_start,
            v_end,
            p_end,
            t_ramp_down,
            t_total,
        }
    }

    pub fn create_without_ramps(p_start: Location<i64>, p_end: Location<i64>, speed: f64) -> Self {
        let delta: Location<f64> = (p_end.clone() - p_start.clone()).into();
        let t_total = delta.distance() / speed;
        let v_max = delta.abs().div(t_total);
        Self {
            v_start: v_max.clone(),
            t_ramp_up: 0.0,
            a_acc_t: Location::<f64>::identity(),

            p_v_max_start: p_start.clone(),
            t_at_max_speed: t_total,

            p_v_end_start: p_end.clone(),
            v_end: v_max.clone(),
            t_ramp_down: 0.0,
            a_dec_t: Location::<f64>::identity(),
            p_start,
            v_max,
            p_end,
            t_total,
        }
    }
    pub fn is_complete(&self, pos: Location<i64>) -> bool {
        self.p_end == pos
    }
    pub fn get_expected_steps_until_now(&self, duration: Duration) -> Location<i64> {
        let d_t = duration.as_nanos() as f64;
        match self.get_phase(d_t) {
            0 => {
                // multiply speed
                let current_speed = self.v_start.clone() + self.a_acc_t.clone() * d_t;
                (self.v_start.clone() + current_speed)
                    .div(2.0f64)
                    .mul(d_t)
                    .into()
            }
            1 => {
                let t_in_max_speed = d_t - self.t_ramp_up;
                self.p_v_max_start.clone() + (self.v_max.clone().mul(t_in_max_speed)).into()
            }
            _ => {
                let current_speed = self.v_max.clone() + self.a_dec_t.clone().mul(d_t);
                let moved_in_end_ramp_until_now =
                    ((self.v_max.clone() + current_speed) / 2.0 * d_t).into();
                self.p_v_end_start.clone() + moved_in_end_ramp_until_now
            }
        }
    }
    fn get_phase(&self, d_t: f64) -> u32 {
        if d_t < self.t_ramp_up {
            0
        } else if d_t < self.t_ramp_up + self.t_at_max_speed {
            1
        } else {
            2
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CircleDirection {
    CW,
    CCW,
}

#[derive(Debug, Clone)]
pub struct InstructionCurve {
    pub v_start: Location<f64>,
    pub t_ramp_up: u64,
    pub t_at_max_speed: u64,
    // t_ramp_up: u64,
    pub v_end: Location<f64>,
    // curve data
    pub p_end: Location<i64>,

    /** circle center */
    pub p_center: Location<i64>,
    /** initial set radius in steps (float to comp with output of step_sizes) */
    pub radius_sq: f64,
    /**
     * correction value to calculate the radius when the sep sizes differ on the axes
     * # Example:
     * ```
     * let distance: Location<i64> = pos() - center;
     * let radius_sq = (distance.into() * step_sizes).distance_sq()
     */
    pub step_sizes: Location<f64>,
    /** cw or cww direction to mill the circle */
    pub turn_direction: CircleDirection,

    /** max speed steps/ns */
    pub v_max: f64,
    /** seconds between steps */
    pub step_delay: f64,
}
#[derive(Debug, Clone)]
pub enum CalibrateType {
    None,
    Min,
    Max,
    Middle,
    ContactPin,
}
#[derive(Debug, Clone)]
pub struct InstructionCalibrate {
    pub x: CalibrateType,
    pub y: CalibrateType,
    pub z: CalibrateType,
}
#[derive(Debug, Clone)]
pub struct InstructionMotorOn {
    pub speed: f64,
    pub cw: bool,
}
#[derive(Debug, Clone)]
pub struct InstructionSpeed {
    pub speed: f64,
    pub cw: bool,
}
#[derive(Debug, Clone)]
pub struct InstructionToolChanged {
    pub tool_id: i32,
    pub length: Option<f64>,
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InstructionWaitFor {
    ToolChanged(i32, Option<f64>),
}
#[derive(Debug, Clone)]
pub struct InstructionManualMovement {
    /** speed from +- speed in steps/sec */
    pub speed: Location<f64>,
}

impl InstructionManualMovement {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            speed: Location::new(x, y, z),
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.speed.is_null()
    }

    pub fn steps_in_time(&self, dt: Duration) -> Location<i64> {
        self.speed.clone().mul(dt.as_secs_f64()).into()
    }
}
