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
use leafwing_input_manager::prelude::{
    GamepadStick, VirtualDPad, WithDualAxisProcessingPipelineExt,
};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

mod hook;

pub use hook::HookTarget;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<crate::spire::Knight, Player>()
            .init_resource::<hook::ViableTargets>()
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
            ))
            .add_systems(Startup, hook::spawn_hook)
            .add_systems(
                Update,
                (
                    (manage_brushing_move, update, jump, update_current_level).chain(),
                    (
                        hook::gather_viable_targets,
                        hook::move_hook,
                        hook::terminal_velocity,
                        hook::collision_hook,
                    ),
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
const MAX_X_VEL: f32 = 100.;
const WALL_IMPULSE: f32 = 400.;
const AIR_ACCEL: f32 = 0.08;
const AIR_DAMPING: f32 = 0.08;
const SLIDE_SPEED: f32 = 20.;
const WALL_STICK_TIME: f32 = 0.20;

const JUMP_SPEED: f32 = 200.;
const JUMP_MAX_DURATION: f32 = 0.2;
// const JUMP_MAX_DURATION: f32 = 200.;

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
pub struct Player;

fn animation_controller() -> AnimationController<PlayerAnimation> {
    AnimationController::new(
        12.,
        [
            (PlayerAnimation::Idle, (0, 4)),
            (PlayerAnimation::Run, (16, 32)),
        ],
    )
}

fn input_map() -> InputMap<Action> {
    InputMap::new([
        (Action::Jump, KeyCode::Space),
        (Action::Interact, KeyCode::KeyE),
    ])
    .with(Action::Jump, GamepadButton::South)
    .with(Action::Interact, GamepadButton::North)
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
}

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Run,
    Jump,
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

#[derive(Component, Default, Debug)]
struct BrushingMove(Stopwatch);

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

    if grounded.is_none() && brushing_left.is_some() && direction == Direction::Right
        || brushing_right.is_some() && direction == Direction::Left
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
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
            ),
            With<Player>,
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
                velocity.0.x = dir.into_unit_vec2().x * 100.;
            } else {
                if !((brushing_left.is_some() || brushing_right.is_some())
                    && brushing_move.0.elapsed_secs() <= WALL_STICK_TIME)
                {
                    match dir {
                        Direction::Right => {
                            if velocity.0.x < MAX_X_VEL {
                                velocity.0.x =
                                    (velocity.0.x + AIR_ACCEL * MAX_X_VEL).min(MAX_X_VEL);
                            }
                        }
                        Direction::Left => {
                            if velocity.0.x > -MAX_X_VEL {
                                velocity.0.x =
                                    (velocity.0.x - AIR_ACCEL * MAX_X_VEL).max(-MAX_X_VEL);
                            }
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
                Action::Interact => {
                    info!("interacted");
                }
                _ => {}
            }
        }
    }
}

fn jump(
    mut commands: Commands,
    player: Option<
        Single<(Entity, &ActionState<Action>, &mut Velocity), (With<Player>, With<Jumping>)>,
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

// TODO: need some notion of look direction
// fn animate_cutscene(
//     mut player: Query<
//         (&mut AnimationController<PlayerAnimation>, &CutsceneVelocity),
//         (With<Player>, With<CutsceneMovement>),
//     >,
//     mut last_direction: Local<Option<Direction>>,
// ) {
//     if let Ok((mut animation, velocity)) = player.get_single_mut() {
//         let vel = velocity.0.xy();
//
//         if vel == Vec2::ZERO {
//             if let Some(dir) = *last_direction {
//                 animation.set_animation(PlayerAnimation::Idle(dir));
//                 *last_direction = None;
//             }
//         } else {
//             let direction = Direction::from_velocity(vel);
//
//             let update = match *last_direction {
//                 None => {
//                     *last_direction = Some(direction);
//                     true
//                 }
//                 Some(ld) if ld != direction => {
//                     *last_direction = Some(direction);
//                     true
//                 }
//                 _ => false,
//             };
//
//             if update {
//                 animation.set_animation(PlayerAnimation::Walk(Direction::from_velocity(vel)));
//             }
//         }
//     }
// }
