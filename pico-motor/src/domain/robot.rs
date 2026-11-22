use core::{
    cell::{Ref, RefCell},
    ops::{Deref, DerefMut},
};

//use critical_section::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

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

static ROBOT_STATE: Mutex<CriticalSectionRawMutex, RobotState> = Mutex::new(RobotState::new(0u64));

pub struct RobotStateFactory;

impl RobotStateFactory {
    /// Last accepted motion, initially stopped; hardware application is not confirmed.
    pub(crate) async fn accepted_motion() -> Motion {
        Self::with_state(|state| state.motion).await
    }

    pub(crate) async fn set_accepted_motion(motion: Motion) {
        Self::with_state(|state| state.motion = motion).await;
    }

    async fn with_state<R>(f: impl FnOnce(&mut RobotState) -> R) -> R {
        let mut new_robot_state = ROBOT_STATE.lock().await;
        f(&mut new_robot_state)
    }
}
