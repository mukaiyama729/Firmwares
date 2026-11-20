use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    peripherals::{PIN_2, PIN_3, PIN_4, PIN_5},
};

use crate::adapter::clients::motor::motor_client::MotorStartError;

trait MotorPort {
    fn new() -> Self;
    fn start(
        &'static self,
        spawner: Spawner,
        p2: Peri<'static, PIN_2>,
        p3: Peri<'static, PIN_3>,
        p4: Peri<'static, PIN_4>,
        p5: Peri<'static, PIN_5>,
    ) -> Result<(), MotorStartError>;
}
