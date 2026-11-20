use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    peripherals::{PIN_2, PIN_3, PIN_4, PIN_5},
};

use crate::adapter::clients::motor::motor_client::{MotorCommand, MotorStartError};

pub trait MotorPort {
    async fn send_command(&self, command: MotorCommand) -> Result<(), MotorStartError>;
}
