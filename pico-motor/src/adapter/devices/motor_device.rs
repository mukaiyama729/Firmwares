use crate::{
    adapter::clients::motor::motor_client::{MotorClient, MotorCommand, MotorStartError},
    application::ports::outbound::motor_port::MotorPort,
};

pub struct MotorDevice {
    pub client: &'static MotorClient,
}

impl MotorPort for MotorDevice {
    async fn send_command(&self, command: MotorCommand) -> Result<(), MotorStartError> {
        Ok(())
    }
}
