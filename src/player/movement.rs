use super::health::Knockback;
use super::hook::HookTargetCollision;
use super::Action;
use super::Direction;
use super::Player;
use super::PlayerAnimation;
use super::PlayerHurtBox;
use super::PlayerSettings;
use super::PlayerSystems;
use crate::animation::AnimationController;
use crate::health::Health;
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_enoki::prelude::OneShot;
use bevy_enoki::ParticleEffectHandle;
use bevy_enoki::ParticleSpawner;
use bevy_tween::combinator::tween;
use bevy_tween::prelude::*;
use core::f32;
use interpolate::sprite_color_to;
use leafwing_input_manager::prelude::ActionState;
use physics::Physics;
use physics::{prelude::*, TimeScale};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Physics,
            (
                (start_jump, start_dash, air_strafe),
                brushing,
                (
                    homing,
                    homing_effects,
                    dashing,
                    wall_slide,
                    jumping,
                    wall_jump_impulse,
                    ground_strafe,
                    knockback,
                    air_damping,
                    homing_hitbox,
                    // debug,
                )
                    .chain(),
            )
                .chain()
                .in_set(PlayerSystems::Movement),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HomingState {
    Hooking,
    Moving,
    Exploding,
}

/// The player is homing in on a hooked target.
#[derive(Debug, Component)]
pub struct Homing {
    target: Entity,
    starting_velocity: Vec2,
    state: HomingState,
    timer: Timer,
    ticks: usize,
    average_direction: Vec2,
}

