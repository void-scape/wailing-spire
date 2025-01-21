use super::{
    movement::Homing, Acceleration, Action, Player, PlayerAnimation, PlayerHurtBox, PlayerSettings,
    TriggerEnter,
};
use crate::{
    animation::AnimationController,
    health::{Dead, Health, TriggeredHitBoxes},
    tween::DespawnFinished,
};
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
use physics::{prelude::Collider, time_scale, TimeScale};
use std::time::Duration;

pub(super) fn death(
    mut commands: Commands,
    server: Res<AssetServer>,
    player: Option<
        Single<
            (
                &mut ActionState<Action>,
                &mut AnimationController<PlayerAnimation>,
            ),
            With<Player>,
        >,
    >,
    player_hurtbox: Option<Single<Entity, (With<PlayerHurtBox>, With<Dead>)>>,
) {
    if player_hurtbox.is_none() {
        return;
    }

    if let Some((mut action_state, mut animations)) = player.map(|p| p.into_inner()) {
        animations.set_animation_one_shot(PlayerAnimation::Death);
        action_state.disable_all_actions();
    }

    error!("You Died!");
    commands.spawn((
        AudioPlayer::new(server.load("audio/sfx/death.wav")),
        PlaybackSettings::DESPAWN,
    ));
}

#[derive(Debug, Component)]
pub struct Knockback(Vec2);

impl Knockback {
    pub fn normalized(&self) -> Vec2 {
        self.0
    }
}

pub(super) fn insert_knockback(
    mut commands: Commands,
    player: Option<Single<Entity, With<Player>>>,
    player_hurtbox: Option<
        Single<
            (&TriggeredHitBoxes, &GlobalTransform, &Collider),
            (With<PlayerHurtBox>, Without<Dead>),
        >,
    >,
    transform_query: Query<(&GlobalTransform, &Collider)>,
) {
    let Some(entity) = player else {
        return;
    };

    let Some((triggered_hitboxes, transform, collider)) = player_hurtbox.map(|p| p.into_inner())
    else {
        return;
    };

    let mut diff = Vec2::ZERO;
    for hitbox_entity in triggered_hitboxes.entities() {
        let Ok((target_t, target_c)) = transform_query.get(*hitbox_entity) else {
            continue;
        };

        diff += (collider.global_absolute(transform).center()
            - target_c.global_absolute(target_t).center())
        .normalize_or_zero();
    }

    if diff != Vec2::ZERO {
        commands.entity(*entity).insert(Knockback(diff));
    }
}

pub(super) fn update_knockback(
    mut commands: Commands,
    player: Option<
        Single<
            (Entity, &mut AnimationController<PlayerAnimation>),
            (With<Player>, With<Knockback>),
        >,
    >,
    glitch_intensity: Single<Entity, With<GlitchIntensity>>,
    mut screen_shake: ResMut<screen_shake::ScreenShake>,
    gamepads: Query<Entity, With<Gamepad>>,
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
    settings: Res<PlayerSettings>,
    time: Res<Time>,
    time_scale_entity: Single<Entity, With<TimeScale>>,
    time_scale: Single<&TimeScale>,
    mut timer: Local<Timer>,
    mut new_hit: Local<bool>,
) {
    let Some((entity, mut animations)) = player.map(|p| p.into_inner()) else {
        *new_hit = true;
        return;
    };

    if *new_hit {
        *new_hit = false;
        *timer = Timer::new(
            Duration::from_secs_f32(settings.knockback_duration),
            TimerMode::Once,
        );

        animations.set_animation_one_shot(PlayerAnimation::Hit);

        let scale = time_scale_entity.into_target();
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
    }

    timer.tick(Duration::from_secs_f32(time.delta_secs() * time_scale.0));
    if timer.finished() {
        commands.entity(entity).remove::<Knockback>();
    }
}
