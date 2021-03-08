pub mod manual_spindel;
pub mod on_off_spindel;

pub use manual_spindel::ManualSpindel;
pub use on_off_spindel::OnOffSpindel;

pub trait Executer {
    /**
     * returns the time to wait to finish the operation
     */
    fn on(&mut self, speed: f64, cw: bool) -> Result<f64, &str>;
    /**
     * returns the time to wait to finish the operation
     */
    fn off(&mut self) -> Result<f64, &str>;
    /**
     * returns the time to wait to finish the operation
     */
    fn resume(&mut self) -> Result<f64, &str>;
    /**
     * returns the state of the executer
     */
    fn is_on(&self) -> bool;
}
