use crate::{
    adapter::clients::cdc_acm::cdc_acm_client::{CdcAcmClient, ResponseFrame},
    application::{
        dto::set_motion_dto::MotionOutput,
        ports::outbound::set_motion_present_port::{PresentError, SetMotionPresentPort},
    },
    domain::motor::Motion,
};

pub struct SetMotionPresenter {
    cdc_acm_client: &'static CdcAcmClient,
}

impl SetMotionPresenter {
    pub fn new(cdc_acm_client: &'static CdcAcmClient) -> Self {
        Self { cdc_acm_client }
    }
}

impl SetMotionPresentPort for SetMotionPresenter {
    async fn present(&self, output: MotionOutput) -> Result<(), PresentError> {
        let state = match output.state {
            Motion::Stopped => robot_protocol::MotionState::Stopped,
            Motion::Forward => robot_protocol::MotionState::Forward,
            Motion::Backward => robot_protocol::MotionState::Backward,
        };

        let response = robot_protocol::Response {
            seq: output.seq,
            ok: output.result,
            state,
        };
        let mut frame = ResponseFrame::new();
        robot_protocol::encode_response(&mut frame, response).map_err(|_| PresentError::Encode)?;

        self.cdc_acm_client.send(frame).await;

        Ok(())
    }
}
