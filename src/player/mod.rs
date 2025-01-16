use crate::spikes;
use crate::TILE_SIZE;
use crate::{
    animation::{AnimationController, AnimationPlugin},
    physics::prelude::*,
};
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_ldtk_scene::extract::levels::LevelMeta;
use bevy_ldtk_scene::levels::Level;
use bevy_pixel_gfx::camera::MainCamera;
use bevy_pixel_gfx::{anchor::AnchorTarget, camera::CameraOffset};
use bevy_tween::combinator::tween;
use bevy_tween::prelude::*;
use combo::Combo;
use health::Health;
use interpolate::sprite_color_to;
use leafwing_input_manager::prelude::{
    GamepadStick, VirtualDPad, WithDualAxisProcessingPipelineExt,
};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

mod combo;
pub mod health;
pub mod hook;

pub use combo::ComboCollision;
pub use health::HookedDamage;
pub use hook::{HookTarget, HookTargetCollision, OccludeHookTarget};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<crate::spire::Knight, Player>()
            .add_event::<hook::HookTargetCollision>()
            .init_resource::<hook::ViableTargets>()
            .insert_resource(hook::ShowHook::default())
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
            ))
            .add_systems(Startup, hook::spawn_hook)
            .add_systems(
                Update,
                (
                    (
                        manage_brushing_move,
                        (update, homing_movement, update_current_level),
                        (jump, dash),
                    )
                        .chain(),
                    (
                        hook::gather_viable_targets,
                        hook::move_hook,
                        hook::terminal_velocity,
                        hook::collision_hook,
                        combo::combo,
                    )
                        .chain(),
                    health::death,
                    health::hook_collision,
                    hook::show_hook,
                ),
            )
            .add_systems(
                PostUpdate,
                move_camera.before(TransformSystem::TransformPropagate),
            );
    }
}

const CAMERA_SPEED: f32 = 0.1;

const MAX_VEL: f32 = 300.;
const WALL_IMPULSE: f32 = 400.;
const WALK_SPEED: f32 = 130.;
const AIR_ACCEL: f32 = 0.08;
const AIR_DAMPING: f32 = 0.04;
const SLIDE_SPEED: f32 = 40.;
const WALL_STICK_TIME: f32 = 0.20;

/// The angle (in terms of the dot product)
/// at which the player should break lock-on
/// with a target when hitting a static body.
const BREAK_ANGLE: f32 = 0.66;

const JUMP_SPEED: f32 = 200.;
const JUMP_MAX_DURATION: f32 = 0.2;

const DASH_DURATION: f32 = 0.1;
const DASH_SPEED: f32 = 1000.;
/// Divides the velocity by this factor _once_ after a dash is completed.
const DASH_DECAY: f32 = 2.;

#[derive(Default, Component)]
#[require(AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(Velocity, Gravitational, TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(collider))]
#[require(Friction(|| Friction(0.)), MaxVelocity(|| MaxVelocity(Vec2::splat(MAX_VEL))))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE * 2.))))]
#[require(AnchorTarget)]
#[require(BrushingMove)]
#[require(layers::CollidesWith<layers::Wall>)]
#[require(layers::Player)]
#[require(Combo)]
#[require(Health(|| Health::PLAYER))]
#[require(layers::CollidesWith<spikes::Spike>)]
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
    .with(Action::Jump, GamepadButton::South)
    .with(Action::Interact, GamepadButton::North)
    .with(Action::Dash, GamepadButton::East)
    .with_dual_axis(Action::Run, GamepadStick::LEFT.with_deadzone_symmetric(0.3))
    .with_dual_axis(Action::Run, VirtualDPad::wasd())
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

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Run,
    Jump,
    Dash,
    Interact,
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

// fn smooth_camera_offset(player: Option<Single<(&Direction, &mut CameraOffset)>>) {
//     if let Some((direction, mut cam_offset)) = player.map(|i| i.into_inner()) {
//         let target = direction.into_unit_vec2() * TILE_SIZE;
//
//         // gradually approach the target offset
//         let delta = (target - cam_offset.0) * 0.05;
//         cam_offset.0 += delta;
//     }
// }

#[derive(Component)]
struct CurrentLevel(LevelMeta);

fn update_current_level(
    mut commands: Commands,
    player: Query<(Entity, &GlobalTransform), With<Player>>,
    level_query: Query<(&GlobalTransform, &Level)>,
) {
    let Ok((entity, player)) = player.get_single() else {
        return;
    };

    if let Some(level) = level_query
        .iter()
        .find(|(t, l)| l.meta().rect(t).contains(player.translation().xy()))
        .map(|(_, l)| l)
    {
        commands.entity(entity).insert(CurrentLevel(*level.meta()));
    }
}

