use super::{movement::Homing, Acceleration, Action, Player, PlayerAnimation, TriggerEnter};
use crate::{animation::AnimationController, tween::DespawnFinished};
use bevy::{
    input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
};
use bevy_pixel_gfx::{
    glitch::{self, GlitchIntensity},
    screen_shake,
};
use bevy_tween::{
    combinator::{sequence, tween},
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};
use leafwing_input_manager::prelude::ActionState;
use physics::TimeScale;
use std::time::Duration;

/// Deal damage to player if hooked and collided with.
#[derive(Default, Component)]
pub struct HookedDamage;

#[derive(Component)]
pub struct Health {
    current: usize,
    max: usize,
    dead: bool,
}

impl Health {
    pub const PLAYER: Self = Health::full(3);

    pub const fn full(max: usize) -> Self {
        Self {
            current: max,
            dead: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: usize) {
        self.current = (self.current + heal).max(self.max);
    }

    pub fn damage(&mut self, damage: usize) {
        self.current = self.current.saturating_sub(damage);
        self.dead = self.current == 0;
    }

    pub fn damage_all(&mut self) {
        self.current = 0;
        self.dead = true;
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn dead(&mut self) -> bool {
        if self.current == 0 {
            self.dead = true;
        }

        self.dead
    }
}

#[derive(Component)]
pub struct Dead;

pub(super) fn death(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut player: Query<
        (
            Entity,
            &mut Health,
            &mut ActionState<Action>,
            &mut AnimationController<PlayerAnimation>,
        ),
        (With<Player>, Without<Dead>),
    >,
) {
    let Ok((entity, mut health, mut action_state, mut animations)) = player.get_single_mut() else {
        return;
    };

    if health.dead() {
        error!("You Died!");

        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/death.wav")),
            PlaybackSettings::DESPAWN,
        ));

        animations.set_animation_one_shot(PlayerAnimation::Death);

        action_state.disable_all_actions();
        commands.entity(entity).insert(Dead);
    }
}

pub(super) fn no_shield_collision(
    mut commands: Commands,
    mut player: Query<
        (
            Entity,
            &GlobalTransform,
            &mut Health,
            &mut AnimationController<PlayerAnimation>,
            &mut Acceleration,
        ),
        (With<Player>, Without<Homing>),
    >,
    glitch_intensity: Single<Entity, With<GlitchIntensity>>,
    transform_query: Query<&GlobalTransform>,
    mut enter: EventReader<TriggerEnter>,
    time_scale: Single<Entity, With<TimeScale>>,
    mut screen_shake: ResMut<screen_shake::ScreenShake>,
    gamepads: Query<Entity, With<Gamepad>>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
) {
    let Ok((entity, transform, mut health, mut animations, mut acceleration)) =
        player.get_single_mut()
    else {
        enter.clear();
        return;
    };

    for event in enter.read() {
        if event.trigger == entity {
            health.damage(1);
            animations.set_animation_one_shot(PlayerAnimation::Hit);
            error!("Ouch! [{}/{}]", health.current(), health.max());

            let scale = time_scale.into_target();
            commands
                .animation()
                .insert(sequence((
                    tween(
                        Duration::from_secs_f32(0.1),
                        EaseKind::Linear,
                        scale.with(physics::time_scale(1., 0.2)),
                    ),
                    tween(
                        Duration::from_secs_f32(0.3),
                        EaseKind::Linear,
                        scale.with(physics::time_scale(0.2, 1.)),
                    ),
                )))
                .insert(DespawnFinished);

            screen_shake
                .max_offset(125.)
                .camera_decay(0.9)
                .trauma_decay(1.2)
                .shake();

            for entity in &gamepads {
                rumble_requests.send(GamepadRumbleRequest::Add {
                    duration: Duration::from_secs_f32(0.3),
                    intensity: GamepadRumbleIntensity::weak_motor(0.5),
                    gamepad: entity,
                });
            }

            let glitch = glitch_intensity.into_target();
            commands
                .animation()
                .insert(sequence((
                    tween(
                        Duration::from_secs_f32(0.05),
                        EaseKind::Linear,
                        glitch.with(glitch::glitch_intensity(0., 0.5)),
                    ),
                    tween(
                        Duration::from_secs_f32(0.2),
                        EaseKind::Linear,
                        glitch.with(glitch::glitch_intensity(0.5, 0.)),
                    ),
                )))
                .insert(DespawnFinished);

            let Ok(target_t) = transform_query.get(event.target) else {
                continue;
            };

            let diff = (transform.translation() - target_t.translation())
                .xy()
                .normalize_or_zero();
            // acceleration.apply_force(diff * 500.);
        }
    }
}
