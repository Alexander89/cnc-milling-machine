use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorSettings {
    pub max_step_speed: u32,
    pub pull_gpio: u8,
    pub dir_gpio: u8,
    pub ena_gpio: Option<u8>,
    pub end_left_gpio: Option<u8>,
    pub end_right_gpio: Option<u8>,
    pub step_size: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub dev_mode: bool,
    pub motor_x: MotorSettings,
    pub motor_y: MotorSettings,
    pub motor_z: MotorSettings,
    pub calibrate_z_gpio: Option<u8>,
    pub input_dir: Vec<String>,
    pub input_update_reduce: u32,
    pub default_speed: f64,
    pub rapid_speed: f64,
    pub scaler: f64,
    pub invert_z: bool,
    pub show_console_output: bool,
    pub console_pos_update_reduce: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dev_mode: false,
            motor_x: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 18,
                dir_gpio: 27,
                ena_gpio: None,
                end_left_gpio: Some(21),
                end_right_gpio: Some(20),
                step_size: 0.004f64,
            },
            motor_y: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 22,
                dir_gpio: 23,
                ena_gpio: None,
                end_left_gpio: Some(19),
                end_right_gpio: Some(26),
                step_size: 0.004f64,
            },
            motor_z: MotorSettings {
                max_step_speed: 200,
                pull_gpio: 25,
                dir_gpio: 24,
                ena_gpio: None,
                end_left_gpio: Some(5),
                end_right_gpio: Some(6),
                step_size: 0.004f64,
            },
            calibrate_z_gpio: Some(16),
            input_dir: vec![String::from(".")],
            input_update_reduce: 10u32,
            default_speed: 5.0f64,
            rapid_speed: 50.0f64,
            scaler: 1.0f64,
            invert_z: false,
            show_console_output: false,
            console_pos_update_reduce: 50u32,
        }
    }
}

impl Settings {
    pub fn from_file(file_path: &str) -> Settings {
        let file_path_2 = file_path.clone();
        let mut settings = if let Ok(data) = fs::read_to_string(file_path_2) {
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
}
