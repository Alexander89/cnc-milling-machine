use std::fs::File;
use std::io::prelude::*;
use gcode::{Mnemonic, Parser, Nop, buffers::DefaultBuffers};

#[derive(Debug)]
pub struct Bounds {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
}

#[derive(Debug)]
pub struct Program {
    content: String,
    bounds: Option<Bounds>,
}

#[derive(Debug)]
pub struct Location {
    x: f64,
    y: f64,
    z: f64,
}

impl Program {
    pub fn new(path: &str) -> std::io::Result<Program> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(Program {
            content: content,
            bounds: None,
        })
    }
    
    pub fn run_program (&self) -> () {
        let lines: Parser<Nop, DefaultBuffers> = Parser::new(&self.content, Nop);
        for line in lines {
            for code in line.gcodes() {
                match code.mnemonic() {
                    Mnemonic::General => match code.major_number() {
                        0 | 1 | 2 | 3 => println!("command {:?}", code),
                        _ => {}
                    },
                    _ => {}
                }
            }
        };
    }
}