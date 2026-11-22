use crate::domain::motor::Motion;

pub struct MotionInput {
    pub seq: u16,
    pub request: MotionRequest,
}

pub enum MotionRequest {
    Set(Motion),
    ReadState,
}

impl MotionInput {
    pub fn new(motion: Motion, seq: u16) -> Self {
        Self {
            request: MotionRequest::Set(motion),
            seq,
        }
    }

    pub fn read_state(seq: u16) -> Self {
        Self {
            request: MotionRequest::ReadState,
            seq,
        }
    }
}

pub struct MotionOutput {
    pub seq: u16,
    pub result: bool,
    pub state: Motion,
}

impl MotionOutput {
    pub fn new(seq: u16, result: bool, state: Motion) -> Self {
        Self {
            seq: seq,
            result: result,
            state: state,
        }
    }
}
