use crate::application::{
    dto::set_motion_dto::MotionInput, ports::inbound::jetson::set_motion::SetMotionInputPort,
};

pub struct JetsonController<C>
where
    C: SetMotionInputPort,
{
    usecase: C,
}

impl<C> JetsonController<C>
where
    C: SetMotionInputPort,
{
    pub fn new(usecase: C) -> Self {
        Self { usecase }
    }

    pub async fn run(&self, input: MotionInput) -> () {
        self.usecase.execute(input).await
    }
}
