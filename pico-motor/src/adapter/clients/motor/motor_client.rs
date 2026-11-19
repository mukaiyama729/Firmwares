use embassy_executor::Spawner;

use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals::{PIN_2, PIN_3, PIN_4, PIN_5},
};

use crate::adapter::clients::client_core::{
    ClientCore, ClientSendError, ClientStartError, ClientStatus,
};

pub type MotorStartError = ClientStartError;
pub type MotorTaskStatus = ClientStatus;

pub enum MotorCommand {
    SetCoils(u8),
    Stop,
}

pub struct MotorClient {
    core: ClientCore<MotorCommand, 8>,
}

pub static MOTOR_CLIENT: MotorClient = MotorClient::new();

impl MotorClient {
    pub const fn new() -> Self {
        Self {
            core: ClientCore::<MotorCommand, 8>::new(),
        }
    }

    pub fn start(
        &'static self,
        spawner: Spawner,
        p2: Peri<'static, PIN_2>,
        p3: Peri<'static, PIN_3>,
        p4: Peri<'static, PIN_4>,
        p5: Peri<'static, PIN_5>,
    ) -> Result<(), MotorStartError> {
        self.core.begin_start()?;

        let pins = [
            Output::new(p2, Level::Low),
            Output::new(p3, Level::Low),
            Output::new(p4, Level::Low),
            Output::new(p5, Level::Low),
        ];

        let token = match motor_worker(self, pins) {
            Ok(token) => token,
            Err(error) => {
                self.core.rollback_start();
                return Err(MotorStartError::Spawn(error));
            }
        };

        // 4. Executorへ登録
        spawner.spawn(token);

        Ok(())
    }

    pub fn get() -> Option<&'static Self> {
        if MOTOR_CLIENT.is_running() {
            Some(&MOTOR_CLIENT)
        } else {
            None
        }
    }

    pub fn status(&self) -> MotorTaskStatus {
        self.core.status()
    }

    pub fn is_started(&self) -> bool {
        self.core.is_started()
    }

    pub fn is_running(&self) -> bool {
        self.core.is_running()
    }

    pub async fn set_coils(&self, pattern: u8) -> Result<(), ClientSendError> {
        self.core
            .send(MotorCommand::SetCoils(pattern & 0b1111))
            .await
    }

    pub async fn stop(&self) -> Result<(), ClientSendError> {
        self.core.send(MotorCommand::Stop).await
    }
}

#[embassy_executor::task]
async fn motor_worker(client: &'static MotorClient, mut pins: [Output<'static>; 4]) {
    client.core.mark_running();

    loop {
        let command = client.core.receive().await;

        let pattern = match command {
            MotorCommand::SetCoils(pattern) => pattern,
            MotorCommand::Stop => 0,
        };

        for (i, pin) in pins.iter_mut().enumerate() {
            let active = ((pattern >> i) & 1) != 0;

            if active {
                pin.set_high();
            } else {
                pin.set_low();
            }
        }
    }
}
