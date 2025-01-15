use super::{hook::HookKill, BrushingLeft, BrushingRight, Grounded, Homing};
use bevy::prelude::*;
use bevy_pixel_gfx::pixel_perfect::HIGH_RES_LAYER;
use bevy_tween::{
    combinator::{sequence, tween},
    prelude::*,
};
use interpolate::angle_z_to;
use std::f32::consts::PI;

const COMBO_PITCH_FACTOR: f32 = 0.;

#[derive(Debug, Default, Component)]
pub struct Combo(usize);

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct TextAnimation;

pub(super) fn combo(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<HookKill>,
    mut player: Query<(
        Entity,
        &mut Combo,
        Option<&Homing>,
        Option<&Grounded>,
        Option<&BrushingLeft>,
        Option<&BrushingRight>,
    )>,
    text_query: Query<Entity, With<ComboText>>,
    animation_query: Query<Entity, With<TextAnimation>>,
) {
    let Ok((entity, mut combo, homing, grounded, _brushing_left, _brushing_right)) =
        player.get_single_mut()
    else {
        return;
    };

    if combo.0 > 0 && grounded.is_some() && homing.is_none()
    // || brushing_left.is_some() || brushing_right.is_some()
    {
        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/combo.wav")),
            PlaybackSettings::DESPAWN.with_speed(1. + combo.0 as f32 * COMBO_PITCH_FACTOR),
        ));
        for entity in text_query.iter().chain(animation_query.iter()) {
            commands.entity(entity).despawn();
        }
        combo.0 = 0;
    }

    for _ in reader.read() {
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
                // tween(
                //     Duration::from_secs_f32(1.),
                //     EaseKind::Linear,
                //     text.state(transform.translation).with(translation_to(
                //         transform.translation + Vec3::new(0., -10., 0.),
                //     )),
                // ),
            )))
            .insert(TextAnimation);
    }
}
