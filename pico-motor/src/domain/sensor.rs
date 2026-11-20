pub struct MotionSensorState {
    pub pir_detected: bool,
}

impl MotionSensorState {
    pub const fn new() -> Self {
        Self {
            pir_detected: false,
        }
    }
}
