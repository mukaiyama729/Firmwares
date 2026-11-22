use crate::application::dto::set_motion_dto::MotionOutput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentError {
    Encode,
}

pub trait SetMotionPresentPort {
    async fn present(&self, output: MotionOutput) -> Result<(), PresentError>;
}
