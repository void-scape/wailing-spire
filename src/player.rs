use crate::{
    animation::{AnimationController, AnimationPlugin},
    physics::prelude::*,
};
use crate::{spire, TILE_SIZE};
use bevy::prelude::*;
use bevy_pixel_gfx::{anchor::AnchorTarget, camera::CameraOffset, zorder::YOrigin};
use leafwing_input_manager::{
    plugin::InputManagerPlugin,
    prelude::{ActionState, InputMap},
    Actionlike,
};
use std::hash::Hash;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<spire::Knight, Player>()
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                AnimationPlugin::<PlayerAnimation>::default(),
            ))
            .add_systems(Update, (update, smooth_camera_offset).chain());
    }
}

pub const MAX_X_VEL: f32 = 100.;
pub const MAX_Y_VEL: f32 = 250.;
pub const RUN_FORCE: f32 = 40.;
pub const JUMP_FORCE: f32 = 800.;

#[derive(Default, Component)]
#[require(AnimationController<PlayerAnimation>(animation_controller), Direction)]
#[require(ActionState<Action>, InputMap<Action>(input_map))]
#[require(Velocity, TriggerLayer(|| TriggerLayer(0)), DynamicBody, Collider(collider))]
#[require(Friction(|| Friction(0.)), MaxVelocity(|| MaxVelocity(Vec2::new(MAX_X_VEL, MAX_Y_VEL))))]
#[require(CameraOffset(|| CameraOffset(Vec2::new(TILE_SIZE, -TILE_SIZE))))]
#[require(YOrigin(|| YOrigin(-TILE_SIZE * 1.9)))]
#[require(AnchorTarget)]
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
        (Action::Run(Direction::Left), KeyCode::KeyA),
        (Action::Run(Direction::Right), KeyCode::KeyD),
        (Action::Jump, KeyCode::Space),
        (Action::Interact, KeyCode::KeyE),
    ])
}

fn collider() -> Collider {
    Collider::from_rect(
        Vec2::new(TILE_SIZE * 0.75, -TILE_SIZE * 1.75),
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
    Run(Direction),
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

    // pub fn from_velocity(velocity: Vec2) -> Self {
    //     #[allow(clippy::collapsible_else_if)]
    //     if velocity.x.abs() > velocity.y.abs() {
    //         if velocity.x > 0.0 {
    //             Direction::Right
    //         } else {
    //             Direction::Left
    //         }
    //     } else {
    //         if velocity.y > 0.0 {
    //             Direction::Up
    //         } else {
    //             Direction::Down
    //         }
    //     }
    // }
}

fn smooth_camera_offset(player: Option<Single<(&Direction, &mut CameraOffset)>>) {
    if let Some((direction, mut cam_offset)) = player.map(|i| i.into_inner()) {
        let target = direction.into_unit_vec2() * TILE_SIZE;

        // gradually approach the target offset
        let delta = (target - cam_offset.0) * 0.05;
        cam_offset.0 += delta;
    }
}

fn update(
    player: Option<
        Single<
            (
                &ActionState<Action>,
                &mut Acceleration,
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
        action_state,
        mut acceleration,
        mut animations,
        mut sprite,
        mut direction,
        mut velocity,
        grounded,
    )) = player.map(|p| p.into_inner())
    {
        for action in action_state.get_pressed() {
            if let Action::Run(dir) = action {
                if *set_idle {
                    animations.set_animation(PlayerAnimation::Run);
                }
                acceleration.apply_force(dir.into_unit_vec2() * RUN_FORCE);
                *direction = dir;
                *set_idle = false;
            }
        }

        if !*set_idle
            && !action_state.get_pressed().iter().any(|a| {
                std::mem::discriminant(a)
                    == std::mem::discriminant(&Action::Run(Direction::default()))
            })
        {
            *set_idle = true;
            velocity.0.x = 0.;
            animations.set_animation(PlayerAnimation::Idle)
        }

        sprite.flip_x = Direction::Left == *direction;

        for action in action_state.get_just_pressed() {
            match action {
                Action::Jump => {
                    if grounded.is_some() {
                        acceleration.apply_force(Vec2::Y * JUMP_FORCE);
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
