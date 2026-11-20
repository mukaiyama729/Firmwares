pub trait SetMotionPresentPort {
    async fn present(&self, succeed: bool) -> ();
}
