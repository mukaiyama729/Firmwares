use crate::domain::motor::Motion;

pub struct MotionInput {
    pub seq: u16,
    pub motion: Motion,
}
