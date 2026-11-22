use core::str::from_utf8;

use crate::{
    adapter::clients::cdc_acm::cdc_acm_client::CdcAcmClient,
    application::{
        dto::set_motion_dto::MotionInput, ports::inbound::jetson::set_motion::SetMotionInputPort,
    },
    domain::motor::Motion,
};

pub struct JetsonController<C>
where
    C: SetMotionInputPort,
{
    cdc_acm_client: &'static CdcAcmClient,
    motion_usecase: C,
}

impl<C> JetsonController<C>
where
    C: SetMotionInputPort,
{
    pub fn new(motion_usecase: C, cdc_acm_client: &'static CdcAcmClient) -> Self {
        Self {
            motion_usecase,
            cdc_acm_client,
        }
    }

    pub async fn run(&self) -> () {
        loop {
            let request = self.cdc_acm_client.receive().await;
            let instruction = match (from_utf8(request.as_slice())) {
                Ok(some_str) => some_str,
                Err(_) => continue,
            };
            let req = match robot_protocol::decode_request(instruction) {
                Ok(req) => req,
                Err(_) => continue,
            };

            let input = match req.command {
                robot_protocol::Command::Stop => MotionInput::new(Motion::Stopped, req.seq),
                robot_protocol::Command::Forward => MotionInput::new(Motion::Forward, req.seq),
                robot_protocol::Command::Backward => MotionInput::new(Motion::Backward, req.seq),
                robot_protocol::Command::Status | robot_protocol::Command::Ping => {
                    MotionInput::read_state(req.seq)
                }
            };

            if self.motion_usecase.execute(input).await.is_err() {
                defmt::warn!("Failed to present motion response");
            }
        }
    }
}
