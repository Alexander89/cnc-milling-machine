use super::Executer;
use crate::io::Actor;

pub struct OnOffSpindel {
    gpio: Actor,
    switch_delay_s: f64,
}

impl OnOffSpindel {
    pub fn new(pin: u8, invert: bool, switch_delay_s: f64) -> OnOffSpindel {
        OnOffSpindel {
            gpio: Actor::new(pin, invert, invert),
            switch_delay_s,
        }
    }
}

impl Executer for OnOffSpindel {
    fn on(&mut self, _speed: f64, _cw: bool) -> Result<f64, &str> {
        self.gpio.set_high();
        if self.gpio.is_high() {
            Ok(0.0)
        } else {
            Ok(self.switch_delay_s)
        }
    }
    fn off(&mut self) -> Result<f64, &str> {
        self.gpio.set_low();
        if self.gpio.is_high() {
            Ok(self.switch_delay_s)
        } else {
            Ok(0.0)
        }
    }
    fn resume(&mut self) -> Result<f64, &str> {
        self.gpio.set_high();
        if self.gpio.is_high() {
            Ok(0.0)
        } else {
            Ok(self.switch_delay_s)
        }
    }
    fn is_on(&self) -> bool {
        self.gpio.is_high()
    }
}
