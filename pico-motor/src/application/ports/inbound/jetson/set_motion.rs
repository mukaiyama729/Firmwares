pub trait SetMotionInputPort {
use crate::application::dto::MotionInput;
    fn execute(&self, input: MotionInput) -> ();
}
