pub mod xbox_controller;
pub mod software_controller;

pub use xbox_controller::XBoxController;
pub use nes_controller::NesController;
//pub use software_controller::SoftwareController;

pub enum AuxCommand {
    CalibrateZ,
    Cancel,
    MoveAway,
    NextProgram,
    PrefProgram,
    ResetPos,
    Select,
    Start,
    ToggleMoveSpeed,
}

pub trait Controller {
    pub fn poll_input(&mut self) -> Option<Vec<AuxCommand>> {}
    pub fn get_move_if_changed(mut& self) -> Option<Location<f64>> {}
}
