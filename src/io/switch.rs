#![allow(dead_code)]
use rppal::gpio::{Gpio, InputPin, Level};

#[derive(Debug, Clone, PartialEq)]
pub enum SwitchState {
    OPEN,
    CLOSED,
}

#[derive(Debug)]
pub struct Switch {
    pin: u8,
    invert_input: bool,
    gpio: InputPin,
    level: Level,
}

impl Switch {
    pub fn new(pin: u8, invert_input: bool) -> Switch {
        let gpio = Gpio::new().unwrap().get(pin).unwrap().into_input();
        let state = gpio.read();
        Switch {
            pin,
            invert_input,
            gpio,
            level: state,
        }
    }
    pub fn poll(&mut self) {
        self.level = self.gpio.read();
    }
    pub fn is_open(&mut self) -> bool {
        self.get_state() == SwitchState::OPEN
    }
    pub fn is_closed(&mut self) -> bool {
        self.get_state() == SwitchState::CLOSED
    }
    pub fn get_state(&mut self) -> SwitchState {
        self.poll();
        match self.level {
            Level::High if self.invert_input => SwitchState::OPEN,
            Level::High if !self.invert_input => SwitchState::CLOSED,
            Level::Low if self.invert_input => SwitchState::CLOSED,
            Level::Low if !self.invert_input => SwitchState::OPEN,
            _ => panic!("level is not a level"),
        }
    }
}
