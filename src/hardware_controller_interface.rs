use std::{fmt::Debug, sync::mpsc::{SendError, Sender, channel}, thread, time::Duration};
use thread_priority::{set_current_thread_priority, ThreadPriority};

use crate::{app::{SystemPublisher, SystemSubscriber, SystemEvents}, control::UserControlInput, hardware_controller::{InstructionCalibrate, InstructionManualMovement, instruction::CalibrateType}};
use crate::hardware_controller::{
    hardware_feedback::HardwareFeedback, instruction::Instruction, HardwareController,
    SettingsHardwareController,
};

#[derive(Debug)]
pub struct HardwareControllerInterface {
    controller_thread: thread::JoinHandle<()>,
    queue_thread: thread::JoinHandle<()>,
    instruction_sender: Sender<Instruction>,
}

impl HardwareControllerInterface {
    pub fn new(event_publish: SystemPublisher, event_subscribe: SystemSubscriber, settings: SettingsHardwareController) -> HardwareControllerInterface {
        let (instruction_sender, instruction_receiver) = channel();
        let (feedback_sender, feedback_receiver) = channel::<HardwareFeedback>();

        let controller_thread = thread::spawn(move || {
            if let Err(e) = set_current_thread_priority(ThreadPriority::Max) {
                println!("can not set thread lvl {:?}", e);
            }
            HardwareController::new(settings, instruction_receiver, feedback_sender).run();
        });

        let inner_instruction_sender = instruction_sender.clone();
        let queue_thread = thread::spawn(move || {
            'main: loop {
                while let Ok(event) = event_subscribe.try_recv() {
                    match event {
                        SystemEvents::Terminate => {
                            let _ = inner_instruction_sender.send(Instruction::Shutdown);
                            println!("terminate");
                            break 'main;
                        },
                        SystemEvents::ControlInput(UserControlInput::CalibrateZ) => {
                            let _ = inner_instruction_sender.send(Instruction::Calibrate(InstructionCalibrate {
                                x: CalibrateType::None,
                                y: CalibrateType::None,
                                z: CalibrateType::ContactPin,
                            }));
                        },
                        SystemEvents::ControlInput(UserControlInput::ManualControl(dir)) => {
                            let _ = inner_instruction_sender.send(Instruction::ManualMovement(InstructionManualMovement {
                                speed: dir
                            }));
                        },
                        _ => ()

                    }

                }
                while let Ok(feedback) = feedback_receiver.try_recv() {
                    let _ = event_publish.send(SystemEvents::HardwareFeedback(feedback));
                }
                thread::sleep(Duration::from_millis(100));
            }

        });

        Self {
            controller_thread,
            queue_thread,
            instruction_sender,
        }
    }

    #[allow(dead_code)]
    pub fn enqueue_instruction(&mut self, instruction: Instruction) -> Result<(), SendError<Instruction>> {
        self.instruction_sender.send(instruction)
    }

    #[allow(dead_code)]
    pub fn set_settings(&mut self, settings: SettingsHardwareController) -> Result<(), SendError<Instruction>> {
        self.instruction_sender.send(Instruction::Settings(settings))
    }

}
