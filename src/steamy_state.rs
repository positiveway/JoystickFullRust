use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};

macro_rules! button_converter {
	($events:expr, $current:expr, $new:expr, { }) =>
		();

	($events:expr, $current:expr, $new:expr, { $flag:expr => $button:expr, $($rest:tt)* }) => (
		button_converter!($events, $current, $button, $new, $flag);
		button_converter!($events, $current, $new, { $($rest)* });
	);

	($events:expr, $current:expr, $button:expr, $new:expr, $flag:expr) => (
		if !$current.contains($flag) && $new.contains($flag) {
			$events.push(SteamyEvent::Button($button, true));
		}

		if $current.contains($flag) && !$new.contains($flag) {
			$events.push(SteamyEvent::Button($button, false));
		}
	);
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct SteamyRawPadStick {
    /// The left pad.
    pub left_pad: steamy_base::Axis,

    /// The right pad.
    pub right_pad: steamy_base::Axis,

    pub stick: steamy_base::Axis,
}

#[derive(Debug)]
pub struct SteamyState {
    buttons: steamy_base::Button,
    trigger: steamy_base::Trigger,
    pad_stick: SteamyRawPadStick,
    orientation: steamy_base::Angles,
    acceleration: steamy_base::Angles,
}

impl Default for SteamyState {
    fn default() -> Self {
        SteamyState {
            buttons: steamy_base::Button::empty(),
            trigger: Default::default(),
            pad_stick: Default::default(),
            orientation: Default::default(),
            acceleration: Default::default(),
        }
    }
}

fn i16_to_f32(value: i16) -> f32 {
    value as f32 / i16::MAX as f32
    // if value == 0{
    // 	0.0
    // } else if value > 0 {
    // 	value as f32 / i16::MAX as f32
    // } else {
    // 	value as f32 / (i16::MAX as f32 + 1.0) // TODO: check + 1
    // }
}

impl SteamyState {
    pub fn update(&mut self, state: steamy_base::State, buffer: Vec<u8>) -> Vec<SteamyEvent> {
        let mut events = Vec::new();

        match state {
            steamy_base::State::Power(true) => {
                events.push(SteamyEvent::Connected);
            }

            steamy_base::State::Power(false) => {
                events.push(SteamyEvent::Disconnected);
            }

            steamy_base::State::Input { buttons, trigger, pad, orientation, acceleration, .. } => {
                button_converter!(events, self.buttons, buttons, {
					steamy_base::Button::A => SteamyButton::A,
					steamy_base::Button::B => SteamyButton::B,
					steamy_base::Button::X => SteamyButton::X,
					steamy_base::Button::Y => SteamyButton::Y,

					steamy_base::Button::PAD_DOWN  => SteamyButton::Down,
					steamy_base::Button::PAD_LEFT  => SteamyButton::Left,
					steamy_base::Button::PAD_RIGHT => SteamyButton::Right,
					steamy_base::Button::PAD_UP    => SteamyButton::Up,

					steamy_base::Button::PAD        => SteamyButton::LeftPadPressed,
					steamy_base::Button::PAD_TOUCH  => SteamyButton::LeftPadTouch,

					steamy_base::Button::STICK       => SteamyButton::StickPressed,
					steamy_base::Button::STICK_TOUCH => SteamyButton::StickTouch,

					steamy_base::Button::TRACK       => SteamyButton::RightPadPressed,
					steamy_base::Button::TRACK_TOUCH => SteamyButton::RightPadTouch,

					steamy_base::Button::BACK    => SteamyButton::Back,
					steamy_base::Button::HOME    => SteamyButton::Home,
					steamy_base::Button::FORWARD => SteamyButton::Forward,

					steamy_base::Button::LEFT_BUMPER  => SteamyButton::BumperLeft,
					steamy_base::Button::RIGHT_BUMPER => SteamyButton::BumperRight,

					steamy_base::Button::LEFT_GRIP  => SteamyButton::GripLeft,
					steamy_base::Button::RIGHT_GRIP => SteamyButton::GripRight,

					steamy_base::Button::LEFT_TRIGGER  => SteamyButton::TriggerLeft,
					steamy_base::Button::RIGHT_TRIGGER => SteamyButton::TriggerRight,
				});

                if self.trigger.left != trigger.left {
                    events.push(SteamyEvent::Trigger(SteamyTrigger::Left(trigger.left)));
                }

                if self.trigger.right != trigger.right {
                    events.push(SteamyEvent::Trigger(SteamyTrigger::Right(trigger.right)));
                }

                let is_left_pad = buffer[6] == 8;

                if is_left_pad {
                    if self.pad_stick.left_pad.x != pad.left.x {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::LeftPadX(i16_to_f32(pad.left.x))));
                    }
                    if self.pad_stick.left_pad.y != pad.left.y {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::LeftPadY(i16_to_f32(pad.left.y))));
                    }
                    self.pad_stick.left_pad = pad.left;
                } else {
                    if self.pad_stick.stick.x != pad.left.x {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::StickX(i16_to_f32(pad.left.x))));
                    }
                    if self.pad_stick.stick.y != pad.left.y {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::StickY(i16_to_f32(pad.left.y))));
                    }
                    self.pad_stick.stick = pad.left;
                }

                if self.pad_stick.right_pad.x != pad.right.x {
                    events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::RightPadX(i16_to_f32(pad.right.x))));
                }
                if self.pad_stick.right_pad.y != pad.right.y {
                    events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::RightPadY(i16_to_f32(pad.right.y))));
                }

                if self.orientation != orientation {
                    events.push(SteamyEvent::Orientation(orientation));
                }

                if self.acceleration != acceleration {
                    events.push(SteamyEvent::Acceleration(acceleration));
                }

                self.buttons = buttons;
                self.trigger = trigger;
                self.pad_stick.right_pad = pad.right;
                self.orientation = orientation;
                self.acceleration = acceleration;
            }

            steamy_base::State::Idle { .. } =>
                (),
        }

        events
    }
}

