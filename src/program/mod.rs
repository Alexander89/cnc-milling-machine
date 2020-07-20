#![allow(dead_code)]
use crate::motor::{Direction, MoveType, Task};
use gcode::{buffers::DefaultBuffers, GCode, Mnemonic, Nop, Parser};
use std::{fs::File, io::prelude::*, time::Duration};

#[derive(Debug)]
enum Coordinations {
    Relative,
    Absolute,
}

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
    sqn: u32,
    scaler: f32,
    max_speed: f32,
    coordinations: Coordinations,
    current_position: Location,
}

#[derive(Debug)]
pub struct Location {
    x: f32,
    y: f32,
    z: f32,
}

impl Program {
    pub fn new(path: &str, max_speed: f32, scaler: f32) -> std::io::Result<Program> {
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
            sqn: 0,
            scaler: scaler,
            max_speed: max_speed,
            coordinations: Coordinations::Absolute,
            current_position: Location {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        })
    }
}

#[derive(Debug)]
pub struct Next3dMovement {
    pub x: Task,
    pub y: Task,
    pub z: Task,
}

#[derive(Debug)]
pub enum NextMiscellaneous {
    SwitchOn,
    SwitchOff,
}

#[derive(Debug)]
pub enum NextInstruction {
    Movement(Next3dMovement),
    Miscellaneous(NextMiscellaneous),
    ToolChange(String),
    InternalInstruction(String),
    NotSupported(String),
}

impl Iterator for Program {
    // we will be counting with usize
    type Item = NextInstruction;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let step = self.codes.get(self.current_step);
        self.current_step += 1;
        if step.is_none() {
            return None;
        }
        let code = step.unwrap().to_owned();

        let res = match code.mnemonic() {
            Mnemonic::General => self.parse_g_code(self.sqn, code),
            Mnemonic::Miscellaneous => self.parse_m_code(code),
            Mnemonic::ProgramNumber => self.parse_p_code(code),
            Mnemonic::ToolChange => self.parse_t_code(code),
        };

        match res {
            None => self.next(),
            Some(NextInstruction::Movement(_)) => {
                self.sqn += 1;
                res
            }
            Some(_) => res,
        }
    }
}

