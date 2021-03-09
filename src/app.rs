pub struct App;

use std::{sync::{Arc, Mutex, mpsc::SendError}, thread, time::Duration};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout};

use crate::{
    control::{init_control, UserControlInput},
    hardware_controller::{
        InstructionCalibrate,
        InstructionManualMovement,
        HardwareFeedback,
        PosData,
        instruction::{
            CalibrateType,
            Instruction
        }
    },
    settings::Settings
};

use super::hardware_controller::{HardwareControllerInterface, SettingsHardwareController};

const SETTINGS_PATH: &str = "./settings.yaml";

impl App {
    pub fn start() -> Result<(), SendError<Instruction>> {
        let out = Arc::new(Mutex::new(stdout().into_raw_mode().unwrap()));

        let settings = Settings::from_file(SETTINGS_PATH);
        let mut hardware = HardwareControllerInterface::new(SettingsHardwareController::from(settings));
        let controller_recv = init_control(out.clone());

        let feedback = hardware.get_feedback_channel();
        thread::sleep(Duration::from_millis(200));
        out.lock().unwrap().flush().unwrap();

        write!(
            out.lock().unwrap(),
            "{}{}{} Welcome to the Rusty-CNC",
            termion::cursor::Goto(1, 1),
            termion::clear::All,
            termion::cursor::Hide,
        ).unwrap();
        out.lock().unwrap().flush().unwrap();
        'main: loop {
            match feedback.try_recv() {
                Ok(HardwareFeedback::Pos(PosData {x, y, z})) =>
                    write!(
                        out.lock().unwrap(),
                        "{} Pos: x {} y {} z {}   ",
                        termion::cursor::Goto(16, 2),
                        x, y, z
                    ).unwrap(),
                Ok(HardwareFeedback::Progress(todo, done)) =>
                    write!(
                        out.lock().unwrap(),
                        "{} todo: {} done: {}  ",
                        termion::cursor::Goto(45, 2),
                        todo,
                        done
                    ).unwrap(),
                Ok(HardwareFeedback::State(state)) =>
                    write!(
                        out.lock().unwrap(),
                        "{}{}        ",
                        termion::cursor::Goto(1, 2),
                        state
                    ).unwrap(),

                Ok(HardwareFeedback::RequireToolChange(id, size)) =>
                    write!(
                        out.lock().unwrap(),
                        "{}next Tool {}, {:?}    ",
                        termion::cursor::Goto(12, 3),
                        id,
                        size
                    ).unwrap(),
                _ => ()
            }
            match controller_recv.try_recv() {
                Ok(UserControlInput::Terminate) => {
                    println!("terminate");
                    break 'main;
                },
                Ok(UserControlInput::Stop) => println!("Stop"),
                Ok(UserControlInput::Start) => println!("Start"),
                Ok(UserControlInput::SelectProgram) => println!("SelectProgram"),
                Ok(UserControlInput::NextProgram) => println!("NextProgram"),
                Ok(UserControlInput::PrefProgram) => println!("PrefProgram"),
                Ok(UserControlInput::CalibrateZ) => {
                    write!(
                        out.lock().unwrap(),
                        "{} calibrate Z",
                        termion::cursor::Goto(1, 3),
                    ).unwrap();
                    hardware
                        .enqueue_instruction(Instruction::Calibrate(InstructionCalibrate {
                            x: CalibrateType::None,
                            y: CalibrateType::None,
                            z: CalibrateType::ContactPin,
                        }))?
                },
                Ok(UserControlInput::ResetPosToNull) => println!("ResetPosToNull"),
                Ok(UserControlInput::ManualControl(dir)) => {
                    write!(
                        out.lock().unwrap(),
                        "{} move manual x {} y {} z {}",
                        termion::cursor::Goto(0, 4),
                        dir.x, dir.y, dir.z
                    ).unwrap();

                    hardware
                        .enqueue_instruction(Instruction::ManualMovement(InstructionManualMovement {
                            speed: dir
                        }))?
                },
                Err(_) => {}
            }
            out.lock().unwrap().flush().unwrap();
            thread::sleep(Duration::from_millis(100));
        }
        out.lock().unwrap().suspend_raw_mode().unwrap();
        println!("");
        Ok(())
    }
}
