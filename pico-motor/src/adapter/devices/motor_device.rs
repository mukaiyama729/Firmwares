use crate::{
    adapter::clients::{
        client_core::ClientSendError,
        motor::motor_client::{MotorClient, MotorCommand},
    },
    application::ports::outbound::motor_port::MotorPort,
    domain::motor::MotorCommandDomain,
};

pub struct MotorDevice {
    pub client: &'static MotorClient,
}

impl MotorPort for MotorDevice {
    async fn send_command(&self, command: MotorCommandDomain) -> Result<(), ClientSendError> {
        match command.into() {
            MotorCommand::SetCoils(pattern) => self.client.set_coils(pattern).await,
            MotorCommand::Stop => self.client.stop().await,
        }
    }
}

impl From<MotorCommandDomain> for MotorCommand {
    fn from(command: MotorCommandDomain) -> Self {
        match command {
            MotorCommandDomain::Forward => Self::SetCoils(0b0001),
            MotorCommandDomain::ForwardRight => Self::SetCoils(0b0011),
            MotorCommandDomain::Right => Self::SetCoils(0b0010),
            MotorCommandDomain::BackwardRight => Self::SetCoils(0b0110),
            MotorCommandDomain::Backward => Self::SetCoils(0b0100),
            MotorCommandDomain::BackwardLeft => Self::SetCoils(0b1100),
            MotorCommandDomain::Left => Self::SetCoils(0b1000),
            MotorCommandDomain::ForwardLeft => Self::SetCoils(0b1001),
            MotorCommandDomain::Stop => Self::Stop,
        }
    }
}
