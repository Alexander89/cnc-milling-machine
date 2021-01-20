use crate::program::Next3dMovement;
use crate::types::{
    CircleMovement, LinearMovement, Location, MachineState, MoveType, SteppedCircleMovement,
    SteppedLinearMovement, SteppedMoveType,
};
use std::{fmt::Debug, time::SystemTime};

#[derive(Debug)]
pub struct InnerTaskProduction {
    pub start_time: SystemTime,
    pub from: Location<i64>,
    pub destination: Location<i64>,
    pub move_type: SteppedMoveType,
}
#[derive(Debug, Clone)]
pub enum CalibrateType {
    None,
    Min,
    Max,
    Middle,
    ContactPin,
}
#[derive(Debug)]
pub struct InnerTaskCalibrate {
    pub start_time: SystemTime,
    pub from: Location<i64>,
    pub x: CalibrateType,
    pub y: CalibrateType,
    pub z: CalibrateType,
}

#[derive(Debug)]
pub enum InnerTask {
    Production(InnerTaskProduction),
    Calibrate(InnerTaskCalibrate),
}
impl InnerTask {
    /**
     * convert task to InnerTask
     *
     * - **t** Task to convert
     * - **current_pos** current position in steps
     * - **step_size** mm per step
     * - **max_speed** mm per sec
     */
    pub fn from_task(
        t: Task,
        current_pos: Location<i64>,
        step_sizes: Location<f64>,
        max_speed: f64,
    ) -> InnerTask {
        match t {
            Task::Manual(task) => {
                let input = Location::new(task.move_x_speed, task.move_y_speed, task.move_z_speed);
                let speed = input.clone().distance() * task.speed;

                let move_vec: Location<f64> = input.clone() * 10000.0f64;
                let delta: Location<i64> = (move_vec.clone() / step_sizes).into(); // [steps] (10m more than the table into i64 steps)

                let destination = current_pos.clone() + delta.clone();
                let distance = move_vec.distance();

                InnerTask::Production(InnerTaskProduction {
                    start_time: SystemTime::now(),
                    from: current_pos,
                    destination,
                    move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                        delta,
                        distance,
                        speed,
                    }),
                })
            }
            Task::Program(Next3dMovement {
                speed,
                move_type,
                to,
                ..
            }) => match move_type {
                MoveType::Linear(LinearMovement { distance, delta }) => {
                    let delta_in_steps: Location<i64> = (delta / step_sizes).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        from: current_pos.clone(),
                        destination: delta_in_steps.clone() - current_pos,
                        move_type: SteppedMoveType::Linear(SteppedLinearMovement {
                            delta: delta_in_steps,
                            distance,
                            speed: speed.min(max_speed),
                        }),
                    })
                }
                MoveType::Rapid(LinearMovement { distance, delta }) => {
                    let delta_in_steps: Location<i64> = (delta / step_sizes).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        from: current_pos.clone(),
                        destination: delta_in_steps.clone() - current_pos,
                        move_type: SteppedMoveType::Rapid(SteppedLinearMovement {
                            delta: delta_in_steps,
                            distance,
                            speed: speed.min(max_speed),
                        }),
                    })
                }
                MoveType::Circle(CircleMovement {
                    center,
                    turn_direction,
                    radius_sq,
                }) => {
                    let destination = to / step_sizes.clone();

                    let step_delay = step_sizes.max() / speed.min(max_speed).max(0.05);
                    let step_center = (center / step_sizes.clone()).into();

                    InnerTask::Production(InnerTaskProduction {
                        start_time: SystemTime::now(),
                        from: current_pos,
                        destination: destination.into(),
                        move_type: SteppedMoveType::Circle(SteppedCircleMovement {
                            center: step_center,
                            radius_sq,
                            step_sizes,
                            turn_direction,
                            speed,
                            step_delay,
                        }),
                    })
                }
            },

            Task::Calibrate(x, y, z) => InnerTask::Calibrate(InnerTaskCalibrate {
                start_time: SystemTime::now(),
                from: current_pos,
                x,
                y,
                z,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ManualTask {
    /** x speed from -1.0 to 1.0 */
    pub move_x_speed: f64,
    /** y speed from -1.0 to 1.0 */
    pub move_y_speed: f64,
    /** z speed from -1.0 to 1.0 */
    pub move_z_speed: f64,
    /** move speed [mm/sec] */
    pub speed: f64,
}

#[derive(Debug, Clone)]
pub enum Task {
    Program(Next3dMovement),
    Manual(ManualTask),
    Calibrate(CalibrateType, CalibrateType, CalibrateType),
}

impl Task {
    pub fn machine_state(&self) -> MachineState {
        match self {
            Task::Program(_) => MachineState::ProgramTask,
            Task::Manual(_) => MachineState::ManualTask,
            Task::Calibrate(_, _, _) => MachineState::Calibrate,
        }
    }
}
