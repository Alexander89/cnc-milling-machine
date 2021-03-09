use std::{
    io::{Stdout, Write},
    sync::{
        Arc,
        Mutex,
        mpsc::{Receiver, SendError, channel}
    },
    thread,
    time::Duration
};

use crate::types::Location;

mod xbox_controller;
mod console_controller;

use termion::raw::RawTerminal;
pub use xbox_controller::XBoxController;
pub use console_controller::ConsoleController;

pub trait Control {
    fn poll(&mut self, op_state: ControlState) -> Result<ControlState, SendError<UserControlInput>>;
}

pub fn init_control(stdout: Arc<Mutex<RawTerminal<Stdout>>>) -> Receiver<UserControlInput> {
    let (sender, receiver) = channel();
    let inner_stdout = stdout.clone();
    thread::spawn(move || {
        let mut ctl = ConsoleController::new(sender.clone());
        let mut xbox_res = XBoxController::new(sender.clone());
        if xbox_res.is_err() {
            write!(inner_stdout.lock().unwrap(),"{}{}",termion::cursor::Goto(1, 6), termion::clear::CurrentLine).unwrap();
            stdout.lock().unwrap().flush().unwrap();
        }
        let mut op_state = ControlState::default();
        loop {
            thread::sleep(Duration::from_millis(100));
            match xbox_res {
                Ok(ref mut xbox) => {
                    match xbox.poll(op_state) {
                        Ok(new_state) => op_state = new_state,
                        Err(e) => {
                            write!(
                                inner_stdout.lock().unwrap(),
                                "{}{} shutdown controller thread {:?}",
                                termion::cursor::Goto(1, 6),
                                termion::clear::CurrentLine,
                                e
                            ).unwrap();
                            break;
                        },
                    }
                }
                _ => (),
            };
            match ctl.poll(op_state) {
                Ok(new_state) => op_state = new_state,
                Err(e) => {
                    write!(
                        inner_stdout.lock().unwrap(),
                        "{}{} shutdown controller thread {:?}",
                        termion::cursor::Goto(1, 6),
                        termion::clear::CurrentLine,
                        e
                    ).unwrap();
                    break;
                },
            }

            inner_stdout.lock().unwrap().flush().unwrap();
        }
        write!(
            inner_stdout.lock().unwrap(),
            "{}{} controllerThread: terminate",
            termion::cursor::Goto(1, 6),
            termion::clear::CurrentLine
        ).unwrap();
        drop(ctl);
    });
    receiver
}

#[derive(Debug, Clone)]
pub struct ControlState {
    manual_move_enabled: bool,
    freeze_x: bool,
    freeze_y: bool,
    freeze_z: bool,
}

impl Default for ControlState {
    fn default() -> Self {
        Self {
            manual_move_enabled: true,
            freeze_x: false,
            freeze_y: false,
            freeze_z: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UserControlInput {
    Terminate,
    Stop,
    Start,
    SelectProgram,
    NextProgram,
    PrefProgram,
    CalibrateZ,
    ResetPosToNull,
    ManualControl(Location<f64>),
}

#[derive(Debug, Clone)]
pub struct SettingsControl {
    input_update_reduce: u32
}
