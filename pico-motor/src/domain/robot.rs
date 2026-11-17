use core::cell::{Ref, RefCell};

use critical_section::Mutex;

use crate::domain::{
    motor::Motion,
    safety::{SafetyState, StopReason},
    sensor::MotionSensorState,
};

pub struct RobotState {
    motion: Motion,
    safety: SafetyState,
    motion_sensor: MotionSensorState,
    last_command_us: u64,
    last_stop_reason: Option<StopReason>,
}

impl RobotState {
    pub const fn new(now_us: u64) -> Self {
        Self {
            motion: Motion::Stopped,
            safety: SafetyState::Normal,
            motion_sensor: MotionSensorState::new(),
            last_command_us: now_us,
            last_stop_reason: None,
        }
    }
}

static ROBOT_STATE: Mutex<RefCell<RobotState>> = Mutex::new(RefCell::new(RobotState::new(0u64)));
