#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StopReason {
    PirMotionDetected,
    DeviceFault,
    CommandTimeout,
}

#[derive(Debug, Clone, Copy)]
pub enum SafetyState {
    Normal,
    Latched(StopReason),
}
