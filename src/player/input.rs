use super::{Action, Selector};
use bevy::prelude::*;
use leafwing_input_manager::prelude::{
    GamepadStick, InputMap, VirtualDPad, WithDualAxisProcessingPipelineExt,
};

pub const XBOX_SELECTOR_MAP: &[(Selector, GamepadButton)] = &[
    (Selector(0), GamepadButton::North),
    (Selector(1), GamepadButton::South),
    (Selector(2), GamepadButton::West),
    (Selector(3), GamepadButton::East),
];

pub(super) fn input_map() -> InputMap<Action> {
    let mut map = InputMap::new([
        (Action::Jump, KeyCode::Space),
        (Action::Interact, KeyCode::KeyE),
        (Action::Dash, KeyCode::KeyC),
    ])
    .with(Action::Jump, GamepadButton::RightTrigger)
    // .with(Action::Jump, GamepadButton::South)
    .with(Action::Interact, GamepadButton::LeftTrigger)
    // .with(Action::Dash, GamepadButton::West)
    .with_dual_axis(
        Action::Aim,
        GamepadStick::RIGHT.with_deadzone_symmetric(0.3),
    )
    .with_dual_axis(Action::Run, GamepadStick::LEFT.with_deadzone_symmetric(0.3))
    .with_dual_axis(Action::Run, VirtualDPad::wasd());

    for (selector, button) in XBOX_SELECTOR_MAP.iter() {
        map = map.with(Action::Hook(*selector), *button);
    }

    map
}