fn move_camera(
    mut cam: Query<&mut Transform, With<MainCamera>>,
    player: Query<(&GlobalTransform, &CurrentLevel), (With<Player>, Without<MainCamera>)>,
    level_query: Query<(&GlobalTransform, &Level)>,
) {
    let Ok(mut cam) = cam.get_single_mut() else {
        return;
    };

    let Ok((player, level)) = player.get_single() else {
        return;
    };

    if let Some(level_transform) = level_query
        .iter()
        .find(|(_, l)| l.uid() == level.0.uid)
        .map(|(t, _)| t)
    {
        let x = level.0.size.x / 2. + level_transform.translation().x;
        // let x = player.translation().x;
        let target_position = Vec3::new(x, player.translation().y + TILE_SIZE * 1.5, 0.);
        let delta = target_position - cam.translation;

        cam.translation += delta * CAMERA_SPEED;
    }
}

#[derive(Component)]
struct Jumping;

#[derive(Component)]
struct Dashing(Option<Vec2>);

#[derive(Component, Default, Debug)]
struct BrushingMove(Stopwatch);

/// The player is homing in on a hooked target.
#[derive(Component)]
struct Homing {
    target: Entity,
    starting_velocity: Vec2,
}

fn homing_movement(
    mut player: Query<
        (
            Entity,
            &mut Homing,
            &GlobalTransform,
            &Collider,
            &mut Velocity,
            &Resolution,
            Option<&Collision>,
        ),
        With<Player>,
    >,
    target: Query<(&GlobalTransform, &Collider, Option<&Velocity>), Without<Player>>,
    mut commands: Commands,
) {
    let Ok((player, mut homing, player_trans, player_collider, mut player_vel, res, collision)) =
        player.get_single_mut()
    else {
        return;
    };

    let Ok((target_trans, target_collider, target_vel)) = target.get(homing.target) else {
        warn!("A homing target is missing one or more components");
        return;
    };

    let target = target_trans.compute_transform();
    let abs_target = target_collider.absolute(&target);
    let abs_player = player_collider.global_absolute(player_trans);

    let vector = (abs_target.center() - abs_player.center()).normalize_or_zero();

    if collision.is_some() {
        let contact_normal = res.get().normalize_or_zero();
        let bounce_dot = (contact_normal * -1.0).dot(vector);

        if bounce_dot > BREAK_ANGLE {
            commands.entity(player).remove::<Homing>();
            // TODO: get a nice bounce
            // player_vel.0 = contact_normal * 100.;
            return;
        }
    }

    // we have some velocity damping
    homing.starting_velocity *= 0.97;

    player_vel.0 =
        vector * 500. + target_vel.map(|t| t.0).unwrap_or_default() + homing.starting_velocity;
}

fn manage_brushing_move(
    player: Option<
        Single<
            (
                &ActionState<Action>,
                &mut BrushingMove,
                Option<&Grounded>,
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
            ),
            With<Player>,
        >,
    >,
    time: Res<Time>,
) {
    let Some((action, mut brushing_move, grounded, brushing_left, brushing_right)) =
        player.map(|p| p.into_inner())
    else {
        return;
    };

    let axis_pair = action.clamped_axis_pair(&Action::Run);
    let direction = Direction::from_vec(axis_pair);

    if grounded.is_none()
        && (brushing_left.is_some() && direction == Direction::Right
            || brushing_right.is_some() && direction == Direction::Left)
    {
        brushing_move.0.tick(time.delta());
    } else {
        brushing_move.0.reset();
    }
}

fn update(
    mut commands: Commands,
    player: Option<
        Single<
            (
                Entity,
                &ActionState<Action>,
                &mut AnimationController<PlayerAnimation>,
                &mut Sprite,
                &mut Direction,
                &mut Velocity,
                &BrushingMove,
                Option<&Grounded>,
                Option<&Dashing>,
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
            ),
            (With<Player>, Without<Homing>),
        >,
    >,
    mut set_idle: Local<bool>,
) {
    if let Some((
        entity,
        action_state,
        mut animations,
        mut sprite,
        mut direction,
        mut velocity,
        brushing_move,
        grounded,
        dashing,
        brushing_left,
        brushing_right,
    )) = player.map(|p| p.into_inner())
    {
        let axis_pair = action_state.clamped_axis_pair(&Action::Run);
        if axis_pair.x != 0. {
            if *set_idle {
                animations.set_animation(PlayerAnimation::Run);
            }
            let dir = Direction::from_vec(axis_pair);

            if grounded.is_some() {
                velocity.0.x = dir.into_unit_vec2().x * WALK_SPEED;
            } else if dashing.is_none()
                && !((brushing_left.is_some() || brushing_right.is_some())
                    && brushing_move.0.elapsed_secs() <= WALL_STICK_TIME)
            {
                match dir {
                    Direction::Right => {
                        if velocity.0.x < WALK_SPEED {
                            velocity.0.x = (velocity.0.x + AIR_ACCEL * WALK_SPEED).min(WALK_SPEED);
                        }
                    }
                    Direction::Left => {
                        if velocity.0.x > -WALK_SPEED {
                            velocity.0.x = (velocity.0.x - AIR_ACCEL * WALK_SPEED).max(-WALK_SPEED);
                        }
                    }
                }
            }

            *direction = dir;
            *set_idle = false;
        } else {
            *set_idle = true;

            if grounded.is_some() {
                velocity.0.x = 0.;
            }

            animations.set_animation(PlayerAnimation::Idle)
        }

        if grounded.is_none() {
            // air damping
            velocity.0.x *= 1.0 - AIR_DAMPING;
        }

        if brushing_left.is_some() || brushing_right.is_some() {
            velocity.0.y = velocity.0.y.max(-SLIDE_SPEED);
        }

        sprite.flip_x = Direction::Left == *direction;

        for action in action_state.get_just_pressed() {
            match action {
                Action::Jump => {
                    if grounded.is_some() {
                        commands.entity(entity).insert(Jumping);
                    } else if brushing_left.is_some() {
                        commands.entity(entity).insert(Jumping);
                        velocity.0.x += WALL_IMPULSE;
                    } else if brushing_right.is_some() {
                        commands.entity(entity).insert(Jumping);
                        velocity.0.x -= WALL_IMPULSE;
                    }
                }
                Action::Dash => {
                    commands
                        .entity(entity)
                        .insert(Dashing((axis_pair != Vec2::ZERO).then_some(axis_pair)));
                }
                _ => {}
            }
        }
    }
}

