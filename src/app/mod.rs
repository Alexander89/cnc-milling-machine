pub struct App;

use crate::hardware_controller::{
    instruction::{CalibrateType, Instruction},
    InstructionCalibrate,
};

use super::hardware_controller::{HardwareControllerInterface, SettingsHardwareController};

impl App {
    pub fn start() -> App {
        let mut hardware = HardwareControllerInterface::new(SettingsHardwareController::mock());

        let feedback = hardware.get_feedback_channel();
        hardware
            .enqueue_instruction(Instruction::Calibrate(InstructionCalibrate {
                x: CalibrateType::Middle,
                y: CalibrateType::Middle,
                z: CalibrateType::Middle,
            }))
            .expect("can not enqueue");

        loop {
            println!("fb: {:?}", feedback.recv().unwrap());
        }
    }
}
