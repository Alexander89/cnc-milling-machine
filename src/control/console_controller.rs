use std::{sync::{mpsc::{SendError, Sender}}};
use crate::{types::Location};
use super::{Control, ControlState, UserControlInput};

use termion::{event::Key};
use termion::input::TermRead;

use std::io::stdin;

pub struct ConsoleController {
    last_control: Location<f64>,
    sender: Sender<UserControlInput>,
}

impl ConsoleController {
    pub fn new(sender: Sender<UserControlInput>) -> Self {
        ConsoleController {
            last_control: Location::default(),
            sender,
        }
    }
}
impl Control for ConsoleController {
    fn poll(&mut self, op_state: ControlState) -> Result<ControlState, SendError<UserControlInput>> {
        let mut control = self.last_control.clone();

        while let Some(Ok(key)) = stdin().keys().next() {
            match key {
                Key::Char('s') => self.sender.send(UserControlInput::Start)?,
                Key::Char('q') => self.sender.send(UserControlInput::Stop)?,
                Key::Char('z') => self.sender.send(UserControlInput::CalibrateZ)?,
                Key::Char('r') => self.sender.send(UserControlInput::ResetPosToNull)?,
                Key::Char('y') => self.sender.send(UserControlInput::PrefProgram)?,
                Key::Char('x') => self.sender.send(UserControlInput::NextProgram)?,
                Key::Char(' ') => self.sender.send(UserControlInput::SelectProgram)?,
                Key::Ctrl('c') | Key::Esc => {
                    self.sender.send(UserControlInput::Terminate)?;
                    return Ok(op_state);
                },

                Key::PageUp if !op_state.freeze_z && control.z == 1.0 => control.z = 0.0,
                Key::PageUp if !op_state.freeze_z => control.z = 1.0,

                Key::PageDown if !op_state.freeze_z && control.z == -1.0 => control.z = 0.0,
                Key::PageDown if !op_state.freeze_z => control.z = -1.0,

                Key::Left if !op_state.freeze_x && control.x != 1.0 => control.x = 1.0,
                Key::Left if !op_state.freeze_x => control.x = 0.0,

                Key::Right if !op_state.freeze_x && control.x != -1.0 => control.x = -1.0,
                Key::Right if !op_state.freeze_x => control.x = 0.0,

                Key::Up if !op_state.freeze_y && control.y != 1.0 => control.y = 1.0,
                Key::Up if !op_state.freeze_y => control.y = 0.0,

                Key::Down if !op_state.freeze_y && control.y != -1.0 => control.y = -1.0,
                Key::Down if !op_state.freeze_y => control.y = 0.0,
                _ => {}
            }
            if control != self.last_control {
                self.sender.send(UserControlInput::ManualControl(control.clone()))?;
                self.last_control = control.clone();
            }
        }
        Ok(op_state)
    }
}
