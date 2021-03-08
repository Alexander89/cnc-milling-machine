pub struct App;

use crate::{hardware_controller::{
    instruction::{CalibrateType, Instruction},
    InstructionCalibrate,
}, settings::Settings};

use super::hardware_controller::{HardwareControllerInterface, SettingsHardwareController};

const SETTINGS_PATH: &str = "./settings.yaml";

impl App {
    pub fn start() -> App {

        let settings = Settings::from_file(SETTINGS_PATH);

        let mut hardware = HardwareControllerInterface::new(SettingsHardwareController::from(settings));

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
