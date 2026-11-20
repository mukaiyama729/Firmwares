#[derive(Debug, Clone, Copy)]
pub enum Motion {
    Stopped,
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MotorCommandDomain {
    Forward,       // 上（前進）
    ForwardRight,  // 右上
    Right,         // 右
    BackwardRight, // 右下
    Backward,      // 下（後退）
    BackwardLeft,  // 左下
    Left,          // 左
    ForwardLeft,   // 左上
    Stop,          // 停止
}
