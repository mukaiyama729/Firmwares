use crate::application::dto::set_motion_dto::MotionInput;

pub trait SetMotionInputPort {
    async fn execute(&self, input: MotionInput) -> ();
}
