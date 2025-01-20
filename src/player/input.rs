use super::{Action, Selector};
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
};
use leafwing_input_manager::prelude::{
    GamepadStick, InputMap, VirtualDPad, WithDualAxisProcessingPipelineExt,
};

pub const KEYBOARD_SELECTOR_MAP: &[(Selector, KeyCode)] = &[
    (Selector(0), KeyCode::KeyH),
    (Selector(1), KeyCode::KeyJ),
    (Selector(2), KeyCode::KeyK),
    (Selector(3), KeyCode::KeyL),
];

pub const CONTROLLER_SELECTOR_MAP: &[(Selector, GamepadButton)] = &[
    (Selector(0), GamepadButton::North),
    (Selector(1), GamepadButton::South),
    (Selector(2), GamepadButton::West),
    (Selector(3), GamepadButton::East),
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputType {
    Controller,
    #[default]
    Keyboard,
}

#[derive(Debug, Default, Resource)]
pub struct ActiveInputType(InputType);

impl ActiveInputType {
    pub fn ty(&self) -> InputType {
        self.0
    }
}

pub(super) fn update_active_input_type(
    mut input_type: ResMut<ActiveInputType>,
    mut keyboard: EventReader<KeyboardInput>,
    mut controller: EventReader<GamepadEvent>,
) {
    if keyboard.read().next().is_some() && input_type.0 != InputType::Keyboard {
        input_type.0 = InputType::Keyboard;
    } else if controller.read().next().is_some() && input_type.0 != InputType::Controller {
        input_type.0 = InputType::Controller;
    }
}

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

    for (selector, button) in CONTROLLER_SELECTOR_MAP.iter() {
        map = map.with(Action::Hook(*selector), *button);
    }

    for (selector, button) in KEYBOARD_SELECTOR_MAP.iter() {
        map = map.with(Action::Hook(*selector), *button);
    }

    map
}
