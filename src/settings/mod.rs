use serde::{Deserialize, Serialize};
use std::{env, fs};

use crate::control::SettingsControl;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub dev_mode: bool,
    pub motor_x: SettingsMotor,
    pub motor_y: SettingsMotor,
    pub motor_z: SettingsMotor,
    pub calibrate_z_gpio: Option<u8>,
    pub on_off_gpio: Option<u8>,
    pub on_off_switch_delay: f64,
    pub on_off_invert: bool,
    pub input_dir: Vec<String>,
    pub default_speed: f64,
    pub rapid_speed: f64,
    pub scale: f64,
    pub invert_z: bool,
    pub pos_update_every_x_sec: f64,
    pub external_input_enabled: bool,
    pub ui: SettingsUi,
    pub control: SettingsControl,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dev_mode: false,
            motor_x: SettingsMotor {
                max_step_speed: 200,
                step_size: 0.004f64,
                acceleration: 1.0f64,
                deceleration: 1.0f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,

                driver_settings: DriverType::Stepper(StepperDriverSettings {
                    pull_gpio: 18,
                    dir_gpio: 27,
                    invert_dir: false,
                    ena_gpio: None,
                    end_left_gpio: Some(21),
                    end_right_gpio: Some(20),
                }),
            },
            motor_y: SettingsMotor {
                max_step_speed: 200,
                step_size: 0.004f64,
                acceleration: 1.0f64,
                deceleration: 1.0f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,
                driver_settings: DriverType::Stepper(StepperDriverSettings {
                    pull_gpio: 22,
                    dir_gpio: 23,
                    invert_dir: false,
                    ena_gpio: None,
                    end_left_gpio: Some(19),
                    end_right_gpio: Some(26),
                }),
            },
            motor_z: SettingsMotor {
                max_step_speed: 200,
                step_size: 0.004f64,
                acceleration: 1.0f64,
                deceleration: 1.0f64,
                free_step_speed: 20.0f64,
                acceleration_time_scale: 2.0f64,
                driver_settings: DriverType::Stepper(StepperDriverSettings {
                    pull_gpio: 25,
                    dir_gpio: 24,
                    invert_dir: true,
                    ena_gpio: None,
                    end_left_gpio: Some(5),
                    end_right_gpio: Some(6),
                }),
            },
            calibrate_z_gpio: Some(16),
            on_off_gpio: Some(13),
            on_off_switch_delay: 3.5f64,
            on_off_invert: false,
            input_dir: vec![String::from(".")],
            default_speed: 360.0f64,
            rapid_speed: 720.0f64,
            scale: 1.0f64,
            invert_z: false,
            pos_update_every_x_sec: 0.2f64,
            external_input_enabled: false,
            ui: SettingsUi::default(),
            control: SettingsControl::default()
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsUi {
    pub console: bool,
    pub web: bool,
}
impl Default for SettingsUi {
    fn default() -> Self {
        Self {
            console: true,
            web: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DriverType {
    Stepper(StepperDriverSettings),
    Mock,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StepperDriverSettings {
    pub pull_gpio: u8,
    pub dir_gpio: u8,
    pub invert_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ena_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_left_gpio: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_right_gpio: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsMotor {
    pub driver_settings: DriverType,
    pub max_step_speed: u32,
    pub step_size: f64,
    // acceleration constance
    pub acceleration: f64,
    // deceleration constance
    pub deceleration: f64,
    // reduce the acceleration on higher speed,
    // pub acceleration_damping: f64,
    // speed that requires no acceleration. It just runs with that.
    pub free_step_speed: f64,
    // value to adjust UI Graph
    pub acceleration_time_scale: f64,
}

impl Settings {
    pub fn from_file(file_path: &str) -> Settings {
        let file_path_2 = file_path.to_owned();
        let mut settings = if let Ok(data) = fs::read_to_string(&file_path_2) {
            serde_yaml::from_str(&data).unwrap()
        } else {
            let s = Settings::default();
            s.write_to_file(file_path).expect("can not save settings to disk");
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
