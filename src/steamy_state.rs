use crate::configs::AxisCorrectionConfigs;
use crate::math_ops::coord_to_f32;
use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};
use color_eyre::eyre::{bail, Result};
use steamy_base::Axis;

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

pub type SteamyInputCoord = i16;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
struct _Pair {
    x: SteamyInputCoord,
    y: SteamyInputCoord,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct SteamyRawPadStick {
    pub _left_pad_max: _Pair,
    pub _left_pad_min: _Pair,
    pub _right_pad_max: _Pair,
    pub _right_pad_min: _Pair,

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

fn _print_max_min(
    cur_value: SteamyInputCoord,
    is_x: bool,
    min_coords: &mut _Pair,
    max_coords: &mut _Pair,
) {
    match is_x {
        true => {
            if cur_value > max_coords.x {
                println!("Max X: {}", cur_value);
                max_coords.x = cur_value;
            }

            if cur_value < min_coords.x {
                println!("Min X: {}", cur_value);
                min_coords.x = cur_value;
            }
        }
        false => {
            if cur_value > max_coords.y {
                println!("Max Y: {}", cur_value);
                max_coords.y = cur_value;
            }

            if cur_value < min_coords.y {
                println!("Min Y: {}", cur_value);
                min_coords.y = cur_value;
            }
        }
    }
}

fn _print_cur_axis(axis: &Axis) {
    println!("X: {}, Y: {}", axis.x, axis.y)
}

// #[inline(always)]
// fn steamy_coord_to_f32(value: SteamyInputCoord, correction: SteamyInputCoord, use_correction: bool) -> Result<f32> {
//     let mut value = value as f32; //first convert to f32 otherwise i16 will overflow
//
//     if use_correction {
//         value += correction as f32;
//         value = value.clamp(MIN_COORD_VALUE, MAX_COORD_VALUE);
//     } else {
//         if value > MAX_COORD_VALUE {
//             bail!(
//                 "Cur value: '{}' is higher than Max allowed: '{}'",
//                 value,
//                 MAX_COORD_VALUE
//             )
//         };
//         if value < MIN_COORD_VALUE {
//             bail!(
//                 "Cur value: '{}' is lower than Min allowed: '{}'",
//                 value,
//                 MIN_COORD_VALUE
//             )
//         };
//     }
//
//     Ok(if value == 0.0 {
//         0.0
//     } else if value > 0.0 {
//         value / MAX_COORD_VALUE
//     } else {
//         value / MIN_COORD_VALUE.abs()
//     })
// }

impl SteamyState {
    #[inline]
    pub fn update(
        &mut self,
        state: steamy_base::State,
        is_left_pad: bool,
    ) -> Result<Vec<SteamyEvent>> {
        let mut events = Vec::new();

        match state {
            steamy_base::State::Power(true) => {
                events.push(SteamyEvent::Connected);
            }

            steamy_base::State::Power(false) => {
                events.push(SteamyEvent::Disconnected);
            }

            steamy_base::State::Input {
                buttons,
                trigger,
                pad,
                orientation,
                acceleration,
                ..
            } => {
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

                if is_left_pad {
                    if self.pad_stick.left_pad.x != pad.left.x {
                        // _print_max_min(pad.left.x, true, &mut self.pad_stick._left_pad_min, &mut self.pad_stick._left_pad_max);

                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::LeftPadX(
                            coord_to_f32(pad.left.x),
                        )));
                    }
                    if self.pad_stick.left_pad.y != pad.left.y {
                        // _print_max_min(pad.left.y, false, &mut self.pad_stick._left_pad_min, &mut self.pad_stick._left_pad_max);

                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::LeftPadY(
                            coord_to_f32(pad.left.y),
                        )));
                    }
                    self.pad_stick.left_pad = pad.left;
                } else {
                    if self.pad_stick.stick.x != pad.left.x {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::StickX(
                            coord_to_f32(pad.left.x),
                        )));
                    }
                    if self.pad_stick.stick.y != pad.left.y {
                        events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::StickY(
                            coord_to_f32(pad.left.y),
                        )));
                    }
                    self.pad_stick.stick = pad.left;
                }

                if self.pad_stick.right_pad.x != pad.right.x {
                    // _print_max_min(pad.right.x, true, &mut self.pad_stick._right_pad_min, &mut self.pad_stick._right_pad_max);

                    events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::RightPadX(
                        coord_to_f32(pad.right.x),
                    )));
                }
                if self.pad_stick.right_pad.y != pad.right.y {
                    // _print_max_min(pad.right.y, false, &mut self.pad_stick._right_pad_min, &mut self.pad_stick._right_pad_max);

                    events.push(SteamyEvent::PadStickF32(SteamyPadStickF32::RightPadY(
                        coord_to_f32(pad.right.y),
                    )));
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

                // _print_cur_axis(&pad.left);
                // _print_cur_axis(&pad.right);
            }

            steamy_base::State::Idle { .. } => (),
        }

        Ok(events)
    }
}
