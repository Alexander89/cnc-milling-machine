pub struct App;
use std::{io::{Stdout, Write}, sync::{Arc, Mutex}, thread, time::Duration};
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::stdout;
use crossbeam_channel::{unbounded, Sender, Receiver};

use crate::{control::{ControlCommands, UserControlInput, Control}, hardware_controller::{
        HardwareFeedback,
        instruction::{
            Instruction
        }
    }, settings::Settings
};

use crate::hardware_controller_interface::HardwareControllerInterface;
use super::hardware_controller::SettingsHardwareController;
use super::ui::Ui;

const SETTINGS_PATH: &str = "./settings.yaml";
pub enum SystemEvents {
    #[allow(dead_code)]
    HardwareInstruction(Instruction),
    HardwareFeedback(HardwareFeedback),
    ControlInput(UserControlInput),
    #[allow(dead_code)]
    ControlCommands(ControlCommands),
    Terminate,
    //UI(UiMessages),
    //UICommands(UiCommandsFrom),
}

pub type SystemPublisher = Sender<SystemEvents>;
pub type SystemSubscriber = Receiver<SystemEvents>;
pub type ConsoleOut = Arc<Mutex<RawTerminal<Stdout>>>;

impl App {
    pub fn start() {
        // create global event bus
        let (event_publish, event_subscribe) = unbounded::<SystemEvents>();

        let settings = Settings::from_file(SETTINGS_PATH);
        let out: ConsoleOut = Arc::new(Mutex::new(stdout().into_raw_mode().unwrap()));

        // create modules

        HardwareControllerInterface::new(event_publish.clone(), event_subscribe.clone(), SettingsHardwareController::from(settings.clone()));
        Control::new(event_publish.clone(), event_subscribe.clone(), settings.control.clone());
        Ui::new(event_publish.clone(), event_subscribe.clone(), out.clone(), settings.ui.clone());

        // wait before flushing the
        thread::sleep(Duration::from_millis(1000));
        out.lock().unwrap().flush().unwrap();

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
            thread::sleep(Duration::from_millis(250));
        }
        thread::sleep(Duration::from_millis(1000));
        out.lock().unwrap().suspend_raw_mode().unwrap();
        println!("");
    }
}
