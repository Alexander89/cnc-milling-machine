#![allow(dead_code)]
use crate::types::{CircleDirection, CircleMovement, LinearMovement, Location, MoveType};
use gcode::{buffers::DefaultBuffers, GCode, Mnemonic, Nop, Parser};
use std::{fs::File, io::prelude::*};

#[derive(Debug)]
enum Coordinations {
    Relative,
    Absolute,
}

#[derive(Debug)]
pub struct Program {
    content: String,
    codes: Vec<GCode>,
    current_step: usize,
    scaler: f64,
    coordinations: Coordinations,
    current_position: Location<f64>,
    invert_z: bool,
    current_speed: f64,
    rapid_speed: f64,
}

impl Program {
    pub fn new(
        path: &str,
        default_speed: f64,
        rapid_speed: f64,
        scaler: f64,
        start_pos: Location<f64>,
        invert_z: bool,
    ) -> std::io::Result<Program> {
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
            codes: codes,
            current_step: 0,
            scaler: scaler,
            coordinations: Coordinations::Absolute,
            current_position: start_pos,
            invert_z: invert_z,
            current_speed: default_speed,
            rapid_speed: rapid_speed,
        })
    }
}

#[derive(Debug)]
/** parsed GCode instruction to move the rotor head */
pub struct Next3dMovement {
    /** mm per sec */
    pub speed: f64,
    /** validation start os */
    pub from: Location<f64>,
    /** target pos */
    pub to: Location<f64>,
    /** movement type (Linear, Rapid, Bevel, ...) */
    pub move_type: MoveType,
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
            Mnemonic::General => self.parse_g_code(code),
            Mnemonic::Miscellaneous => self.parse_m_code(code),
            Mnemonic::ProgramNumber => self.parse_p_code(code),
            Mnemonic::ToolChange => self.parse_t_code(code),
        };

        match res {
            None => self.next(),
            Some(_) => res,
        }
    }
}

impl Program {
    fn parse_g_code(&mut self, code: GCode) -> Option<NextInstruction> {
        match code.major_number() {
            0 => {
                println!(
                    "{:?} {:?} {:?}",
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z')
                );
                let delta = self.move_delta(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                let distance = delta.distance();
                if distance == 0.0 {
                    return None;
                }
                let next_move = Next3dMovement {
                    speed: self.rapid_speed,
                    from: self.current_position.clone(),
                    to: self.update_location(
                        code.value_for('X'),
                        code.value_for('Y'),
                        code.value_for('Z'),
                    ),
                    move_type: MoveType::Rapid(LinearMovement {
                        delta: delta,
                        distance: distance,
                    }),
                };
                Some(NextInstruction::Movement(next_move))
            }
            1 => {
                let delta = self.move_delta(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                let distance = delta.distance();
                if distance == 0.0 {
                    return None;
                }
                let speed = self.get_speed(code.value_for('F'));

                let next_move = Next3dMovement {
                    speed: speed,
                    from: self.current_position.clone(),
                    to: self.update_location(
                        code.value_for('X'),
                        code.value_for('Y'),
                        code.value_for('Z'),
                    ),
                    move_type: MoveType::Linear(LinearMovement {
                        delta: delta,
                        distance: distance,
                    }),
                };
                Some(NextInstruction::Movement(next_move))
            }
            major_number @ 2 | major_number @ 3 => {
                let delta = self.move_delta(
                    code.value_for('X'),
                    code.value_for('Y'),
                    code.value_for('Z'),
                );
                if delta.distance() == 0.0 {
                    return None;
                }
                let speed = self.get_speed(code.value_for('F'));
                let center = self.rel_pos(
                    code.value_for('I'),
                    code.value_for('J'),
                    code.value_for('K'),
                );

                let turn_direction = if major_number == 2 {
                    CircleDirection::CW
                } else {
                    CircleDirection::CCW
                };

                let next_move = Next3dMovement {
                    speed: speed,
                    from: self.current_position.clone(),
                    to: self.update_location(
                        code.value_for('X'),
                        code.value_for('Y'),
                        code.value_for('Z'),
                    ),
                    move_type: MoveType::Circle(CircleMovement {
                        center: center.clone(),
                        radius_sq: center.distance_sq(),
                        turn_direction: turn_direction,
                    }),
                };
                Some(NextInstruction::Movement(next_move))
            }
            20 => {
                panic!("inch not supported");
                Some(NextInstruction::InternalInstruction(format!(
                    "use inch unit {}",
                    code.major_number()
                )))
            }
            21 => {
                Some(NextInstruction::InternalInstruction(format!(
                    "use mm unit {}",
                    code.major_number()
                )))
            }
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

    /// Calculate the move distance to the new coordinate corresponding to the relative or absolute mode
    fn move_delta(&self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Location<f64> {
        match self.coordinations {
            Coordinations::Relative => self.rel_pos(x, y, z),
            Coordinations::Absolute => Location {
                x: self.get_or_default(x, self.current_position.x, false) - self.current_position.x,
                y: self.get_or_default(y, self.current_position.y, false) - self.current_position.y,
                z: self.get_or_default(z, self.current_position.z, self.invert_z)
                    - self.current_position.z,
            },
        }
    }

    fn rel_pos(&self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Location<f64> {
        Location {
            x: self.get_or_default(x, 0.0, false),
            y: self.get_or_default(y, 0.0, false),
            z: self.get_or_default(z, 0.0, self.invert_z),
        }
    }

    /// Update the current Location for the next instruction coordinate corresponding to the relative or absolute mode
    fn update_location(&mut self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Location<f64> {
        match self.coordinations {
            Coordinations::Relative => self.current_position = self.rel_pos(x, y, z),
            Coordinations::Absolute => {
                self.current_position = Location {
                    x: self.get_or_default(x, self.current_position.x, false),
                    y: self.get_or_default(y, self.current_position.y, false),
                    z: self.get_or_default(z, self.current_position.z, self.invert_z),
                }
            }
        };
        self.current_position.clone()
    }
    fn get_or_default(&self, value: Option<f32>, default: f64, invert: bool) -> f64 {
        let value = if let Some(v) = value {
            v as f64
        } else {
            default
        };

        if invert {
            -value
        } else {
            value
        }
    }
    fn get_speed(&mut self, value: Option<f32>) -> f64 {
        if let Some(v) = value {
            self.current_speed = v as f64;
        };
        self.current_speed
    }
}
