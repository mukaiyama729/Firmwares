use crate::application::dto::set_motion_dto::MotionInput;

pub trait SetMotionInputPort {
    fn execute(&self, input: MotionInput) -> ();
}
