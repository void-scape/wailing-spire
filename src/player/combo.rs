use super::{hook::HookTargetCollision, BrushingLeft, BrushingRight, Grounded, Homing};
use bevy::prelude::*;
use bevy_pixel_gfx::pixel_perfect::HIGH_RES_LAYER;
use bevy_tween::{
    combinator::{sequence, tween},
    prelude::*,
};
use interpolate::angle_z_to;
use std::f32::consts::PI;

#[derive(Debug, Default, Component)]
pub struct Combo(usize);

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct TextAnimation;

/// An entity with this marker (and [`Collider`](physics::collision::Collider)) will increase the player's [`Combo`] score when collided with.
#[derive(Default, Component)]
pub struct ComboCollision;

pub(super) fn combo(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<HookTargetCollision>,
    mut player: Query<(Entity, &mut Combo, Option<&Homing>, Option<&Grounded>)>,
    combo_query: Query<Entity, With<ComboCollision>>,
    text_query: Query<Entity, With<ComboText>>,
    animation_query: Query<Entity, With<TextAnimation>>,
) {
    let Ok((entity, mut combo, homing, grounded)) = player.get_single_mut() else {
        return;
    };

    if combo.0 > 0 && grounded.is_some() && homing.is_none() {
        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/combo.wav")),
            PlaybackSettings::DESPAWN.with_speed(1. + combo.0 as f32),
        ));
        for entity in text_query.iter().chain(animation_query.iter()) {
            commands.entity(entity).despawn();
        }
        combo.0 = 0;
    }

    for _ in reader.read().filter(|c| combo_query.get(c.target).is_ok()) {
        combo.0 += 1;

        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/kill.wav")),
            PlaybackSettings::DESPAWN,
        ));

        for entity in text_query.iter().chain(animation_query.iter()) {
            commands.entity(entity).despawn();
        }

        let rot = -PI * 0.1;
        let dur = 0.05;
        let translation = Vec3::new(0., 0., 1.);
        let text = commands
            .spawn((
                HIGH_RES_LAYER,
                ComboText,
                Text2d::new(format!("{}x", combo.0)),
                TextFont {
                    font_size: 32.,
                    font: server.load("joystix.otf"),
                    ..Default::default()
                },
                Transform::from_translation(translation)
                    .with_scale(Vec3::splat(0.25 + combo.0 as f32 * 0.05)),
            ))
            .id();
        commands.entity(entity).add_child(text);

        let text = text.into_target();
        commands
            .animation()
            .repeat(Repeat::times(3))
            .insert(sequence((
                tween(
                    Duration::from_secs_f32(dur),
                    EaseKind::Linear,
                    text.state(0.).with(angle_z_to(rot)),
                ),
                tween(
                    Duration::from_secs_f32(dur),
                    EaseKind::Linear,
                    text.state(rot).with(angle_z_to(0.)),
                ),
            )))
            .insert(TextAnimation);
    }
}
