use crate::application::{
    dto::set_motion_dto::MotionInput, ports::outbound::set_motion_present_port::PresentError,
};

pub trait SetMotionInputPort {
    async fn execute(&self, input: MotionInput) -> Result<(), PresentError>;
}
