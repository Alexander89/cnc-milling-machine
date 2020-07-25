#![allow(dead_code)]
use std::convert::Into;
use std::{
    fmt::{self, Debug, Display},
    ops::{Add, Div, Mul, Sub},
};

#[derive(PartialEq, Clone)]
pub enum MachineState {
    Idle,
    ManualTask,
    ProgrammTask,
    Unknown,
}
impl Into<u32> for MachineState {
    fn into(self: Self) -> u32 {
        match self {
            MachineState::Idle => 0,
            MachineState::ManualTask => 1,
            MachineState::ProgrammTask => 2,
            MachineState::Unknown => 99,
        }
    }
}
impl Into<MachineState> for u32 {
    fn into(self: Self) -> MachineState {
        match self {
            0 => MachineState::Idle,
            1 => MachineState::ManualTask,
            2 => MachineState::ProgrammTask,
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
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
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
}
impl Location<i32> {
    pub fn abs(&self) -> Location<u32> {
        Location::<u32> {
            x: self.x.abs() as u32,
            y: self.y.abs() as u32,
            z: self.z.abs() as u32,
        }
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
}
impl Location<f32> {
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
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
    pub fn split(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
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
    fn into(self: Self) -> Location<f64> {
        Location {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }
}
impl Into<Location<i64>> for Location<f64> {
    fn into(self: Self) -> Location<i64> {
        Location {
            x: self.x.round() as i64,
            y: self.y.round() as i64,
            z: self.z.round() as i64,
        }
    }
}
impl Into<Location<u128>> for Location<i64> {
    fn into(self: Self) -> Location<u128> {
        let abs = self.abs();
        Location {
            x: abs.x as u128,
            y: abs.y as u128,
            z: abs.z as u128,
        }
    }
}
#[derive(Debug)]
pub enum MoveType {
    Linear,
    Rapid,
}
impl Display for MoveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MoveType::Linear => write!(f, "Move linear"),
            MoveType::Rapid => write!(f, "Move rapid"),
        }
    }
}
