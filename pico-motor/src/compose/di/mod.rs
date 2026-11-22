use crate::{
    adapter::{
        clients::cdc_acm::cdc_acm_client::CdcAcmClient,
        controllers::jetson_controller::JetsonController, devices::motor_device::MotorDevice,
        presenters::set_motion_presenter::SetMotionPresenter,
    },
    application::usecases::set_motion::SetMotionUseCase,
};

pub fn create_jetson_controller<'a>(
    motor_device: &'a MotorDevice,
    cdc_acm_client: &'static CdcAcmClient,
) -> JetsonController<SetMotionUseCase<'a, MotorDevice, SetMotionPresenter>> {
    let presenter = SetMotionPresenter::new(cdc_acm_client);
    let set_motion_use_case = SetMotionUseCase::new(motor_device, presenter);
    JetsonController::new(set_motion_use_case, cdc_acm_client)
}
