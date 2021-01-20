mod app;
mod io;
mod motor;
mod program;
mod types;
mod ui;

use app::App;

fn main() {
    // let package = WsCommands::Program(WsCommandProgram::Load{program_name: String::from("name")});
    // println!("output {:?}", serde_json::to_string(&package));
    App::start();
}
