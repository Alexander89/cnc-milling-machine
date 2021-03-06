use crate::motor::MotorSettings;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub dev_mode: bool,
    pub motor_x: MotorSettings,
    pub motor_y: MotorSettings,
    pub motor_z: MotorSettings,
    pub calibrate_z_gpio: Option<u8>,
    pub on_off_gpio: Option<u8>,
    pub switch_on_off_delay: f64,
    pub input_dir: Vec<String>,
    pub input_update_reduce: u32,
    pub default_speed: f64,
    pub rapid_speed: f64,
    pub scale: f64,
    pub invert_z: bool,
    pub show_console_output: bool,
    pub console_pos_update_reduce: u32,
    #[serde(default)]
    pub external_input_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dev_mode: false,
            motor_x: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 18,
                dir_gpio: 27,
                invert_dir: false,
                ena_gpio: None,
                end_left_gpio: Some(21),
                end_right_gpio: Some(20),
                step_size: 0.004f64,
                acceleration: 5.7f64,
                acceleration_damping: 0.0009f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,
            },
            motor_y: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 22,
                dir_gpio: 23,
                invert_dir: false,
                ena_gpio: None,
                end_left_gpio: Some(19),
                end_right_gpio: Some(26),
                step_size: 0.004f64,
                acceleration: 5.7f64,
                acceleration_damping: 0.0009f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,
            },
            motor_z: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 25,
                dir_gpio: 24,
                invert_dir: true,
                ena_gpio: None,
                end_left_gpio: Some(5),
                end_right_gpio: Some(6),
                step_size: 0.004f64,
                acceleration: 5.7f64,
                acceleration_damping: 0.0009f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,
            },
            calibrate_z_gpio: Some(16),
            on_off_gpio: Some(13),
            switch_on_off_delay: 3.5f64,
            input_dir: vec![String::from(".")],
            input_update_reduce: 10u32,
            default_speed: 360.0f64,
            rapid_speed: 720.0f64,
            scale: 1.0f64,
            invert_z: false,
            show_console_output: false,
            console_pos_update_reduce: 50u32,
            external_input_enabled: false,
        }
    }
}

impl Settings {
    pub fn from_file(file_path: &str) -> Settings {
        let file_path_2 = file_path.to_owned();
        let mut settings = if let Ok(data) = fs::read_to_string(&file_path_2) {
            serde_yaml::from_str(&data).unwrap()
        } else {
            let s = Settings::default();
            let data = serde_yaml::to_string(&s).unwrap();
            fs::write(file_path_2, data).expect("Unable to write file");
            s
        };

        let args = env::args();
        for arg in args {
            if arg == *"dev_mode" {
                settings.dev_mode = true;
            }
        }
        settings
    }
    pub fn write_to_file(&self, file_path: &str) -> Result<(), String> {
        let data = serde_yaml::to_string(self).unwrap();
        fs::write(file_path, data).map_err(|e| format!("{:?}", e))
    }
}
