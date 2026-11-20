use crate::{
    adapter::{
        controllers::jetson_controller::JetsonController, devices::motor_device::MotorDevice,
    },
    application::usecases::set_motion::SetMotionUseCase,
};

pub fn create_jetson_controller(
    motor_device: &MotorDevice,
) -> JetsonController<SetMotionUseCase<'_, MotorDevice>> {
    let set_motion_use_case = SetMotionUseCase {
        motor: motor_device,
    };

    JetsonController::new(set_motion_use_case)
}
