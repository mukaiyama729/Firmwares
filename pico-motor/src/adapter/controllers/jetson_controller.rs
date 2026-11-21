use core::str::from_utf8;

use crate::{
    adapter::clients::cdc_acm::cdc_acm_client::CdcAcmClient,
    application::{
        dto::set_motion_dto::MotionInput, ports::inbound::jetson::set_motion::SetMotionInputPort,
    },
};

pub struct JetsonController<C>
where
    C: SetMotionInputPort,
{
    cdc_acm_client: &'static CdcAcmClient,
    usecase: C,
}

impl<C> JetsonController<C>
where
    C: SetMotionInputPort,
{
    pub fn new(usecase: C, cdc_acm_client: &'static CdcAcmClient) -> Self {
        Self {
            usecase,
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

            self.usecase.execute(input).await
        }
    }
}
