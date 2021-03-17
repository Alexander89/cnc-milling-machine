use std::{io::{Stdout, Write}, sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use termion::raw::RawTerminal;

use crate::app::{SystemPublisher, SystemSubscriber, SystemEvents};
use crate::control::UserControlInput;
use crate::{
    hardware_controller::{
        HardwareFeedback,
        PosData,
    }
};


//($dst:expr, $($arg:tt)*) => ($dst.write_fmt($crate::format_args!($($arg)*)))
#[macro_export]
macro_rules! printScreen {
    ($out:expr, $text:expr, $x:expr, $y:expr) => ({
        let mut lock = $out.lock().unwrap();
        write!( lock, "{}", termion::cursor::Goto($x, $y) ).unwrap();
        write!( lock, $text ).unwrap();
    });
    ($out:expr, $text:expr, $x:expr, $y:expr, $($arg:tt)* ) => ({
        let mut lock = $out.lock().unwrap();
        write!( lock, "{}", termion::cursor::Goto($x, $y) ).unwrap();
        write!( lock, $text, $($arg)* ).unwrap();
    });
}


pub struct Console {
    ui_thread: JoinHandle<()>
}

impl Console {
    pub fn new(_event_publish: SystemPublisher, event_subscribe: SystemSubscriber, out: Arc<Mutex<RawTerminal<Stdout>>>) -> Self {
        out.lock().unwrap().flush().unwrap();
        printScreen!(out, "{}{}Welcome to the Rusty-CNC", 1, 1, termion::clear::All, termion::cursor::Hide);
        out.lock().unwrap().flush().unwrap();

        let ui_thread = thread::spawn(move || {
            'main: loop {
                if let Ok(event) = event_subscribe.recv() {
                    match event {
                        SystemEvents::HardwareFeedback(HardwareFeedback::Pos(PosData {x, y, z})) =>
                            printScreen!(out, "Pos: x {} y {} z {}   ", 16, 2, x, y, z),
                        SystemEvents::HardwareFeedback(HardwareFeedback::Progress(todo, done)) =>
                            printScreen!(out, "todo: {} done: {}  ", 45, 2, todo, done),
                        SystemEvents::HardwareFeedback(HardwareFeedback::State(state)) =>
                            printScreen!(out, "{}        ", 1, 2, state),
                        SystemEvents::HardwareFeedback(HardwareFeedback::RequireToolChange(id, size)) =>
                            printScreen!(out, "next Tool {}, {:?}    ", 12, 3, id, size),
                        SystemEvents::Terminate => {
                            printScreen!(out, "Terminate    ", 15, 4);
                            break 'main;
                        }
                        SystemEvents::ControlInput(UserControlInput::Stop) =>
                            printScreen!(out, "Stop         ", 15, 4),
                        SystemEvents::ControlInput(UserControlInput::Start) =>
                            printScreen!(out, "Start         ", 15, 4),
                        SystemEvents::ControlInput(UserControlInput::SelectProgram) =>
                            printScreen!(out, "SelectProgram ", 15, 4),
                        SystemEvents::ControlInput(UserControlInput::NextProgram) =>
                            printScreen!(out, "NextProgram   ", 15, 4),
                        SystemEvents::ControlInput(UserControlInput::PrefProgram) =>
                            printScreen!(out, "PrefProgram   ", 15, 4),
                        SystemEvents::ControlInput(UserControlInput::CalibrateZ) =>
                            printScreen!(out, "calibrate Z   ", 1, 3),
                        SystemEvents::ControlInput(UserControlInput::ResetPosToNull) =>
                            printScreen!(out, "ResetPosToNull", 1, 3),
                        SystemEvents::ControlInput(UserControlInput::ManualControl(dir)) =>
                            printScreen!(out, "move manual x {} y {} z {}  ", 0, 4, dir.x, dir.y, dir.z),

                        _ => continue 'main,
                    }
                    out.lock().unwrap().flush().unwrap();
                }

            };
        });

        Self {
            ui_thread
        }
    }
}

