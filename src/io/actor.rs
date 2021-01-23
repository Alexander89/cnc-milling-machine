#![allow(dead_code)]
use rppal::gpio::{Gpio, Level, OutputPin};

#[derive(Debug)]
pub struct Actor {
    pin: u8,
    invert_output: bool,
    gpio: Option<OutputPin>,
    level: Level,
}

impl Actor {
    pub fn new(pin: u8, invert_output: bool, default: bool) -> Actor {
        let gpio = Gpio::new()
            .ok()
            .and_then(|new_gpio| new_gpio.get(pin).ok().map(|io| io.into_output()));
        if gpio.is_none() {
            println!("> failed to connect actor GPIO for pin {}", pin);
        }
        let mut actor = Actor {
            pin,
            invert_output,
            gpio,
            level: if default { Level::High } else { Level::Low },
        };
        actor.set_to(default);
        actor
    }
    fn set_to(&mut self, value: bool) {
        let lvl = if value != self.invert_output {
            Level::High
        } else {
            Level::Low
        };
        if let Some(out) = self.gpio.as_mut() {
            out.write(lvl)
        }
        self.level = lvl;
    }
    pub fn set_high(&mut self) {
        self.set_to(true)
    }
    pub fn set_low(&mut self) {
        self.set_to(false)
    }
    pub fn is_high(&self) -> bool {
        (self.level == Level::High) != self.invert_output
    }
    pub fn is_low(&self) -> bool {
        (self.level == Level::Low) != self.invert_output
    }
}
