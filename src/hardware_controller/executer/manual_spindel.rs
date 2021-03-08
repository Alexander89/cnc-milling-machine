use super::Executer;

pub struct ManualSpindel {
    switch_delay_s: f64,
}

impl ManualSpindel {
    pub fn new(switch_delay_s: f64) -> ManualSpindel {
        ManualSpindel { switch_delay_s }
    }
}

impl Executer for ManualSpindel {
    fn on(&mut self, _speed: f64, _cw: bool) -> Result<f64, &str> {
        Ok(self.switch_delay_s)
    }
    fn off(&mut self) -> Result<f64, &str> {
        Ok(self.switch_delay_s)
    }
    fn resume(&mut self) -> Result<f64, &str> {
        Ok(self.switch_delay_s)
    }
    fn is_on(&self) -> bool {
        true
    }
}
