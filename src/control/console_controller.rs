use crossbeam_channel::SendError;
use crate::{app::{SystemEvents, SystemPublisher}, types::Location};
use super::{Controller, ControlState, UserControlInput};

use termion::{event::Key};
use termion::input::TermRead;

use std::io::stdin;

#[derive(Debug, Clone)]
pub struct ConsoleController {
    last_control: Location<f64>,
    event_publish: SystemPublisher
}

impl ConsoleController {
    pub fn new(event_publish: SystemPublisher) -> Self {
        ConsoleController {
            last_control: Location::default(),
            event_publish,
        }
    }
}
impl Controller for ConsoleController {
    fn poll(&mut self, op_state: ControlState) -> Result<ControlState, SendError<SystemEvents>> {
        let mut control = self.last_control.clone();

        while let Some(Ok(key)) = stdin().keys().next() {
            match key {
                Key::Char('s') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::Start))?,
                Key::Char('q') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::Stop))?,
                Key::Char('z') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::CalibrateZ))?,
                Key::Char('r') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::ResetPosToNull))?,
                Key::Char('y') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::PrefProgram))?,
                Key::Char('x') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::NextProgram))?,
                Key::Char(' ') => self.event_publish.send(SystemEvents::ControlInput(UserControlInput::SelectProgram))?,
                Key::Ctrl('c') | Key::Esc => {
                    self.event_publish.send(SystemEvents::Terminate)?;
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
                self.event_publish.send(SystemEvents::ControlInput(UserControlInput::ManualControl(control.clone())))?;
                self.last_control = control.clone();
            }
        }
        Ok(op_state)
    }
}
