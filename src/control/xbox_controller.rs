use std::sync::mpsc::{SendError, Sender};
use gilrs::{Axis, Button, Event, EventType, Gilrs};
use crate::{types::Location};
use super::{Control, ControlState, UserControlInput};

#[derive(Debug)]
pub struct XBoxController {
    gilrs: Gilrs,
    last_control: Location<f64>,
    sender: Sender<UserControlInput>,
}

impl XBoxController {
    pub fn new(sender: Sender<UserControlInput>) -> Result<Self, &'static str> {
        let gilrs = Gilrs::new()
            .map_err(|_| "gamepad not valid")
            .expect("controller is missing");

        if !XBoxController::gamepad_connected(&gilrs) {
            Err("no gamepad connected")
        } else {
            Ok(XBoxController {
                gilrs,
                last_control: Location::default(),
                sender,
            })
        }

    }

    fn gamepad_connected(gilrs: &Gilrs) -> bool {
        let mut gamepad_found = false;
        for (_id, gamepad) in gilrs.gamepads() {
            println!("{} is {:?}", gamepad.name(), gamepad.power_info());
            gamepad_found = true;
        }
        gamepad_found
    }
}
impl Control for XBoxController {
    fn poll(&mut self, op_state: ControlState) -> Result<ControlState, SendError<UserControlInput>> {
        let mut control = self.last_control.clone();
        // map GamePad events to update the manual program or start a program
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    if !op_state.freeze_x && value > 0.15 {
                        control.x = (value as f64 - 0.15) / 8.5 * -10.0;
                    } else if !op_state.freeze_x && value < -0.15 {
                        control.x = (value as f64 + 0.15) / 8.5 * -10.0;
                    } else {
                        control.x = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    if !op_state.freeze_y && value > 0.15 {
                        control.y = (value as f64 - 0.15) / 8.5 * 10.0;
                    } else if !op_state.freeze_y && value < -0.15 {
                        control.y = (value as f64 + 0.15) / 8.5 * 10.0;
                    } else {
                        control.y = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::RightStickY, value, _) => {
                    if !op_state.freeze_z && value > 0.15 {
                        control.z = (value as f64 - 0.15) / 8.5 * 10.0;
                    } else if !op_state.freeze_z && value < -0.15 {
                        control.z = (value as f64 + 0.15) / 8.5 * 10.0;
                    } else {
                        control.z = 0.0;
                    }
                }

                EventType::ButtonReleased(Button::Select, _) => {
                    // remove later to avoid killing the machine by mistake
                    self.sender.send(UserControlInput::Terminate)?;
                    return Ok(op_state);
                }
                EventType::ButtonReleased(Button::Start, _) => {
                    self.sender.send(UserControlInput::Start)?;
                }
                EventType::ButtonReleased(Button::Mode, _) => {
                    self.sender.send(UserControlInput::Stop)?;
                }
                // add cross to select a program
                EventType::ButtonPressed(Button::DPadDown, _)=> {
                    self.sender.send(UserControlInput::NextProgram)?
                }
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    self.sender.send(UserControlInput::PrefProgram)?
                }
                EventType::ButtonPressed(Button::South, _) => {
                    self.sender.send(UserControlInput::SelectProgram)?;
                }
                EventType::ButtonPressed(Button::North, _) => {
                    self.sender.send(UserControlInput::ResetPosToNull)?;
                }
                EventType::ButtonPressed(Button::East, _) => {
                    self.sender.send(UserControlInput::CalibrateZ)?;
                }
                _ => {}
            }
        }
        if control != self.last_control {
            self.sender.send(UserControlInput::ManualControl(control.clone()))?;
            self.last_control = control;
        }
        Ok(op_state)
    }
}
