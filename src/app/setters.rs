#![allow(clippy::option_map_unit_fn)]
use super::App;

use crate::ui::types::{
    Mode, WsCommandSettingsSetRuntimeSettings, WsCommandSettingsSetSystemSettings,
};

use super::SETTINGS_PATH;

use std::sync::mpsc;

impl App {
    pub fn set_prog_state(&mut self, todo: i64, done: i64) {
        if self.steps_todo != todo || self.steps_done != done {
            self.steps_todo = todo;
            self.steps_done = done;
            self.send_status_msg();
        }
    }
    pub fn set_current_mode(&mut self, mode: Mode) {
        self.current_mode = mode;
        self.send_status_msg();
    }
    pub fn set_available_programs(&mut self, available_progs: Vec<String>) {
        self.available_progs = available_progs;
        self.send_available_program_msg();
    }
    pub fn set_selected_program(&mut self, selected_program: Option<String>) {
        self.selected_program = selected_program;
        self.send_status_msg();
    }
    pub fn set_runtime_settings(
        &mut self,
        settings: WsCommandSettingsSetRuntimeSettings,
        update_path: &mpsc::Sender<Vec<String>>,
    ) -> Result<(), String> {
        settings.input_dir.map(|v| {
            self.settings.input_dir = v.clone();
            update_path.send(v)
        });
        settings
            .input_update_reduce
            .map(|v| self.settings.input_update_reduce = v);
        settings
            .default_speed
            .map(|v| self.settings.default_speed = v);
        settings.rapid_speed.map(|v| self.settings.rapid_speed = v);
        settings.scale.map(|v| self.settings.scale = v);
        settings.invert_z.map(|v| self.settings.invert_z = v);
        settings
            .show_console_output
            .map(|v| self.settings.show_console_output = v);
        settings
            .console_pos_update_reduce
            .map(|v| self.settings.console_pos_update_reduce = v);

        self.settings.write_to_file(SETTINGS_PATH)
    }
    pub fn set_system_settings(
        &mut self,
        settings: WsCommandSettingsSetSystemSettings,
    ) -> Result<(), String> {
        self.settings.dev_mode = settings.dev_mode;
        self.settings.motor_x = settings.motor_x;
        self.settings.motor_y = settings.motor_y;
        self.settings.motor_z = settings.motor_z;
        self.settings.calibrate_z_gpio = settings.calibrate_z_gpio;
        self.settings.on_off_gpio = settings.on_off_gpio;
        self.settings.switch_on_off_delay = settings.switch_on_off_delay;

        self.settings.write_to_file(SETTINGS_PATH)
    }
}
