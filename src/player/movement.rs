use super::hook::HookTargetCollision;
use super::params::*;
use super::Action;
use super::Direction;
use super::Player;
use super::PlayerSystems;
use crate::physics::prelude::*;
use crate::physics::TimeScale;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_tween::combinator::tween;
use bevy_tween::prelude::*;
use interpolate::sprite_color_to;
use leafwing_input_manager::prelude::ActionState;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                brushing,
                homing,
                jumping,
                dashing,
                air_strafe,
                wall_slide,
                walk,
                air_damping,
            )
                .in_set(PlayerSystems::Movement),
        );
    }
}

/// The player is homing in on a hooked target.
#[derive(Component)]
pub struct Homing {
    target: Entity,
    starting_velocity: Vec2,
}

impl Homing {
    pub fn new(target: Entity, starting_velocity: Vec2) -> Self {
        Self {
            target,
            starting_velocity,
        }
    }

    pub fn target(&self) -> Entity {
        self.target
    }
}

fn homing(
    player: Option<
        Single<
            (
                Entity,
                &mut Homing,
                &GlobalTransform,
                &Collider,
                &mut Velocity,
                &Resolution,
                Option<&Collision<layers::Wall>>,
            ),
            With<Player>,
        >,
    >,
    target: Query<(&GlobalTransform, &Collider, Option<&Velocity>), Without<Player>>,
    mut commands: Commands,
) {
    let Some((player, mut homing, player_trans, player_collider, mut player_vel, res, collision)) =
        player.map(|p| p.into_inner())
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

#[derive(Debug, Clone, Component)]
pub struct BrushingMove(Stopwatch);

impl BrushingMove {
    pub fn watch(&self) -> &Stopwatch {
        &self.0
    }
}

fn brushing(
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
    scale: Single<&TimeScale>,
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
        brushing_move
            .0
            .tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
    } else {
        brushing_move.0.reset();
    }
}

#[derive(Component)]
pub struct Jumping;

fn jumping(
    mut commands: Commands,
    player: Option<
        Single<
            (Entity, &ActionState<Action>, &mut Velocity),
            (With<Player>, With<Jumping>, Without<Dashing>),
        >,
    >,
    time: Res<Time>,
    scale: Single<&TimeScale>,
    mut timer: Local<Option<Timer>>,
) {
    if let Some((entity, action_state, mut velocity)) = player.map(|p| p.into_inner()) {
        let timer =
            timer.get_or_insert_with(|| Timer::from_seconds(JUMP_MAX_DURATION, TimerMode::Once));

        timer.tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
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

#[derive(Default, Component)]
pub struct Dashing(Option<Vec2>);

impl Dashing {
    pub fn new(direction: Option<Vec2>) -> Self {
        Self(direction)
    }
}

fn dashing(
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
    scale: Single<&TimeScale>,
    mut timer: Local<Option<Timer>>,
    mut spawn_ghost_timer: Local<Option<Timer>>,
    mut ghost_z: Local<usize>,
    mut dash_reset: Local<bool>,
    mut last_dir: Local<Vec2>,
) {
    let Some((entity, transform, sprite, mut velocity, action_state, dash, grounded)) =
        player.map(|p| p.into_inner())
    else {
        return;
    };

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

            let dash_timer =
                timer.get_or_insert_with(|| Timer::from_seconds(DASH_DURATION, TimerMode::Once));
            dash_timer.tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
            if dash_timer.finished() {
                *dash_reset = false;
                commands.entity(entity).remove::<Dashing>();
                *timer = None;
                velocity.0 /= DASH_DECAY;
                return;
            }

            let dash_vec = dash.0.unwrap_or_else(|| *last_dir);
            velocity.0 = dash_vec.normalize_or_zero() * DASH_SPEED;

            let ghost_timer = spawn_ghost_timer.get_or_insert_with(|| {
                Timer::from_seconds(DASH_DURATION / 5., TimerMode::Repeating)
            });
            ghost_timer.tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
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

fn air_strafe(
    player: Option<
        Single<
            (&mut Velocity, &Direction, &BrushingMove),
            (
                With<Player>,
                Without<BrushingLeft>,
                Without<BrushingRight>,
                Without<Dashing>,
            ),
        >,
    >,
) {
    let Some((mut velocity, direction, brushing_move)) = player.map(|p| p.into_inner()) else {
        return;
    };

    if brushing_move.watch().elapsed_secs() <= WALL_STICK_TIME {
        match direction {
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
}

fn wall_slide(
    player: Option<
        Single<&mut Velocity, (With<Player>, Or<(With<BrushingLeft>, With<BrushingRight>)>)>,
    >,
) {
    let Some(mut velocity) = player else {
        return;
    };

    velocity.0.y = velocity.0.y.max(-SLIDE_SPEED);
}

fn walk(
    player: Option<
        Single<
            (&mut Velocity, &ActionState<Action>),
            (With<Player>, With<Grounded>, Without<Dashing>),
        >,
    >,
) {
    let Some((mut velocity, action_state)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let axis_pair = action_state.clamped_axis_pair(&Action::Run);
    let dir = Direction::from_vec(axis_pair);
    velocity.0.x = dir.into_unit_vec2().x * WALK_SPEED;
}

fn air_damping(
    player: Option<
        Single<
            &mut Velocity,
            (
                With<Player>,
                Without<Grounded>,
                Without<BrushingLeft>,
                Without<BrushingRight>,
            ),
        >,
    >,
) {
    let Some(mut velocity) = player else {
        return;
    };

    velocity.0.x *= 1.0 - AIR_DAMPING;
}
