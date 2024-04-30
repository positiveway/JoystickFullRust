#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SteamyEvent {
    Connected,
    Disconnected,

    Button(SteamyButton, bool),
    Trigger(SteamyTrigger),
    PadStickF32(SteamyPadStickF32),
    Orientation(steamy_base::Angles),
    Acceleration(steamy_base::Angles),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SteamyButton {
    A,
    B,
    X,
    Y,

    Down,
    Left,
    Right,
    Up,

    LeftPadPressed,
    LeftPadTouch,

    StickPressed,
    StickTouch,

    RightPadPressed,
    RightPadTouch,

    Back,
    Home,
    Forward,

    BumperLeft,
    BumperRight,

    GripLeft,
    GripRight,

    TriggerLeft,
    TriggerRight,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SteamyTrigger {
    Left(f32),
    Right(f32),
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SteamyPadStickF32 {
    /// The left pad.
    LeftPadX(f32),
    LeftPadY(f32),

    /// The right pad.
    RightPadX(f32),
    RightPadY(f32),

    StickX(f32),
    StickY(f32),
}




