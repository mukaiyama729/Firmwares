use crate::{adapter::clients::client_core::ClientSendError, domain::motor::MotorCommandDomain};

pub trait MotorPort {
    async fn send_command(&self, command: MotorCommandDomain) -> Result<(), ClientSendError>;
}
