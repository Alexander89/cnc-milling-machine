pub struct App;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout};
use crossbeam_channel::{unbounded, Sender, Receiver};

use crate::{control::{ControlCommands, UserControlInput, Control}, hardware_controller::{
        InstructionCalibrate,
        InstructionManualMovement,
        HardwareFeedback,
        PosData,
        instruction::{
            CalibrateType,
            Instruction
        }
    }, settings::Settings
};

use crate::hardware_controller_interface::HardwareControllerInterface;
use super::hardware_controller::SettingsHardwareController;
use super::ui::UI;

const SETTINGS_PATH: &str = "./settings.yaml";

pub enum SystemEvents {
    HardwareInstruction(Instruction),
    HardwareFeedback(HardwareFeedback),
    ControlInput(UserControlInput),
    ControlCommands(ControlCommands),
    Terminate,
    //UI(UiMessages),
    //UICommands(UiCommandsFrom),
}

pub type SystemPublisher = Sender<SystemEvents>;
pub type SystemSubscriber = Receiver<SystemEvents>;

impl App {
    pub fn start() {
        // create global event bus
        let (event_publish, event_subscribe) = unbounded::<SystemEvents>();

        let settings = Settings::from_file(SETTINGS_PATH);
        let out = Arc::new(Mutex::new(stdout().into_raw_mode().unwrap()));

        // create modules

        let mut hardware_modul = HardwareControllerInterface::new(event_publish.clone(), event_subscribe.clone(), SettingsHardwareController::from(settings));
        let controller_modul = Control::new(event_publish.clone(), event_subscribe.clone(), settings.control);
        let ui_modul = UI::new(settings.ui);

        // wait before flushing the
        thread::sleep(Duration::from_millis(200));


        'main: loop {
            while let Ok(event) = event_subscribe.try_recv() {
                match event {
                    SystemEvents::Terminate => {
                        println!("terminate");
                        break 'main;
                    },
                    _ => ()
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        out.lock().unwrap().suspend_raw_mode().unwrap();
        println!("");
    }
}