fn jump(
    mut commands: Commands,
    player: Option<
        Single<
            (Entity, &ActionState<Action>, &mut Velocity),
            (With<Player>, With<Jumping>, Without<Dashing>),
        >,
    >,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    if let Some((entity, action_state, mut velocity)) = player.map(|p| p.into_inner()) {
        let timer =
            timer.get_or_insert_with(|| Timer::from_seconds(JUMP_MAX_DURATION, TimerMode::Once));

        timer.tick(time.delta());
        if timer.finished()
            || action_state
                .get_pressed()
                .iter()
                .all(|a| *a != Action::Jump)
        {
            commands.entity(entity).remove::<Jumping>();
            timer.reset();
            velocity.0.y /= 2.;
            return;
        }

        // acceleration.apply_force(Vec2::Y * JUMP_FORCE);
        velocity.0.y = JUMP_SPEED;
    }
}

fn dash(
    mut commands: Commands,
    server: Res<AssetServer>,
    player: Option<
        Single<
            (
                Entity,
                &GlobalTransform,
                &Sprite,
                &mut Velocity,
                &ActionState<Action>,
                Option<&Dashing>,
                Option<&Grounded>,
            ),
            With<Player>,
        >,
    >,
    mut reader: EventReader<HookTargetCollision>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut spawn_ghost_timer: Local<Option<Timer>>,
    mut ghost_z: Local<usize>,
    mut dash_reset: Local<bool>,
    mut last_dir: Local<Vec2>,
) {
    if let Some((entity, transform, sprite, mut velocity, action_state, dash, grounded)) =
        player.map(|p| p.into_inner())
    {
        let axis_pair = action_state.clamped_axis_pair(&Action::Run);
        if axis_pair != Vec2::ZERO {
            *last_dir = axis_pair;
        }

        if grounded.is_some() || reader.read().next().is_some() {
            *dash_reset = true;
            *ghost_z = 0;
        }

        if let Some(dash) = dash {
            if *dash_reset {
                if timer.is_none() {
                    commands.spawn((
                        AudioPlayer::new(server.load("audio/sfx/dash.wav")),
                        PlaybackSettings::DESPAWN,
                    ));
                }

                let dash_timer = timer
                    .get_or_insert_with(|| Timer::from_seconds(DASH_DURATION, TimerMode::Once));
                dash_timer.tick(time.delta());
                if dash_timer.finished() {
                    *dash_reset = false;
                    commands.entity(entity).remove::<Dashing>();
                    *timer = None;
                    velocity.0 /= DASH_DECAY;
                    return;
                }

                let dash_vec = dash.0.unwrap_or_else(|| *last_dir);
                velocity.0 = dash_vec * DASH_SPEED;

                let ghost_timer = spawn_ghost_timer.get_or_insert_with(|| {
                    Timer::from_seconds(DASH_DURATION / 5., TimerMode::Repeating)
                });
                ghost_timer.tick(time.delta());
                if ghost_timer.just_finished() {
                    let ghost = commands
                        .spawn((
                            sprite.clone(),
                            Transform::from_translation(
                                transform.translation().xy().extend(*ghost_z as f32),
                            ),
                        ))
                        .id()
                        .into_target();
                    *ghost_z += 1;

                    commands.animation().insert(tween(
                        Duration::from_secs_f32(0.2),
                        EaseKind::Linear,
                        ghost
                            .state(Color::srgba(0., 0., 1., 1.))
                            .with(sprite_color_to(Color::srgba(0., 0., 1., 0.))),
                    ));
                }
            } else {
                commands.entity(entity).remove::<Dashing>();
            }
        }
    }
}
