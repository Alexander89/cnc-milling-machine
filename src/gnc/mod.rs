#![allow(dead_code)]
use crate::types::{CircleDirection, CircleMovement, LinearMovement, Location, MoveType};
use gcode::{buffers::DefaultBuffers, GCode, Mnemonic, Nop, Parser};
use std::{fs::File, io::prelude::*};

#[derive(Debug, Clone)]
enum Coordinations {
    Relative,
    Absolute,
}

#[derive(Debug, Clone)]
pub struct Gnc {
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

impl Gnc {
    pub fn new(
        path: &str,
        default_speed: f64,
        rapid_speed: f64,
        scaler: f64,
        start_pos: Location<f64>,
        invert_z: bool,
    ) -> std::io::Result<Gnc> {
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

        Ok(Gnc {
            content,
            codes,
            current_step: 0,
            scaler,
            coordinations: Coordinations::Absolute,
            current_position: start_pos,
            invert_z,
            current_speed: default_speed,
            rapid_speed,
        })
    }

    pub fn len(&self) -> usize {
        let mut current_step = 0usize;
        let mut count = 0usize;
        while let Some(step) = self.codes.get(current_step) {
            current_step += 1;
            if Mnemonic::General == step.mnemonic() {
                count += 1;
            }
        }
        count
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum NextMiscellaneous {
    SwitchOn,
    SwitchOff,
    ToolChange(i32),
    SpeedChange(f64),
}

#[derive(Debug, Clone)]
pub enum NextInstruction {
    Movement(Next3dMovement),
    Miscellaneous(NextMiscellaneous),
    ToolChange(String),
    InternalInstruction(String),
    NotSupported(String),
}

impl Iterator for Gnc {
    // we will be counting with usize
    type Item = NextInstruction;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let step = self.codes.get(self.current_step);
        self.current_step += 1;
        step?; // only continue if step
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

impl Gnc {
    fn parse_g_code(&mut self, code: GCode) -> Option<NextInstruction> {
        match code.major_number() {
            0 => {
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
                    move_type: MoveType::Rapid(LinearMovement { delta, distance }),
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
                    speed,
                    from: self.current_position.clone(),
                    to: self.update_location(
                        code.value_for('X'),
                        code.value_for('Y'),
                        code.value_for('Z'),
                    ),
                    move_type: MoveType::Linear(LinearMovement { delta, distance }),
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
                    speed,
                    from: self.current_position.clone(),
                    to: self.update_location(
                        code.value_for('X'),
                        code.value_for('Y'),
                        code.value_for('Z'),
                    ),
                    move_type: MoveType::Circle(CircleMovement {
                        center: center.clone(),
                        radius_sq: center.distance_sq(),
                        turn_direction,
                    }),
                };
                Some(NextInstruction::Movement(next_move))
            }
            20 => {
                // Some(NextInstruction::InternalInstruction(format!(
                //     "use inch unit {}",
                //     code.major_number()
                // )));
                panic!("inch not supported");
            }
            21 => Some(NextInstruction::InternalInstruction(format!(
                "use mm unit {}",
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
            0 | 1 | 2 | 5 => Some(NextInstruction::Miscellaneous(NextMiscellaneous::SwitchOff)),
            3 | 4 => Some(NextInstruction::Miscellaneous(NextMiscellaneous::SwitchOn)),
            6 => Some(NextInstruction::Miscellaneous(
                NextMiscellaneous::ToolChange(code.value_for('T').unwrap_or(1.0f32) as i32),
            )),

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
                x: get_or_default(x, self.current_position.x, false) - self.current_position.x,
                y: get_or_default(y, self.current_position.y, false) - self.current_position.y,
                z: get_or_default(z, self.current_position.z, self.invert_z)
                    - self.current_position.z,
            },
        }
    }
    fn rel_pos(&self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Location<f64> {
        Location {
            x: get_or_default(x, 0.0, false),
            y: get_or_default(y, 0.0, false),
            z: get_or_default(z, 0.0, self.invert_z),
        }
    }

    /// Update the current Location for the next instruction coordinate corresponding to the relative or absolute mode
    fn update_location(&mut self, x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Location<f64> {
        match self.coordinations {
            Coordinations::Relative => self.current_position = self.rel_pos(x, y, z),
            Coordinations::Absolute => {
                self.current_position = Location {
                    x: get_or_default(x, self.current_position.x, false),
                    y: get_or_default(y, self.current_position.y, false),
                    z: get_or_default(z, self.current_position.z, self.invert_z),
                }
            }
        };
        self.current_position.clone()
    }

    fn get_speed(&mut self, value: Option<f32>) -> f64 {
        if let Some(v) = value {
            self.current_speed = v as f64;
        };
        self.current_speed
    }
}

fn get_or_default(value: Option<f32>, default: f64, invert: bool) -> f64 {
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
