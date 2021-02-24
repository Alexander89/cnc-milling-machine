use super::App;

use crate::gnc::{Gnc, NextInstruction};
use crate::motor::task::CalibrateType;
use crate::types::{Location, MachineState};
use crate::ui::{
    types::{Mode, WsCommandsFrom, WsControllerMessage, WsMessages, WsPositionMessage},
    ui_main,
};

use crossbeam_channel::{Receiver, Sender};
use gilrs::{Axis, Button, Event, EventType};
use std::{thread, time::Duration};

impl App {
    pub fn run(&mut self, data_receiver: Receiver<WsMessages>, cmd_sender: Sender<WsCommandsFrom>) {
        let (update_path, new_progs) = self.start_file_watcher();

        // create Http-Server for the UI
        let pos_msg = WsPositionMessage::new(0.0f64, 0.0f64, 0.0f64);
        let status_msg = self.get_status_msg();
        let controller_msg = WsControllerMessage::new(&Location::default(), false, false, false);
        self.pool.spawn_ok(async {
            ui_main(
                cmd_sender,
                data_receiver,
                pos_msg,
                status_msg,
                controller_msg,
            )
            .expect("could not start WS-server");
        });

        // initial output
        println!("rusty cnc controller started\n access the UI with http://localhost:1506");
        if self.settings.show_console_output {
            println!("Found programs in you input_path:");
            for (i, p) in self.available_progs.iter().enumerate() {
                println!("{}: {}", i, p);
            }
        }

        let mut last = Location::default();
        'running: loop {
            thread::sleep(Duration::new(0, 5_000_000));

            // display position, or send it to the ws client
            self.display_counter += 1;
            if self.display_counter >= self.settings.console_pos_update_reduce {
                let pos = self.cnc.get_pos();
                if last != pos {
                    self.send_pos_msg(&pos);
                    if self.settings.show_console_output {
                        println!("  {{ x: {}, y: {}, z: {} }},", pos.x, pos.y, pos.z);
                    }
                    last = pos;
                }
                self.display_counter = 0;
            }

            // poll current mode of the cnc
            let ok = match self.current_mode {
                Mode::Program => self.program_mode(),
                Mode::Calibrate => self.calibrate_mode(),
                _ => self.manual_mode(),
            };
            if !ok {
                println!("terminate program");
                break 'running;
            }

            if let Ok(p) = new_progs.try_recv() {
                self.set_available_programs(p);
            }

            self.handle_network_commands(&update_path);
        }
    }

    pub fn apply_control(&mut self, control: Location<f64>) {
        if self.last_control != control {
            self.cnc
                .manual_move(control.x, control.y, control.z, 1440.0);
            self.last_control = control;
            self.send_controller_msg();
        }
    }
    pub fn manual_mode(&mut self) -> bool {
        // controller just every n-th tick
        self.input_reduce += 1;
        if self.input_reduce < self.settings.input_update_reduce {
            return true;
        }
        self.input_reduce = 0;

        let mut control = self.last_control.clone();
        let speed = if self.slow_control { 2.5f64 } else { 10.0f64 };
        // map GamePad events to update the manual program or start a program
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::ButtonReleased(Button::Select, _) => {
                    // remove later to avoid killing the machine by mistake
                    return false;
                }
                EventType::ButtonReleased(Button::Start, _)
                | EventType::ButtonReleased(Button::Mode, _) => {
                    if let Some(sel_prog) = self.selected_program.to_owned() {
                        self.start_program(&sel_prog, false, 1.0);
                    } else {
                        self.error("No Program selected".to_string());
                    }
                }
                EventType::AxisChanged(Axis::LeftStickX, value, _) => {
                    if !self.freeze_x && value > 0.15 {
                        control.x = (value as f64 - 0.15) / 8.5 * -speed;
                    } else if !self.freeze_x && value < -0.15 {
                        control.x = (value as f64 + 0.15) / 8.5 * -speed;
                    } else {
                        control.x = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::LeftStickY, value, _) => {
                    if !self.freeze_y && value > 0.15 {
                        control.y = (value as f64 - 0.15) / 8.5 * speed;
                    } else if !self.freeze_y && value < -0.15 {
                        control.y = (value as f64 + 0.15) / 8.5 * speed;
                    } else {
                        control.y = 0.0;
                    }
                }
                EventType::AxisChanged(Axis::RightStickY, value, _) => {
                    if value > 0.15 {
                        control.z = (value as f64 - 0.15) / 8.5 * speed;
                    } else if value < -0.15 {
                        control.z = (value as f64 + 0.15) / 8.5 * speed;
                    } else {
                        control.z = 0.0;
                    }
                }
                // add cross to select a program
                EventType::ButtonPressed(dir @ Button::DPadDown, _)
                | EventType::ButtonPressed(dir @ Button::DPadUp, _) => {
                    match dir {
                        Button::DPadUp => {
                            if self.program_select_cursor <= 0 {
                                self.program_select_cursor = self.available_progs.len() as i32 - 1;
                            } else {
                                self.program_select_cursor -= 1;
                            }
                        }
                        Button::DPadDown => {
                            self.program_select_cursor += 1;

                            if self.program_select_cursor >= self.available_progs.len() as i32 {
                                self.program_select_cursor = 0;
                            }
                        }
                        _ => (),
                    };
                    if self.settings.show_console_output {
                        for (i, p) in self.available_progs.iter().enumerate() {
                            println!("{}: {}", i, p);
                        }
                        println!(
                            "select {} {}",
                            self.program_select_cursor,
                            self.available_progs
                                .get(
                                    self.program_select_cursor
                                        .min(self.available_progs.len() as i32)
                                        .max(0) as usize
                                )
                                .unwrap()
                        );
                    }
                }
                EventType::ButtonPressed(Button::South, _) => {
                    let selected = self
                        .available_progs
                        .get(
                            self.program_select_cursor
                                .min(self.available_progs.len() as i32)
                                .max(0) as usize,
                        )
                        .map(|p| p.to_owned());
                    self.set_selected_program(selected);
                    self.info(format!("select {:?}", self.selected_program));
                }
                EventType::ButtonPressed(Button::North, _) => {
                    let pos = self.cnc.get_pos();
                    if pos.x == 0.0 && pos.y == 0.0 {
                        self.info("reset all (x, y, z)".to_string());
                        self.cnc.reset();
                    } else {
                        self.info("reset only plane move (x, y) -- Reset again without moving to reset the z axis as well".to_string());
                        self.cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                    }
                }
                EventType::ButtonPressed(Button::East, _) => {
                    self.info("calibrate".to_string());
                    self.cnc.calibrate(
                        CalibrateType::None,
                        CalibrateType::None,
                        CalibrateType::ContactPin,
                    );
                    self.set_current_mode(Mode::Calibrate);
                    self.in_opp = true;
                    thread::sleep(Duration::new(0, 10_000_000));
                }
                _ => {}
            }
        }

        self.apply_control(control);

        true
    }
    pub fn program_mode(&mut self) -> bool {
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.info("Cancel current job".to_string());
                self.set_current_mode(Mode::Manual);
                if self.cnc.cancel_task().is_err() {
                    self.error("cancel did not work".to_string());
                    panic!("cancel did not work!");
                };
            }
        }
        if let Some(prog) = self.prog.as_mut() {
            for next_instruction in prog {
                match next_instruction {
                    NextInstruction::Movement(next_movement) => {
                        self.cnc.query_g_task(next_movement);
                    }
                    NextInstruction::Miscellaneous(next_movement) => {
                        self.cnc.query_m_task(next_movement);
                    }
                    NextInstruction::NotSupported(err) => {
                        println!("NotSupported {:?}", err);
                        //self.warning(format!("NotSupported {:?}", err));
                    }
                    NextInstruction::InternalInstruction(err) => {
                        println!("InternalInstruction {:?}", err);
                        //self.info(format!("InternalInstruction {:?}", err));
                    }
                    _ => {}
                };
            }
            thread::sleep(Duration::new(0, 100_000_000));

            if self.cnc.get_state() == MachineState::Idle {
                self.set_current_mode(Mode::Manual);
                self.in_opp = false;
            }
        }

        self.set_prog_state(self.cnc.get_steps_todo(), self.cnc.get_steps_done());

        true
    }
    pub fn calibrate_mode(&mut self) -> bool {
        while let Some(Event { event, .. }) = self.gilrs.next_event() {
            if let EventType::ButtonReleased(Button::Select, _) = event {
                self.set_current_mode(Mode::Manual);
                if self.cnc.cancel_task().is_err() {
                    self.error("cancel did not work".to_string());
                    panic!("cancel did not work!");
                };
            }
        }

        if self.cnc.get_state() == MachineState::Idle {
            let calibrate_hight = Location {
                x: 0.0f64,
                y: 0.0f64,
                z: if self.settings.invert_z {
                    -20.0f64
                } else {
                    20.0f64
                },
            };
            self.cnc.set_pos(calibrate_hight);
            self.set_current_mode(Mode::Manual);
            self.calibrated = true;
            self.in_opp = false;
        }

        true
    }
    pub fn start_program(&mut self, program_name: &str, invert_z: bool, scale: f64) -> bool {
        if !self.calibrated {
            self.warning("start program without calibration".to_string());
        }
        self.set_selected_program(Some(program_name.to_owned()));
        if let Ok(load_prog) = Gnc::new(
            &program_name,
            5.0,
            50.0,
            scale,
            self.cnc.get_pos(),
            invert_z,
        ) {
            println!("commands found {:?}", load_prog.len());
            self.prog = Some(load_prog);
            self.set_current_mode(Mode::Program);
            true
        } else {
            self.error("program is not able to load".to_string());
            false
        }
    }
    pub fn cancel_program(&mut self) {
        self.set_selected_program(None);
        self.set_current_mode(Mode::Manual);
        if self.cnc.cancel_task().is_err() {
            self.error("cancel did not work".to_string());
            panic!("cancel did not work!");
        };
    }
}
