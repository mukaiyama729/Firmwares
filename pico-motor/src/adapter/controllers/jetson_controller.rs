pub struct JetsonController<C>
where
    C: SetMotionInputPort,
{
    usecase: C,
}

impl JetsonController {
    pub async fn run(&self) -> () {
        ()
    }
}
