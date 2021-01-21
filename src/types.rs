#![allow(dead_code)]
use std::convert::{From, Into};
use std::{
    cmp::PartialOrd,
    fmt::{self, Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(PartialEq, Clone, Debug)]
pub enum MachineState {
    Idle,
    ManualTask,
    ProgramTask,
    Calibrate,
    Unknown,
}
impl Into<u32> for MachineState {
    fn into(self) -> u32 {
        match self {
            MachineState::Idle => 0,
            MachineState::ManualTask => 1,
            MachineState::ProgramTask => 2,
            MachineState::Calibrate => 3,
            MachineState::Unknown => 99,
        }
    }
}
impl Into<MachineState> for u32 {
    fn into(self) -> MachineState {
        match self {
            0 => MachineState::Idle,
            1 => MachineState::ManualTask,
            2 => MachineState::ProgramTask,
            3 => MachineState::Calibrate,
            _ => MachineState::Unknown,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum Direction {
    Left,
    Right,
}
impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Left => write!(f, "Left"),
            Direction::Right => write!(f, "Right"),
        }
    }
}
impl Debug for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Left => write!(f, "Left"),
            Direction::Right => write!(f, "Right"),
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Location<T: Debug + PartialEq> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl Location<f64> {
    pub fn distance(&self) -> f64 {
        self.distance_sq().sqrt()
    }
    pub fn distance_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn max(&self) -> f64 {
        self.x.max(self.y).max(self.z)
    }
    pub fn min(&self) -> f64 {
        self.x.min(self.y).min(self.z)
    }
}

impl Location<i64> {
    pub fn abs(&self) -> Location<u64> {
        Location::<u64> {
            x: self.x.abs() as u64,
            y: self.y.abs() as u64,
            z: self.z.abs() as u64,
        }
    }
    pub fn distance_sq(&self) -> i64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn identity() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}
impl Location<i32> {
    pub fn abs(&self) -> Location<u32> {
        Location::<u32> {
            x: self.x.abs() as u32,
            y: self.y.abs() as u32,
            z: self.z.abs() as u32,
        }
    }
    pub fn identity() -> Self {
        Self { x: 1, y: 1, z: 1 }
    }
}
impl Location<f64> {
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }
    pub fn identity() -> Self {
        Self {
            x: 1.0f64,
            y: 1.0f64,
            z: 1.0f64,
        }
    }
}
impl Location<f32> {
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
    pub fn floor(&self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }
    pub fn identity() -> Self {
        Self {
            x: 1.0f32,
            y: 1.0f32,
            z: 1.0f32,
        }
    }
}

impl<T> Default for Location<T>
where
    T: Debug + PartialEq + Default,
{
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T> Location<T>
where
    T: Debug + PartialEq + Default + Copy,
{
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
    pub fn split(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }
}

impl<T> Location<T>
where
    T: Debug + PartialEq + Copy + Neg + From<<T as Neg>::Output>,
{
    pub fn rot_z_cw_90(&self) -> Self {
        Self {
            x: self.y,
            y: (-self.x).into(),
            z: self.z,
        }
    }
    pub fn rot_z_ccw_90(&self) -> Self {
        Self {
            x: (-self.y).into(),
            y: self.x,
            z: self.z,
        }
    }
}

impl<T> Display for Location<T>
where
    T: Debug + Display + PartialEq,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x: {} y: {} z: {})", self.x, self.y, self.z)
    }
}
impl<T> Debug for Location<T>
where
    T: Debug + PartialEq,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x: {:?} y: {:?} z: {:?})", self.x, self.y, self.z)
    }
}
impl<T: Add<Output = T>> Add for Location<T>
where
    T: Debug + PartialEq,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl<T: Sub<Output = T>> Sub for Location<T>
where
    T: Debug + PartialEq,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Mul<Output = T>> Mul<T> for Location<T>
