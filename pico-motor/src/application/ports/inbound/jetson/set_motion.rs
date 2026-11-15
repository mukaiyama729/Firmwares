use crate::application::dto::MotionInput;
pub trait SetMotionInputPort {
    fn execute(&self, input: MotionInput, ctx: &mut )
}
