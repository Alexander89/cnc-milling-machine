mod app;
mod gnc;
mod hardware_controller;
mod hardware_controller_interface;
mod io;
mod settings;
mod types;
mod ui;
mod control;

use app::App;

fn main() {
    App::start();
}
