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
    gpio: Option<InputPin>,
    level: Level,
}

impl Switch {
    pub fn new(pin: u8, invert_input: bool) -> Switch {
        let gpio = Gpio::new()
            .ok()
            .and_then(|new_gpio| new_gpio.get(pin).ok().map(|io| io.into_input()));
        if gpio.is_none() {
            println!("> failed to connect switch GPIO for pin {}", pin);
        }

        let mut switch = Switch {
            pin,
            invert_input,
            gpio,
            level: Level::Low,
        };
        switch.poll();
        switch
    }
    pub fn poll(&mut self) {
        self.level = self
            .gpio
            .as_ref()
            .map(|input| input.read())
            .unwrap_or(if self.invert_input {
                Level::Low
            } else {
                Level::High
            });
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
