use crate::TILE_SIZE;
use crate::{
    animation::{AnimationController, AnimationPlugin},
    physics::prelude::*,
};
use bevy::ecs::component::ComponentId;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_pixel_gfx::camera::bind_camera;
use bevy_pixel_gfx::{anchor::AnchorTarget, camera::CameraOffset, zorder::YOrigin};
use leafwing_input_manager::prelude::{
    GamepadStick, VirtualDPad, WithDualAxisProcessingPipelineExt,
};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<crate::spire::Knight, Player>()
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
            ))
            .add_systems(Update, (update, jump, smooth_camera_offset).chain());
    }
}

const MAX_X_VEL: f32 = 100.;
const MAX_Y_VEL: f32 = 250.;
const RUN_FORCE: f32 = 40.;
const JUMP_SPEED: f32 = 200.;
const JUMP_MAX_DURATION: f32 = 0.2;

#[derive(Default, Component)]
#[require(AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(Velocity, TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(collider))]
#[require(Friction(|| Friction(0.)), MaxVelocity(|| MaxVelocity(Vec2::new(MAX_X_VEL, MAX_Y_VEL))))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE, -TILE_SIZE))))]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 1.9)))]
#[require(AnchorTarget)]
#[component(on_insert = on_insert_player)]
pub struct Player;

fn on_insert_player(mut world: DeferredWorld, _: Entity, _: ComponentId) {
    // world.commands().run_system_cached(bind_camera::<Player>);
}

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

fn smooth_camera_offset(player: Option<Single<(&Direction, &mut CameraOffset)>>) {
    if let Some((direction, mut cam_offset)) = player.map(|i| i.into_inner()) {
        let target = direction.into_unit_vec2() * TILE_SIZE;

        // gradually approach the target offset
        let delta = (target - cam_offset.0) * 0.05;
        cam_offset.0 += delta;
    }
}

#[derive(Component)]
struct Jumping;

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
                Option<&Grounded>,
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
        grounded,
    )) = player.map(|p| p.into_inner())
    {
        let axis_pair = action_state.clamped_axis_pair(&Action::Run);
        if axis_pair.x != 0. {
            if *set_idle {
                animations.set_animation(PlayerAnimation::Run);
            }
            let dir = Direction::from_vec(axis_pair);

            velocity.0.x = dir.into_unit_vec2().x * 100.;

            *direction = dir;
            *set_idle = false;
        } else {
            *set_idle = true;
            velocity.0.x = 0.;
            animations.set_animation(PlayerAnimation::Idle)
        }

        sprite.flip_x = Direction::Left == *direction;

        for action in action_state.get_just_pressed() {
            match action {
                Action::Jump => {
                    if grounded.is_some() {
                        commands.entity(entity).insert(Jumping);
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
