use core::sync::atomic::{AtomicU8, Ordering};

use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output},
    peripherals::{PIN_2, PIN_3, PIN_4, PIN_5},
};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MotorTaskStatus {
    Uninitialized,
    Starting,
    Running,
}

pub enum MotorCommand {
    SetCoils(u8),
    Stop,
}

type MotorChannel = Channel<CriticalSectionRawMutex, MotorCommand, 8usize>;

#[derive(Debug)]
pub enum MotorStartError {
    AlreadyStarted,
}

static COMMANDS: MotorChannel = Channel::new();
static STATUS: AtomicU8 = AtomicU8::new(0);

pub struct MotorClient {
    commands: &'static MotorChannel,
}

impl MotorClient {
    pub fn start(
        spawner: &Spawner,
        p2: Peri<'static, PIN_2>,
        p3: Peri<'static, PIN_3>,
        p4: Peri<'static, PIN_4>,
        p5: Peri<'static, PIN_5>,
    ) -> Result<(), MotorStartError> {
        STATUS
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| MotorStartError::AlreadyStarted)?;
        let pins = [
            Output::new(p2, Level::Low),
            Output::new(p3, Level::Low),
            Output::new(p4, Level::Low),
            Output::new(p5, Level::Low),
        ];
    }
}
