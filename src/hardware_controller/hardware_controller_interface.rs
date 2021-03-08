use crossbeam_channel::{unbounded, Receiver as CbReceiver};
use std::{
    fmt::Debug,
    sync::mpsc::{channel, SendError, Sender},
    thread,
};
use thread_priority::{set_current_thread_priority, ThreadPriority};

use super::{
    hardware_feedback::HardwareFeedback, instruction::Instruction, HardwareController,
    SettingsHardwareController,
};

#[derive(Debug)]
pub struct HardwareControllerInterface {
    controller_thread: thread::JoinHandle<()>,
    instruction_sender: Sender<Instruction>,
    feedback_receiver: CbReceiver<HardwareFeedback>,
}

impl HardwareControllerInterface {
    pub fn new(settings: SettingsHardwareController) -> HardwareControllerInterface {
        let (instruction_sender, instruction_receiver) = channel();
        let (feedback_sender, feedback_receiver) = unbounded();

        let controller_thread = thread::spawn(move || {
            if let Err(e) = set_current_thread_priority(ThreadPriority::Max) {
                println!("can not set thread lvl {:?}", e);
            }
            HardwareController::new(settings, instruction_receiver, feedback_sender).run();
        });

        Self {
            controller_thread,
            instruction_sender,
            feedback_receiver,
        }
    }
    pub fn enqueue_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), SendError<Instruction>> {
        self.instruction_sender.send(instruction)
    }

    pub fn set_settings(
        &mut self,
        settings: SettingsHardwareController,
    ) -> Result<(), SendError<Instruction>> {
        self.instruction_sender
            .send(Instruction::Settings(settings))
    }

    pub fn get_feedback_channel(&mut self) -> CbReceiver<HardwareFeedback> {
        self.feedback_receiver.clone()
    }
}
