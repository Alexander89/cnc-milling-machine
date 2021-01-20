use super::App;

use crate::motor::task::CalibrateType;
use crate::program::{NextInstruction, Program};
use crate::types::{Location, MachineState};
use crate::ui::{
    types::{
        Mode, WsCommandController, WsCommandProgram, WsCommandSettings, WsCommands, WsCommandsFrom,
        WsControllerMessage, WsMessages, WsPositionMessage,
    },
    ui_main,
};

use gilrs::{Axis, Button, Event, EventType};
use crossbeam_channel::{Receiver, Sender};
use std::{
    fs::{remove_file, File},
    io::prelude::*,
    path::Path,
    thread,
    time::Duration,
};

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

            // handle incoming commands
            if let Ok(WsCommandsFrom(uuid, cmd)) = self.ui_cmd_receiver.try_recv() {
                match cmd {
                    WsCommands::Controller(WsCommandController::FreezeX { freeze }) => {
                        if self.freeze_x != freeze {
                            self.freeze_x = freeze;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Controller(WsCommandController::FreezeY { freeze }) => {
                        if self.freeze_y != freeze {
                            self.freeze_y = freeze;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Controller(WsCommandController::Slow { slow }) => {
                        if self.slow_control != slow {
                            self.slow_control = slow;
                            self.send_controller_msg();
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Get) => {
                        self.send_available_programs_msg(uuid)
                    }
                    WsCommands::Program(WsCommandProgram::Load { program_name }) => {
                        self.send_program_data_msg(uuid, program_name)
                    }
                    WsCommands::Program(WsCommandProgram::Start {
                        program_name,
                        invert_z,
                        scale,
                    }) => {
                        if self.start_program(&program_name, invert_z, scale) {
                            self.send_start_reply_message(uuid, program_name)
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Cancel) => {
                        self.cancel_program();
                        self.send_cancel_reply_message(uuid, true);
                    }
                    WsCommands::Program(WsCommandProgram::Save {
                        program_name,
                        program,
                    }) => {
                        if Path::new(&program_name).exists() {
                            match File::open(program_name.clone()) {
                                Err(why) => {
                                    self.info(format!("couldn't open {}: {}", program_name, why));
                                    self.send_save_reply_message(uuid, program_name, false);
                                }
                                Ok(mut file) => {
                                    match file.write_all(program.as_bytes()) {
                                        Err(why) => {
                                            self.info(format!(
                                                "couldn't write to {}: {}",
                                                program_name, why
                                            ));
                                            self.send_save_reply_message(uuid, program_name, false);
                                        }
                                        Ok(_) => {
                                            self.send_save_reply_message(uuid, program_name, true)
                                        }
                                    };
                                }
                            }
                        } else {
                            match File::create(&program_name) {
                                Err(why) => {
                                    self.info(format!(
                                        "couldn't write to {}: {}",
                                        program_name, why
                                    ));
                                    self.send_save_reply_message(uuid, program_name, true);
                                }
                                Ok(mut file) => self.send_save_reply_message(
                                    uuid,
                                    program_name,
                                    file.write_all(program.as_bytes()).is_ok(),
                                ),
                            }
                        }
                    }
                    WsCommands::Program(WsCommandProgram::Delete { program_name }) => {
                        self.send_delete_reply_message(
                            uuid,
                            program_name.clone(),
                            remove_file(program_name).is_ok(),
                        );
                    }
                    WsCommands::Settings(WsCommandSettings::GetRuntime) => {
                        self.send_runtime_settings_reply_message(uuid);
                    }
                    WsCommands::Settings(WsCommandSettings::SetRuntime(settings)) => {
                        match self.set_runtime_settings(settings, &update_path) {
                            Ok(()) => self.send_runtime_settings_saved_reply_message(uuid, true),
                            Err(_) => self.send_runtime_settings_saved_reply_message(uuid, false),
                        };
                    }
                    WsCommands::Settings(WsCommandSettings::GetSystem) => {
                        self.send_system_settings_reply_message(uuid);
                    }
                    WsCommands::Settings(WsCommandSettings::SetSystem(settings)) => {
                        match self.set_system_settings(settings) {
                            Ok(()) => self.send_system_settings_saved_reply_message(uuid, true),
                            Err(_) => self.send_system_settings_saved_reply_message(uuid, false),
                        };
                    } //_ => (),
                };
            }
        }
    }

    pub fn apply_control(&mut self, control: Location<f64>) {
        if self.last_control != control {
            self.cnc.manual_move(control.x, control.y, control.z, 20.0);
            self.last_control = control.clone();
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
                    if !self.calibrated {
                        self.warning(format!("start program without calibration"));
                    }
                    if let Some(ref sel_prog) = self.selected_program {
                        if let Ok(load_prog) =
                            Program::new(sel_prog, 5.0, 50.0, 1.0, self.cnc.get_pos(), false)
                        {
                            println!("commands found {:?}", load_prog.len());
                            self.prog = Some(load_prog);
                            self.set_current_mode(Mode::Program);
                        } else {
                            self.error(format!("program is not able to load"));
                        };
                    } else {
                        self.error(format!("No Program selected"));
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
                        self.info(format!("reset all (x, y, z)"));
                        self.cnc.reset();
                    } else {
                        self.info(format!("reset only plane move (x, y) -- Reset again without moving to reset the z axis as well"));
                        self.cnc.set_pos(Location::new(0.0, 0.0, pos.z));
                    }
                }
                EventType::ButtonPressed(Button::West, _) => {
                    self.info(format!("calibrate"));
                    self.cnc.calibrate(
                        CalibrateType::None,
                        CalibrateType::None,
                        CalibrateType::ContactPin,
                    );
                    self.set_current_mode(Mode::Calibrate);
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
                self.info(format!("Cancel current job"));
                self.set_current_mode(Mode::Manual);
                if self.cnc.cancel_task().is_err() {
                    self.error(format!("cancel did not work"));
                    panic!("cancel did not work!");
                };
            }
        }
        if let Some(ref mut prog) = self.prog {
            for next_instruction in prog {
                match next_instruction {
                    NextInstruction::Movement(next_movement) => {
                        self.cnc.query_task(next_movement);
                    }
                    NextInstruction::Miscellaneous(task) => {
                        println!("Miscellaneous {:?}", task);
                        //self.info(format!("Miscellaneous {:?}", task));
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

            match (self.cnc.get_state(), self.in_opp) {
                (MachineState::Idle, true) => {
                    self.set_current_mode(Mode::Manual);
                    self.in_opp = false;
                }
                (MachineState::ProgramTask, false) => {
                    self.calibrated = false;
                    self.in_opp = true;
                }
                _ => (),
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
                    self.error(format!("cancel did not work"));
                    panic!("cancel did not work!");
                };
            }
        }
        match (self.cnc.get_state(), self.in_opp) {
            (MachineState::Idle, true) => {
                let calibrate_hight = Location {
                    x: 0.0f64,
                    y: 0.0f64,
                    z: 20.0f64,
                };
                self.cnc.set_pos(calibrate_hight);
                self.set_current_mode(Mode::Manual);
                self.calibrated = true;
                self.in_opp = false;
            }
            (MachineState::Calibrate, false) => {
                self.in_opp = true;
            }
            _ => (),
        }

        true
    }
    pub fn start_program(&mut self, program_name: &String, invert_z: bool, scale: f64) -> bool {
        if !self.calibrated {
            self.warning(format!("start program without calibration"));
        }
        self.set_selected_program(Some(program_name.to_owned()));
        if let Ok(load_prog) = Program::new(
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
            self.error(format!("program is not able to load"));
            false
        }
    }
    pub fn cancel_program(&mut self) {
        self.set_selected_program(None);
        self.set_current_mode(Mode::Manual);
        if self.cnc.cancel_task().is_err() {
            self.error(format!("cancel did not work"));
            panic!("cancel did not work!");
        };
    }
}
