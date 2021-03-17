use std::{thread, time::Duration};
use serde::{Deserialize, Serialize};
use crossbeam_channel::SendError;

use crate::{
    app::{SystemPublisher, SystemSubscriber, SystemEvents},
    types::Location
};

mod xbox_controller;
mod console_controller;

pub use xbox_controller::XBoxController;
pub use console_controller::ConsoleController;


// pub struct System {
//     sessions: HashMap<Uuid, Connection>,
//     sender: WsSender,
//     receiver: WsReceiver,
//     last_state: SystemState,
// }

#[derive(Debug)]
/// Define Control actor
pub struct Control {
    controller_thread: thread::JoinHandle<()>,
}

impl Control {
    pub fn new(event_publish: SystemPublisher, event_subscribe: SystemSubscriber, settings: SettingsControl) -> Control {
        let controller_thread = thread::spawn(move || {
            let mut controller: Vec<Box<dyn Controller>> = vec!();
            let mut ctl = ConsoleController::new(event_publish.clone());
            controller.push(Box::new(ctl));

            if let Ok(mut xbox_res) = XBoxController::new(event_publish.clone()) {
                controller.push(Box::new(xbox_res));
            }

            let mut op_state = ControlState::default();
            loop {
                thread::sleep(Duration::from_millis(100));
                if let Ok(SystemEvents::ControlCommands(cmd)) = event_subscribe.try_recv() {
                    match cmd {
                        ControlCommands::FreezeChannel(Channel::X, value) => op_state.freeze_x = value,
                        ControlCommands::FreezeChannel(Channel::Y, value) => op_state.freeze_y = value,
                        ControlCommands::FreezeChannel(Channel::Z, value) => op_state.freeze_z = value,
                        ControlCommands::MoveSpeed(MoveSpeed::Slow) => op_state.move_slow = true,
                        ControlCommands::MoveSpeed(_) => op_state.move_slow = false,
                    }
                }

                for ref mut ctrl in controller {
                    match ctl.poll(op_state) {
                        Ok(new_state) => op_state = new_state,
                        Err(e) => {
                            println!("shutdown controller thread {:?}", e);
                            break;
                        },
                    }
                }
            }
        });
        Self {
            controller_thread
        }
    }
}

pub trait Controller {
    fn poll(&mut self, op_state: ControlState) -> Result<ControlState, SendError<SystemEvents>>;
}

#[derive(Debug, Clone)]
pub struct ControlState {
    manual_move_enabled: bool,
    move_slow: bool,
    freeze_x: bool,
    freeze_y: bool,
    freeze_z: bool,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            manual_move_enabled: true,
            move_slow: false,
            freeze_x: false,
            freeze_y: false,
            freeze_z: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UserControlInput {
    Stop,
    Start,
    SelectProgram,
    NextProgram,
    PrefProgram,
    CalibrateZ,
    ResetPosToNull,
    ManualControl(Location<f64>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsControl {
    input_update_reduce: u32
}
impl Default for SettingsControl {
    fn default() -> Self {
        Self {
            input_update_reduce: 100u32,
        }
    }
}

#[derive(Debug, Clone)]
enum Channel { X, Y, Z }

#[derive(Debug, Clone)]
enum MoveSpeed { Slow, Medium, Rapid }

#[derive(Debug, Clone)]
pub enum ControlCommands {
    FreezeChannel(Channel, bool),
    MoveSpeed(MoveSpeed),
}
