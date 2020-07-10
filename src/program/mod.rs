#![allow(dead_code)]
use crate::motor::ProgramTask;
use gcode::{buffers::DefaultBuffers, GCode, Nop, Parser};
use std::fs::File;
use std::io::prelude::*;

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
    codes: Vec<GCode>,
    current_step: usize,
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

        let mut codes = vec![];
        let lines: Parser<Nop, DefaultBuffers> = Parser::new(&content, Nop);
        for line in lines {
            for code in line.gcodes() {
                codes.push(code.to_owned());
            }
        }

        Ok(Program {
            content: content,
            bounds: None,
            codes: codes,
            current_step: 0,
        })
    }
}

pub struct Next3dMovement {
    pub x: Option<ProgramTask>,
    pub y: Option<ProgramTask>,
    pub z: Option<ProgramTask>,
}

impl Iterator for Program {
    // we will be counting with usize
    type Item = Option<Next3dMovement>;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let step = self.codes.get(self.current_step);
        self.current_step += 1;
        if step.is_none() {
            return None;
        }

        let next_move = Next3dMovement {
            x: None,
            y: None,
            z: None,
        };

        return Some(Some(next_move));
    }
}