impl Homing {
    pub fn new(target: Entity, starting_velocity: Vec2) -> Self {
        Self {
            target,
            starting_velocity,
            state: HomingState::Hooking,
            timer: Timer::new(Duration::from_millis(120), TimerMode::Once),
            ticks: 0,
            average_direction: Vec2::default(),
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
                &TotalResolution,
                &Collision<layers::Wall>,
            ),
            With<Player>,
        >,
    >,
    target: Query<(&GlobalTransform, &Collider, Option<&Velocity>), Without<Player>>,
    mut commands: Commands,
    settings: Res<PlayerSettings>,
    time: Res<Time>,
    timescale: Single<&TimeScale>,
    mut hook_collision: EventReader<HookTargetCollision>,
) {
    let timescale = timescale.into_inner();

    let Some((player, mut homing, player_trans, player_collider, mut player_vel, res, collision)) =
        player.map(|p| p.into_inner())
    else {
        return;
    };

    let (vector, target_vel) = match target.get(homing.target) {
        Ok((target_trans, target_collider, target_vel)) => {
            let target = target_trans.compute_transform();
            let abs_target = target_collider.absolute(&target);
            let abs_player = player_collider.global_absolute(player_trans);

            let vector = (abs_target.center() - abs_player.center()).normalize_or_zero();

            homing.average_direction += vector;
            homing.ticks += 1;

            (vector, target_vel)
        }
        Err(_) => {
            error_once!("a homing target is missing one or more components or doesn't exist");
            (homing.average_direction.normalize_or_zero(), None)
        }
    };

    if !collision.entities().is_empty() {
        let contact_normal = res.get().normalize_or_zero();
        let bounce_dot = (contact_normal * -1.0).dot(vector);

        if bounce_dot > settings.break_angle {
            commands.entity(player).remove::<Homing>();
            // TODO: get a nice bounce
            player_vel.0 = contact_normal * 100.;
            return;
        }
    }

    // // we have some velocity damping
    // homing.starting_velocity *= 0.97;
    homing.average_direction += vector;
    homing.ticks += 1;

    match homing.state {
        HomingState::Hooking => {
            player_vel.0 = Vec2::default();
            homing
                .timer
                .tick(Duration::from_secs_f32(time.delta_secs() * timescale.0));

            if homing.timer.just_finished() {
                homing.state = HomingState::Moving;
            }
        }
        HomingState::Moving => {
            player_vel.0 = vector * 700. + target_vel.map(|t| t.0).unwrap_or_default();

            if let Some(_ev) = hook_collision.read().last() {
                homing.state = HomingState::Exploding;
                player_vel.0 = Vec2::default();
            }
        }
        HomingState::Exploding => {
            player_vel.0 = (homing.average_direction / homing.ticks as f32) * 350.;
            commands.entity(player).remove::<Homing>();
        }
    }
}

fn homing_effects(
    mut commands: Commands,
    player: Option<Single<(Entity, &Homing, &GlobalTransform, &Collider), With<Player>>>,
    target: Query<&GlobalTransform, Without<Player>>,
    server: Res<AssetServer>,
    mut prev_state: Local<Option<HomingState>>,
) {
    let Some((entity, homing, transform, collider)) = player.map(|p| p.into_inner()) else {
        return;
    };

    if prev_state.is_none_or(|s| s != homing.state) {
        *prev_state = Some(homing.state);

        match homing.state {
            HomingState::Hooking => {
                // let Ok(target) = target.get(homing.target()) else {
                //     return;
                // };
                //
                // let mut t = Transform::from_translation(
                //     transform.translation() + Vec3::new(TILE_SIZE, -TILE_SIZE, 0.),
                // );
                //
                // t.rotate_around(
                //     collider.global_absolute(transform).center().extend(0.),
                //     Quat::from_rotation_z(
                //         -(target.translation() - transform.translation())
                //             .xy()
                //             .angle_to(Vec2::Y)
                //             + f32::consts::PI,
                //     ),
                // );
                //
                // commands.spawn((
                //     t,
                //     OneShot::Despawn,
                //     ParticleSpawner::default(),
                //     ParticleEffectHandle(server.load("particles/homing_start.ron")),
                // ));
            }
            HomingState::Moving => {}
            HomingState::Exploding => {
                // let Ok(target) = target.get(homing.target()) else {
                //     return;
                // };
                //
                // commands.spawn((
                //     Transform::from_translation(target.translation()).with_rotation(
                //         Quat::from_rotation_z(
                //             (target.translation() - transform.translation())
                //                 .xy()
                //                 .angle_to(Vec2::Y),
                //             // + f32::consts::PI,
                //         ),
                //     ),
                //     ParticleSpawner::default(),
                //     ParticleEffectHandle(server.load("particles/homing_hooking.ron")),
                //     LifeTime::secs(0.5),
                // ));
            }
        }
    }
}

fn homing_hitbox(
    mut commands: Commands,
    mut removed_homing: RemovedComponents<Homing>,
    player: Option<Single<Entity, With<Player>>>,
    added_homing: Option<Single<&Player, Added<Homing>>>,
    hurtbox: Option<Single<(Entity, &Health), With<PlayerHurtBox>>>,
    mut prev_health: Local<Option<Health>>,
) {
    if added_homing.is_some() {
        if let Some((entity, health)) = hurtbox.map(|h| h.into_inner()) {
            *prev_health = Some(*health);
            commands.entity(entity).despawn_recursive();
        }
    } else if let Some(player) = player {
        for entity in removed_homing.read() {
            if *player == entity && hurtbox.is_none() {
                let mut entity = commands.entity(entity);
                if let Some(health) = *prev_health {
                    entity.with_child((PlayerHurtBox, health));
                } else {
                    entity.with_child(PlayerHurtBox);
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone, Component)]
pub struct BrushingMove(Stopwatch);

fn brushing(
    player: Option<
        Single<
            (
                &mut Velocity,
                &mut BrushingMove,
                &Direction,
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
            ),
            (
                With<Player>,
                Without<Grounded>,
                Without<Knockback>,
                Or<(With<BrushingLeft>, With<BrushingRight>)>,
            ),
        >,
    >,
    time: Res<Time>,
    scale: Single<&TimeScale>,
    settings: Res<PlayerSettings>,
) {
    let Some((mut velocity, mut brushing_move, direction, brushing_left, brushing_right)) =
        player.map(|p| p.into_inner())
    else {
        return;
    };

    if (brushing_left.is_some() && *direction == Direction::Right)
        || (brushing_right.is_some() && *direction == Direction::Left)
    {
        brushing_move
            .0
            .tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
    } else {
        brushing_move.0.reset();
    }

    if brushing_move.0.elapsed_secs() >= settings.wall_stick_time {
        brushing_move.0.reset();
    } else {
        // override air_strafe movement
        velocity.0.x = 0.;
    }
}

#[derive(Debug, Component)]
pub struct Jumping;

fn start_jump(
    mut commands: Commands,
    player: Option<
        Single<
            (Entity, &ActionState<Action>, &GlobalTransform),
            (
                Or<(With<Grounded>, With<BrushingLeft>, With<BrushingRight>)>,
                Without<Knockback>,
            ),
        >,
    >,
    server: Res<AssetServer>,
) {
    let Some((entity, action_state, transform)) = player.map(|p| p.into_inner()) else {
        return;
    };

    for action in action_state.get_just_pressed() {
        if action == Action::Jump {
            commands.entity(entity).insert(Jumping);

            commands.spawn((
                Transform::from_translation(
                    transform.translation() + Vec3::new(TILE_SIZE, -TILE_SIZE * 1.75, 0.),
                ),
                OneShot::Despawn,
                ParticleSpawner::default(),
                ParticleEffectHandle(server.load("particles/jump.ron")),
            ));
        }
    }
}

fn jumping(
    mut commands: Commands,
    player: Option<
        Single<
            (Entity, &ActionState<Action>, &mut Velocity),
            (
                With<Player>,
                With<Jumping>,
                Without<Dashing>,
                Without<Homing>,
                Without<Knockback>,
            ),
        >,
    >,
    time: Res<Time>,
    scale: Single<&TimeScale>,
    mut timer: Local<Option<Timer>>,
    settings: Res<PlayerSettings>,
) {
    let Some((entity, action_state, mut velocity)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let timer = timer
        .get_or_insert_with(|| Timer::from_seconds(settings.jump_max_duration, TimerMode::Once));

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
    velocity.0.y = settings.jump_speed;
}

fn wall_jump_impulse(
    player: Option<
        Single<
            (&mut Velocity, Option<&BrushingLeft>, Option<&BrushingRight>),
            (
                With<Player>,
                Added<Jumping>,
                Or<(With<BrushingLeft>, With<BrushingRight>)>,
                Without<Grounded>,
                Without<Knockback>,
            ),
        >,
    >,
    settings: Res<PlayerSettings>,
) {
    let Some((mut velocity, brushing_left, brushing_right)) = player.map(|p| p.into_inner()) else {
        return;
    };

    if brushing_left.is_some() {
        velocity.0.x += settings.wall_impulse;
    } else if brushing_right.is_some() {
        velocity.0.x -= settings.wall_impulse;
    }
}

#[derive(Debug, Default, Component)]
pub struct Dashing(Option<Vec2>);

impl Dashing {
    pub fn new(direction: Option<Vec2>) -> Self {
        Self(direction)
    }
}

fn start_dash(
    mut commands: Commands,
    player: Option<Single<(Entity, &ActionState<Action>), Without<Knockback>>>,
) {
    let Some((entity, action_state)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let axis_pair = action_state.clamped_axis_pair(&Action::Run);
    for action in action_state.get_just_pressed() {
        if action == Action::Dash {
            commands
                .entity(entity)
                .insert(Dashing::new((axis_pair != Vec2::ZERO).then_some(axis_pair)));
        }
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
            (With<Player>, Without<Knockback>),
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
    settings: Res<PlayerSettings>,
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

            let dash_timer = timer.get_or_insert_with(|| {
                Timer::from_seconds(settings.dash_duration, TimerMode::Once)
            });
            dash_timer.tick(Duration::from_secs_f32(time.delta_secs() * scale.0));
            if dash_timer.finished() {
                *dash_reset = false;
                commands.entity(entity).remove::<Dashing>();
                *timer = None;
                velocity.0 /= settings.dash_decay;
                return;
            }

            let dash_vec = dash.0.unwrap_or_else(|| *last_dir);
            velocity.0 = dash_vec.normalize_or_zero() * settings.dash_speed;

            let ghost_timer = spawn_ghost_timer.get_or_insert_with(|| {
                Timer::from_seconds(settings.dash_duration / 5., TimerMode::Repeating)
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
            (
                &mut Velocity,
                &Direction,
                &mut AnimationController<PlayerAnimation>,
            ),
            (
                With<Player>,
                Without<Grounded>,
                Without<Dashing>,
                Without<Homing>,
                Without<Knockback>,
            ),
        >,
    >,
    settings: Res<PlayerSettings>,
) {
    let Some((mut velocity, direction, mut animations)) = player.map(|p| p.into_inner()) else {
        return;
    };

    match direction {
        Direction::Right => {
            if velocity.0.x < settings.walk_speed {
                velocity.0.x = (velocity.0.x + settings.air_accel * settings.walk_speed)
                    .min(settings.walk_speed);
            }
        }
        Direction::Left => {
            if velocity.0.x > -settings.walk_speed {
                velocity.0.x = (velocity.0.x - settings.air_accel * settings.walk_speed)
                    .max(-settings.walk_speed);
            }
        }
        _ => {}
    }

    animations.set_animation_checked(match direction {
        Direction::None => PlayerAnimation::Idle,
        Direction::Right | Direction::Left => PlayerAnimation::Run,
    });
}

fn ground_strafe(
    player: Option<
        Single<
            (
                &mut Velocity,
                &Direction,
                &mut AnimationController<PlayerAnimation>,
            ),
            (
                With<Player>,
                With<Grounded>,
                Without<Dashing>,
                Without<Homing>,
                Without<Knockback>,
            ),
        >,
    >,
    settings: Res<PlayerSettings>,
) {
    let Some((mut velocity, direction, mut animations)) = player.map(|p| p.into_inner()) else {
        return;
    };

    velocity.0.x = direction.unit().x * settings.walk_speed;
    animations.set_animation_checked(match direction {
        Direction::None => PlayerAnimation::Idle,
        Direction::Right | Direction::Left => PlayerAnimation::Run,
    });
}

fn wall_slide(
    player: Option<
        Single<
            &mut Velocity,
            (
                With<Player>,
                Or<(With<BrushingLeft>, With<BrushingRight>)>,
                Without<Knockback>,
            ),
        >,
    >,
    settings: Res<PlayerSettings>,
) {
    let Some(mut velocity) = player else {
        return;
    };

    velocity.0.y = velocity.0.y.max(-settings.slide_speed);
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
                Without<Knockback>,
            ),
        >,
    >,
    settings: Res<PlayerSettings>,
) {
    let Some(mut velocity) = player else {
        return;
    };

    velocity.0.x *= 1.0 - settings.air_damping;
}

fn knockback(
    player: Option<Single<(&mut Velocity, &Knockback), With<Player>>>,
    settings: Res<PlayerSettings>,
    mut set: Local<bool>,
) {
    let Some((mut velocity, knockback)) = player.map(|p| p.into_inner()) else {
        *set = true;
        return;
    };

    if *set {
        *set = false;
        velocity.0.x = knockback.normalized().x * 100.;
    }

    velocity.0.x *= 1.0 - settings.knockback_damping;
}

fn debug(
    player: Option<
        Single<
            (
                Option<&Grounded>,
                Option<&BrushingLeft>,
                Option<&BrushingRight>,
                Option<&Homing>,
                Option<&Jumping>,
                Option<&Dashing>,
                Option<&Knockback>,
            ),
            With<Player>,
        >,
    >,
) {
    if let Some(player) = player {
        println!("{:#?}", player.into_inner());
    }
}
