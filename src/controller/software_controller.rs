struct HardwareController = {
    last_control_published: Location<f64>
    last_control: Location<f64>
}

impl Controller for  HardwareController{
    pub fn poll_input(&mut self) -> Option<Vec<AuxCommand>> {
        let mut auxKeys = vec!([]);

        let mut control = self.last_control.clone();
        // map GamePad events to update the manual program or start a program
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::ButtonReleased(Button::Select, _) => {
                    // remove later to avoid killing the machine by mistake
                    auxKeys.push(AuxCommand::Cancel);
                }
                EventType::ButtonReleased(Button::Start, _)
                | EventType::ButtonReleased(Button::Mode, _) => {
                    auxKeys.push(AuxCommand::Start);
                }
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    if value > 0.15 {
                        control.x = (value as f64 - 0.15) / 8.5 * -1;
                    } else if value < -0.15 {
                        control.x = (value as f64 + 0.15) / 8.5 * -1;
                    } else {
                        control.x = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    if value > 0.15 {
                        control.y = (value as f64 - 0.15) / 8.5;
                    } else if value < -0.15 {
                        control.y = (value as f64 + 0.15) / 8.5;
                    } else {
                        control.y = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::RightStickY, value, _) => {
                    if value > 0.15 {
                        control.z = (value as f64 - 0.15) / 8.5;
                    } else if value < -0.15 {
                        control.z = (value as f64 + 0.15) / 8.5;
                    } else {
                        control.z = 0.0;
                    }
                }
                // add cross to select a program
                EventType::ButtonPressed(Button::DPadDown, _)=> {
                    auxKeys.push(AuxCommand::NextProgram);
                }
                EventType::ButtonPressed(Button::DPadUp, _) => {
                    auxKeys.push(AuxCommand::PrefProgram);
                }
                EventType::ButtonPressed(Button::South, _) => {
                    auxKeys.push(AuxCommand::Select);
                }
                EventType::ButtonPressed(Button::North, _) => {
                    auxKeys.push(AuxCommand::ResetPos);
                }
                EventType::ButtonPressed(Button::East, _) => {
                    auxKeys.push(AuxCommand::CalibrateZ);
                }
                _ => {}
            }
        }
        auxKeys
    }
}
