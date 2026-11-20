use crate::application::{
    dto::set_motion_dto::MotionInput,
    ports::{inbound::jetson::set_motion::SetMotionInputPort, outbound::motor_port::MotorPort},
};

pub struct SetMotionUseCase<'a, M>
where
    M: MotorPort,
{
    pub motor: &'a M,
    //pub presenter: &'a mut R,
}

impl<'a, M> SetMotionUseCase<'a, M>
where
    M: MotorPort,
{
    pub fn new(motor: &'a M) -> Self {
        Self { motor: motor }
    }
}

impl<'a, M> SetMotionInputPort for SetMotionUseCase<'a, M>
where
    M: MotorPort,
{
    async fn execute(&self, input: MotionInput) -> () {
        ()
    }
}