where
    T: Debug + PartialEq + Copy,
{
    type Output = Self;

    fn mul(self, factor: T) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }
}
impl<T: Mul<Output = T>> Mul<Location<T>> for Location<T>
where
    T: Debug + PartialEq + Copy,
{
    type Output = Self;

    fn mul(self, factor: Self) -> Self {
        Self {
            x: self.x * factor.x,
            y: self.y * factor.y,
            z: self.z * factor.z,
        }
    }
}
impl<T: Div<Output = T>> Div<Location<T>> for Location<T>
where
    T: Debug + PartialEq + Copy,
{
    type Output = Self;

    fn div(self, factor: Self) -> Self {
        Self {
            x: self.x / factor.x,
            y: self.y / factor.y,
            z: self.z / factor.z,
        }
    }
}
impl<T: Div<Output = T>> Div<T> for Location<T>
where
    T: Debug + PartialEq + Copy,
{
    type Output = Self;

    fn div(self, factor: T) -> Self {
        Self {
            x: self.x / factor,
            y: self.y / factor,
            z: self.z / factor,
        }
    }
}
impl Into<Location<f64>> for Location<i64> {
    fn into(self) -> Location<f64> {
        Location {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }
}
impl Into<Location<i64>> for Location<f64> {
    fn into(self) -> Location<i64> {
        Location {
            x: self.x.round() as i64,
            y: self.y.round() as i64,
            z: self.z.round() as i64,
        }
    }
}
impl Into<Location<u128>> for Location<i64> {
    fn into(self) -> Location<u128> {
        let abs = self.abs();
        Location {
            x: abs.x as u128,
            y: abs.y as u128,
            z: abs.z as u128,
        }
    }
}
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CircleDirection {
    CW,
    CCW,
}

#[derive(Debug, Clone)]
pub struct LinearMovement {
    /** delta move */
    pub delta: Location<f64>,
    /** delta distance */
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct CircleMovement {
    /** circle center */
    pub center: Location<f64>,
    /** initial radius in mm (²) */
    pub radius_sq: f64,
    /** cw or cww direction to mill the circle */
    pub turn_direction: CircleDirection,
}

#[derive(Debug, Clone)]
pub enum MoveType {
    Linear(LinearMovement),
    Circle(CircleMovement),
    Rapid(LinearMovement),
}
impl Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MoveType::Linear(_) => write!(f, "Move linear"),
            MoveType::Circle(_) => write!(f, "Move circle"),
            MoveType::Rapid(_) => write!(f, "Move rapid"),
        }
    }
}

#[derive(Debug)]
pub struct SteppedLinearMovement {
    /** delta move */
    pub delta: Location<i64>,
    /** delta distance in mm (to calculate speed) */
    pub distance: f64,
    /** speed in mm/sec */
    pub speed: f64,
}

#[derive(Debug)]
pub struct SteppedCircleMovement {
    /** circle center */
    pub center: Location<i64>,
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

