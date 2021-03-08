mod app;
mod gnc;
mod hardware_controller;
mod io;
mod settings;
mod types;
mod ui;

use app::App;

fn main() {
    // let package = WsCommands::Program(WsCommandProgram::Load{program_name: String::from("name")});
    // println!("output {:?}", serde_json::to_string(&package));
    App::start();
}
