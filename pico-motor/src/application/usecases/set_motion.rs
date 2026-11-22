use crate::{
    application::{
        dto::set_motion_dto::{MotionInput, MotionOutput, MotionRequest},
        ports::{
            inbound::jetson::set_motion::SetMotionInputPort,
            outbound::{
                motor_port::MotorPort,
                set_motion_present_port::{PresentError, SetMotionPresentPort},
            },
        },
    },
    domain::{
        motor::{Motion, MotorCommandDomain},
        robot::RobotStateFactory,
    },
};

/// Requests are executed sequentially by the controller.
pub struct SetMotionUseCase<'a, M, P>
where
    M: MotorPort,
    P: SetMotionPresentPort,
{
    pub motor: &'a M,
    pub presenter: P,
}

impl<'a, M, P> SetMotionUseCase<'a, M, P>
where
    M: MotorPort,
    P: SetMotionPresentPort,
{
    pub fn new(motor: &'a M, presenter: P) -> Self {
        Self { motor, presenter }
    }
}

impl<'a, M, P> SetMotionInputPort for SetMotionUseCase<'a, M, P>
where
    M: MotorPort,
    P: SetMotionPresentPort,
{
    async fn execute(&self, input: MotionInput) -> Result<(), PresentError> {
        let result = match input.request {
            MotionRequest::Set(motion) => {
                let command = match motion {
                    Motion::Stopped => MotorCommandDomain::Stop,
                    Motion::Forward => MotorCommandDomain::Forward,
                    Motion::Backward => MotorCommandDomain::Backward,
                };
                let accepted = self.motor.send_command(command).await.is_ok();
                if accepted {
                    RobotStateFactory::set_accepted_motion(motion);
                }
                accepted
            }
            MotionRequest::ReadState => true,
        };

        self.presenter
            .present(MotionOutput::new(
                input.seq,
                result,
                RobotStateFactory::accepted_motion().await,
            ))
            .await
    }
}
