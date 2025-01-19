use self::movement::Dashing;
use self::movement::Homing;
use self::movement::Jumping;
use self::params::*;
use crate::spikes;
use crate::TILE_SIZE;
use crate::{
    animation::{AnimationController, AnimationPlugin},
    physics::{prelude::*, trigger::Trigger},
};
use bevy::prelude::*;
use bevy_pixel_gfx::{anchor::AnchorTarget, camera::CameraOffset};
use combo::Combo;
use health::Health;
use layers::RegisterPhysicsLayer;
use layers::TriggersWith;
use leafwing_input_manager::prelude::{
    GamepadStick, VirtualDPad, WithDualAxisProcessingPipelineExt,
};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

mod camera;
pub mod combo;
pub mod health;
pub mod hook;
mod movement;
mod params;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum PlayerSystems {
    Movement,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_trigger_layer::<layers::Player>()
            .register_required_components::<crate::spire::Knight, Player>()
            .add_event::<hook::HookTargetCollision>()
            .init_resource::<hook::ViableTargets>()
            .insert_resource(hook::ShowHook::default())
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
                movement::MovementPlugin,
            ))
            .add_systems(Startup, hook::spawn_hook)
            .add_systems(
                Update,
                (
                    (
                        hook::gather_viable_targets,
                        hook::move_hook,
                        hook::terminal_velocity,
                        hook::collision_hook,
                        combo::combo,
                    )
                        .chain(),
                    camera::update_current_level,
                    health::death,
                    health::hook_collision,
                    hook::show_hook,
                    (actions, direction).before(PlayerSystems::Movement),
                    flip_sprite.after(PlayerSystems::Movement),
                ),
            )
            .add_systems(
                PostUpdate,
                camera::move_camera.before(TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Default, Component)]
#[require(AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(Velocity, Gravitational, DynamicBody, Collider(collider), Trigger(|| Trigger(collider())), TriggersWith<layers::Player>)]
#[require(MaxVelocity(|| MaxVelocity(Vec2::splat(MAX_VEL))))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE * 2.))))]
#[require(AnchorTarget)]
#[require(layers::CollidesWith<layers::Wall>, layers::CollidesWith<spikes::Spike>)]
#[require(layers::Player)]
#[require(Combo)]
#[require(Health(|| Health::PLAYER))]
pub struct Player;

fn animation_controller() -> AnimationController<PlayerAnimation> {
    AnimationController::new(
        12.,
        [
            (PlayerAnimation::Idle, (0, 4)),
            (PlayerAnimation::Run, (16, 32)),
            (PlayerAnimation::Hit, (48, 52)),
            (PlayerAnimation::Death, (56, 60)),
        ],
    )
}

fn input_map() -> InputMap<Action> {
    InputMap::new([
        (Action::Jump, KeyCode::Space),
        (Action::Interact, KeyCode::KeyE),
        (Action::Dash, KeyCode::KeyC),
    ])
    .with(Action::Jump, GamepadButton::LeftTrigger)
    .with(Action::Jump, GamepadButton::South)
    .with(Action::Interact, GamepadButton::RightTrigger)
    .with(Action::Dash, GamepadButton::West)
    .with_dual_axis(
        Action::Aim,
        GamepadStick::RIGHT.with_deadzone_symmetric(0.3),
    )
    .with_dual_axis(Action::Run, GamepadStick::LEFT.with_deadzone_symmetric(0.3))
    .with_dual_axis(Action::Run, VirtualDPad::wasd())
    .with(Action::Hook(Selector(0)), GamepadButton::North)
    .with(Action::Hook(Selector(1)), GamepadButton::South)
    .with(Action::Hook(Selector(2)), GamepadButton::West)
    .with(Action::Hook(Selector(3)), GamepadButton::East)
}

fn collider() -> Collider {
    Collider::from_rect(
        Vec2::new(TILE_SIZE * 0.75, -TILE_SIZE * 1.25),
        Vec2::splat(TILE_SIZE / 2.),
    )
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PlayerAnimation {
    Run,
    Idle,
    Hit,
    Death,
}

/// A selector ID.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct Selector(pub usize);

/// The maximum number of hook selectors,
/// which may differ depending on the platform.
#[derive(Debug, Resource)]
pub struct MaxSelectors(usize);

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Run,
    #[actionlike(DualAxis)]
    Aim,
    Jump,
    Dash,
    Interact,
    Hook(Selector),
}

#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Component)]
pub enum Direction {
    #[default]
    Right,
    Left,
}

impl Direction {
    pub fn into_unit_vec2(self) -> Vec2 {
        match self {
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }

    pub fn from_vec(vec: Vec2) -> Self {
        if vec.x > 0.0 {
            Direction::Right
        } else {
            Direction::Left
        }
    }
}

fn actions(
    mut commands: Commands,
    player: Option<
        Single<
            (
                Entity,
                &ActionState<Action>,
                &mut Velocity,
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
            ),
            (With<Player>, Without<Homing>),
        >,
    >,
) {
    let Some((entity, action_state, mut velocity, brushing_left, brushing_right)) =
        player.map(|p| p.into_inner())
    else {
        return;
    };

    let axis_pair = action_state.clamped_axis_pair(&Action::Run);
    for action in action_state.get_just_pressed() {
        match action {
            Action::Jump => {
                commands.entity(entity).insert(Jumping);

                if brushing_left.is_some() {
                    velocity.0.x += WALL_IMPULSE;
                } else if brushing_right.is_some() {
                    velocity.0.x -= WALL_IMPULSE;
                }
            }
            Action::Dash => {
                commands
                    .entity(entity)
                    .insert(Dashing::new((axis_pair != Vec2::ZERO).then_some(axis_pair)));
            }
            _ => {}
        }
    }
}

fn direction(player: Option<Single<(&mut Direction, &ActionState<Action>), With<Player>>>) {
    let Some((mut direction, action_state)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let axis_pair = action_state.clamped_axis_pair(&Action::Run);
    *direction = Direction::from_vec(axis_pair);
}

fn flip_sprite(player: Option<Single<(&mut Sprite, &Direction), With<Player>>>) {
    let Some((mut sprite, direction)) = player.map(|p| p.into_inner()) else {
        return;
    };

    sprite.flip_x = Direction::Left == *direction;
}