    /** max speed */
    pub speed: f64,
    /** seconds betwean steps */
    pub step_delay: f64,
}
#[derive(Debug)]
pub enum SteppedMoveType {
    Linear(SteppedLinearMovement),
    Circle(SteppedCircleMovement),
    Rapid(SteppedLinearMovement),
}
impl Display for SteppedMoveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SteppedMoveType::Linear(_) => write!(f, "Move linear"),
            SteppedMoveType::Circle(_) => write!(f, "Move circle"),
            SteppedMoveType::Rapid(_) => write!(f, "Move rapid"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircleStepDir {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleStepCW {
    main: CircleStepDir,
    opt: CircleStepDir,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleStepCCW {
    main: CircleStepDir,
    opt: CircleStepDir,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleStep {
    pub main: CircleStepDir,
    pub opt: CircleStepDir,
}

impl<T> Into<CircleStepCCW> for Location<T>
where
    T: Debug
        + PartialEq
        + Copy
        + Neg
        + Default
        + From<<T as Neg>::Output>
        + PartialOrd
        + Into<<T as Neg>::Output>,
{
    fn into(self) -> CircleStepCCW {
        let turned = self.rot_z_ccw_90();

        let nul = T::default();
        // go right
        let abs_x: T = if turned.x < nul {
            (-turned.x).into()
        } else {
            turned.x
        };
        let abs_y: T = if turned.y < nul {
            (-turned.y).into()
        } else {
            turned.y
        };

        // ccw
        if turned.x >= nul {
            if turned.y < nul && abs_x <= abs_y {
                // go down / opt right
                CircleStepCCW {
                    main: CircleStepDir::Down,
                    opt: CircleStepDir::Right,
                }
            } else if turned.y > nul && abs_x < abs_y {
                // go up / opt right
                CircleStepCCW {
                    main: CircleStepDir::Up,
                    opt: CircleStepDir::Right,
                }
            } else if turned.y <= nul {
                // go right / opt down
                CircleStepCCW {
                    main: CircleStepDir::Right,
                    opt: CircleStepDir::Down,
                }
            } else {
                // go right / opt up
                CircleStepCCW {
                    main: CircleStepDir::Right,
                    opt: CircleStepDir::Up,
                }
            }
        } else {
            // go left
            if turned.y > nul && abs_x <= abs_y {
                // go up / opt left
                CircleStepCCW {
                    main: CircleStepDir::Up,
                    opt: CircleStepDir::Left,
                }
            } else if turned.y < nul && abs_x < abs_y {
                // go down / opt left
                CircleStepCCW {
                    main: CircleStepDir::Down,
                    opt: CircleStepDir::Left,
                }
            } else if turned.y >= nul {
                // go left / opt up
                CircleStepCCW {
                    main: CircleStepDir::Left,
                    opt: CircleStepDir::Up,
                }
            } else {
                // go left / opt down
                CircleStepCCW {
                    main: CircleStepDir::Left,
                    opt: CircleStepDir::Down,
                }
            }
        }
    }
}

impl<T> Into<CircleStepCW> for Location<T>
where
    T: Debug
        + PartialEq
        + Copy
        + Neg
        + Default
        + From<<T as Neg>::Output>
        + PartialOrd
        + Into<<T as Neg>::Output>,
{
    fn into(self) -> CircleStepCW {
        let turned = self.rot_z_cw_90();

        let nul = T::default();
        // go right
        let abs_x: T = if turned.x < nul {
            (-turned.x).into()
        } else {
            turned.x
        };
        let abs_y: T = if turned.y < nul {
            (-turned.y).into()
        } else {
            turned.y
        };

        // cw
        if turned.x >= nul {
            // go right
            if turned.y < nul && abs_x < abs_y {
                // go down / opt right
                CircleStepCW {
                    main: CircleStepDir::Down,
                    opt: CircleStepDir::Right,
                }
            } else if turned.y > nul && abs_x <= abs_y {
                // go up / opt right
                CircleStepCW {
                    main: CircleStepDir::Up,
                    opt: CircleStepDir::Right,
                }
            } else if turned.y < nul {
                // go right / opt down
                CircleStepCW {
                    main: CircleStepDir::Right,
                    opt: CircleStepDir::Down,
                }
            } else {
                // go right / opt up
                CircleStepCW {
                    main: CircleStepDir::Right,
                    opt: CircleStepDir::Up,
                }
            }
        } else {
            // go left
            if turned.y > nul && abs_x < abs_y {
                // go up / opt left
                CircleStepCW {
                    main: CircleStepDir::Up,
                    opt: CircleStepDir::Left,
                }
            } else if turned.y < nul && abs_x <= abs_y {
                // go down / opt left
                CircleStepCW {
                    main: CircleStepDir::Down,
                    opt: CircleStepDir::Left,
                }
            } else if turned.y > nul {
                // go left / opt up
                CircleStepCW {
                    main: CircleStepDir::Left,
                    opt: CircleStepDir::Up,
                }
            } else {
                // go left / opt down
                CircleStepCW {
                    main: CircleStepDir::Left,
                    opt: CircleStepDir::Down,
                }
            }
        }
    }
}

impl Into<CircleStep> for CircleStepCW {
    fn into(self) -> CircleStep {
        CircleStep {
            main: self.main,
            opt: self.opt,
        }
    }
}
impl Into<CircleStep> for CircleStepCCW {
    fn into(self) -> CircleStep {
        CircleStep {
            main: self.main,
            opt: self.opt,
        }
    }
}
