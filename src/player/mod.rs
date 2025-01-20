use self::movement::Homing;
use crate::animation::{AnimationController, AnimationPlugin};
use crate::{spikes, TILE_SIZE};
use ::selector::Selector;
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_pixel_gfx::{anchor::AnchorTarget, camera::CameraOffset};
use combo::Combo;
use health::Health;
use layers::RegisterPhysicsLayer;
use layers::TriggersWith;
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use movement::BrushingMove;
use physics::{prelude::*, trigger::Trigger};
use physics::{Physics, PhysicsSystems};
use std::hash::Hash;

mod camera;
pub mod combo;
pub mod health;
pub mod hook;
mod input;
mod movement;
mod selector;

pub const MAX_VEL: f32 = 300.;

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct PlayerSettings {
    pub camera_speed: f32,

    pub wall_impulse: f32,
    pub walk_speed: f32,
    pub air_accel: f32,
    pub air_damping: f32,
    pub slide_speed: f32,
    pub wall_stick_time: f32,

    /// The angle (in terms of the dot product)
    /// at which the player should break lock-on
    /// with a target when hitting a static body.
    pub break_angle: f32,

    pub jump_speed: f32,
    pub jump_max_duration: f32,

    pub dash_duration: f32,
    pub dash_speed: f32,
    /// Divides the velocity by this factor _once_ after a dash is completed.
    pub dash_decay: f32,

    /// Maximum distance for a hook target
    pub target_threshold: f32,
    pub terminal_velocity2_threshold: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            camera_speed: 0.1,
            wall_impulse: 400.,
            walk_speed: 130.,
            air_accel: 0.08,
            air_damping: 0.04,
            slide_speed: 40.,
            wall_stick_time: 0.20,
            break_angle: 0.66,
            jump_speed: 200.,
            jump_max_duration: 0.2,
            dash_duration: 0.1,
            dash_speed: 1000.,
            dash_decay: 2.,
            target_threshold: 256.0,
            terminal_velocity2_threshold: 60_000.,
        }
    }
}

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
            .init_resource::<::selector::SelectorTick>()
            .insert_resource(::selector::MaxSelectors(4))
            .insert_resource(hook::ShowHook::default())
            .insert_resource(input::ActiveInputType::default())
            .insert_resource(PlayerSettings::default())
            .register_type::<PlayerSettings>()
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
                movement::MovementPlugin,
                ResourceInspectorPlugin::<PlayerSettings>::default(),
            ))
            .add_systems(
                Startup,
                (
                    hook::spawn_hook,
                    selector::spawn_selectors,
                    selector::insert_texture_cache,
                ),
            )
            .add_systems(PreUpdate, input::update_active_input_type)
            .add_systems(
                Update,
                (
                    (
                        direction.before(PlayerSystems::Movement),
                        flip_sprite.after(PlayerSystems::Movement),
                    ),
                    (
                        health::no_shield_collision,
                        hook::collision_hook,
                        (
                            hook::gather_viable_targets,
                            hook::move_hook,
                            selector::clear_removed_entities,
                            ::selector::calculate_selectors,
                            selector::trigger_hook,
                            combo::combo,
                            camera::update_current_level,
                            health::death,
                            hook::show_hook,
                        ),
                    )
                        .chain(),
                ),
            )
            .add_systems(
                Physics,
                (
                    selector::add_selectors,
                    camera::move_camera.after(PhysicsSystems::Collision),
                ),
            );
    }
}

#[derive(Default, Component)]
#[require(AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input::input_map))]
#[require(Velocity, Gravitational, DynamicBody, Collider(collider), Trigger(trigger), TriggersWith<layers::Player>)]
#[require(MaxVelocity(|| MaxVelocity(Vec2::splat(MAX_VEL))))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE * 2.))))]
#[require(AnchorTarget)]
#[require(layers::CollidesWith<layers::Wall>, layers::CollidesWith<spikes::Spike>)]
#[require(layers::Player)]
#[require(BrushingMove)]
#[require(Combo)]
#[require(Health(|| Health::PLAYER))]
#[require(::selector::SelectorSource)]
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

fn collider() -> Collider {
    Collider::from_rect(
        Vec2::new(TILE_SIZE * 0.75, -TILE_SIZE * 1.25),
        Vec2::splat(TILE_SIZE / 2.),
    )
}

fn trigger() -> Trigger {
    Trigger(Collider::from_rect(
        Vec2::new(TILE_SIZE / 2., -TILE_SIZE / 1.5),
        Vec2::splat(TILE_SIZE),
    ))
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum PlayerAnimation {
    Run,
    Idle,
    Hit,
    Death,
}

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
    None,
    Right,
    Left,
}

impl Direction {
    pub fn unit(self) -> Vec2 {
        match self {
            Self::None => Vec2::ZERO,
            Self::Left => Vec2::NEG_X,
            Self::Right => Vec2::X,
        }
    }

    pub fn from_vec(vec: Vec2) -> Self {
        match vec {
            Vec2::ZERO => Self::None,
            vec => {
                if vec.x > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                }
            }
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

fn flip_sprite(
    player: Option<Single<(&mut Sprite, &Direction), With<Player>>>,
    mut prev_left: Local<bool>,
) {
    let Some((mut sprite, direction)) = player.map(|p| p.into_inner()) else {
        return;
    };

    sprite.flip_x = Direction::Left == *direction || *prev_left;

    if *direction == Direction::Left {
        *prev_left = true;
    } else if *direction == Direction::Right {
        *prev_left = false;
    }
}