impl Program {
    fn parse_g_code(&mut self, sqn: u32, code: GCode) -> Option<NextInstruction> {
        match code.major_number() {
            0 => {
                let (dx, dy, dz, distance) = self.move_distance(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                if distance == 0.0 {
                    return None;
                }
                let next_move = Next3dMovement {
                    x: self.mk_task(sqn, dx, distance, self.max_speed),
                    y: self.mk_task(sqn, dy, distance, self.max_speed),
                    z: self.mk_task(sqn, dz, distance, self.max_speed),
                };

                self.update_location(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                Some(NextInstruction::Movement(next_move))
            }
            1 => {
                let (dx, dy, dz, distance) = self.move_distance(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                if distance == 0.0 {
                    return None;
                }
                let next_move = Next3dMovement {
                    x: self.mk_task(
                        sqn,
                        dx,
                        distance,
                        code.value_for('F').unwrap_or(self.max_speed),
                    ),
                    y: self.mk_task(
                        sqn,
                        dy,
                        distance,
                        code.value_for('F').unwrap_or(self.max_speed),
                    ),
                    z: self.mk_task(
                        sqn,
                        dz,
                        distance,
                        code.value_for('F').unwrap_or(self.max_speed),
                    ),
                };
                self.update_location(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                Some(NextInstruction::Movement(next_move))
            }
            2 => Some(NextInstruction::NotSupported(format!(
                "{}",
                code.major_number()
            ))),
            3 => Some(NextInstruction::NotSupported(format!(
                "{}",
                code.major_number()
            ))),
            90 => {
                self.coordinations = Coordinations::Absolute;
                Some(NextInstruction::InternalInstruction(format!(
                    "{}",
                    code.major_number()
                )))
            }
            91 => {
                self.coordinations = Coordinations::Relative;
                Some(NextInstruction::InternalInstruction(format!(
                    "{}",
                    code.major_number()
                )))
            }
            _ => Some(NextInstruction::NotSupported(format!(
                "{}",
                code.major_number()
            ))),
        }
    }
    /// execute Machine and Program codes
    fn parse_m_code(&mut self, code: GCode) -> Option<NextInstruction> {
        match code.major_number() {
            0 | 1 | 5 => Some(NextInstruction::Miscellaneous(NextMiscellaneous::SwitchOff)),
            3 | 4 => Some(NextInstruction::Miscellaneous(NextMiscellaneous::SwitchOn)),
            _ => Some(NextInstruction::NotSupported(format!(
                "M code - {}",
                code.major_number()
            ))),
        }
    }
    fn parse_p_code(&mut self, code: GCode) -> Option<NextInstruction> {
        Some(NextInstruction::NotSupported(format!(
            "Program - {}",
            code.major_number()
        )))
    }
    fn parse_t_code(&mut self, code: GCode) -> Option<NextInstruction> {
        Some(NextInstruction::NotSupported(format!(
            "ToolChange - {}",
            code.major_number()
        )))
    }

    fn mk_task(&self, sqn: u32, distance: f32, overall_distance: f32, speed: f32) -> Task {
        let max_speed = self.max_speed.min(speed) as f64;
        let duration = Duration::from_secs_f64(overall_distance as f64 / max_speed);

        if distance == 0.0 {
            Task::NOOP(sqn, duration)
        } else {
            let d = distance as f64;

            let direction = if d > 0.0 {
                Direction::LEFT
            } else {
                Direction::RIGHT
            };

            Task::program(sqn, direction, d.abs(), duration, MoveType::LINEAR)
        }
    }

    /// Calculate the move distance to the new coordinate corresponding to the relative or absolute mode
    fn move_distance(
        &self,
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
    ) -> (f32, f32, f32, f32) {
        match self.coordinations {
            Coordinations::Relative => {
                let xv = x.unwrap_or(0.0f32) * self.scaler;
                let yv = y.unwrap_or(0.0f32) * self.scaler;
                let zv = z.unwrap_or(0.0f32) * self.scaler;
                let over_all = (xv * xv + yv * yv + zv * zv).sqrt();
                (xv, yv, zv, over_all)
            }
            Coordinations::Absolute => {
                let xv = x.unwrap_or(self.current_position.x / self.scaler) * self.scaler
                    - self.current_position.x;
                let yv = y.unwrap_or(self.current_position.y / self.scaler) * self.scaler
                    - self.current_position.y;
                let zv = z.unwrap_or(self.current_position.z / self.scaler) * self.scaler
                    - self.current_position.z;
                // println!(
                //     "{} {} {} // from {} {} {} + {} {} {} => to {} {} {}",
                //     x.unwrap_or(f32::NAN),
                //     y.unwrap_or(f32::NAN),
                //     z.unwrap_or(f32::NAN),
                //     self.current_position.x,
                //     self.current_position.y,
                //     self.current_position.z,
                //     xv,
                //     yv,
                //     zv,
                //     self.current_position.x + xv,
                //     self.current_position.y + yv,
                //     self.current_position.z + zv,
                // );
                let over_all = (xv * xv + yv * yv + zv * zv).sqrt();
                (xv, yv, zv, over_all)
            }
        }
    }

    /// Update the current Location for the next instruction coordinate corresponding to the relative or absolute mode
    fn update_location(&mut self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> () {
        match self.coordinations {
            Coordinations::Relative => {
                self.current_position = Location {
                    x: self.current_position.x + x.unwrap_or(0.0),
                    y: self.current_position.y + y.unwrap_or(0.0),
                    z: self.current_position.z + z.unwrap_or(0.0),
                }
            }
            Coordinations::Absolute => {
                self.current_position = Location {
                    x: x.unwrap_or(self.current_position.x),
                    y: y.unwrap_or(self.current_position.y),
                    z: z.unwrap_or(self.current_position.z),
                }
            }
        };
    }
}
