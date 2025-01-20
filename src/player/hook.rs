use super::movement::Homing;
use super::PlayerSettings;
use super::{health::Dead, Collider, CollidesWith, Player, Velocity};
use crate::TILE_SIZE;
use bevy::prelude::*;
use physics::spatial::SpatialHash;
use physics::trigger::TriggerEnter;
use selector::SelectorTarget;

#[derive(Debug, Resource)]
pub(super) struct ShowHook(Visibility);

impl Default for ShowHook {
    fn default() -> Self {
        Self(Visibility::Hidden)
    }
}

impl ShowHook {
    pub fn show(&mut self) {
        self.0 = Visibility::Visible;
    }

    pub fn hide(&mut self) {
        self.0 = Visibility::Hidden;
    }
}

pub(super) fn show_hook(
    mut show: ResMut<ShowHook>,
    player: Query<Entity, (With<Player>, Without<Dead>)>,
    viable: Res<ViableTargets>,
) {
    if !player.is_empty() && !viable.0.is_empty() {
        show.show();
    } else {
        show.hide();
    }
}

#[derive(Component)]
pub struct Hook {
    chains: Vec<Entity>,
}

#[derive(Component)]
pub struct Chain;

/// When combined with a [`SpatialHash`], performs ray
/// casting on collision to occlude viable hook targets.
#[derive(Default, Component)]
pub struct OccludeHookTarget;

#[derive(Resource, Debug, Default)]
pub struct ViableTargets(pub(super) Vec<ViableTarget>);

#[derive(Debug)]
pub(super) struct ViableTarget {
    pub(super) entity: Entity,
    pub(super) translation: Vec2,
}

/// Player is moving fast enough to _kill_ enemies.
// #[derive(Component)]
// pub struct TerminalVelocity;

pub(super) fn spawn_hook(server: Res<AssetServer>, mut commands: Commands) {
    let mut chains = Vec::new();
    for _ in 0..8 {
        let chain = commands.spawn((
            Chain,
            Sprite::from_image(server.load("sprites/chain.png")),
            Transform::default().with_translation(Vec3::new(0., 0., 100.)),
        ));

        chains.push(chain.id());
    }

    commands.spawn((
        Hook { chains },
        Sprite::from_image(server.load("sprites/hook.png")),
        Transform::default().with_translation(Vec3::new(0., 0., 100.)),
    ));
}

pub(super) fn gather_viable_targets(
    targets: Query<(Entity, &GlobalTransform), With<SelectorTarget>>,
    player: Query<&GlobalTransform, With<super::Player>>,
    mut viable: ResMut<ViableTargets>,
    spatial_hash_query: Query<&SpatialHash, With<OccludeHookTarget>>,
    settings: Res<PlayerSettings>,
) {
    viable.0.clear();

    let Ok(player) = player.get_single() else {
        return;
    };

    let mut targets: Vec<_> = targets
        .iter()
        .map(|(e, t)| {
            (
                e,
                t,
                t.compute_transform()
                    .translation
                    .distance_squared(player.translation()),
            )
        })
        .filter(|t| t.2 < settings.target_threshold * settings.target_threshold)
        .filter(|t| {
            spatial_hash_query.iter().all(|hash| {
                let pxy = player.translation().xy();
                let txy = t.1.translation().xy();

                let dist = pxy - txy;
                hash.ray_cast(txy, pxy, (dist.length() / TILE_SIZE) as usize)
            })
        })
        .collect();

    targets.sort_unstable_by(|a, b| a.2.total_cmp(&b.2));

    viable.0.extend(targets.drain(..).map(|t| ViableTarget {
        entity: t.0,
        translation: t.1.translation().xy(),
    }));
}

pub(super) fn move_hook(
    mut hook: Query<(&mut Visibility, &mut Transform, &Hook), (Without<Chain>, Without<Player>)>,
    mut chains: Query<&mut Transform, (With<Chain>, Without<Player>)>,
    collider_targets: Query<(Entity, &GlobalTransform, &Collider), Without<Hook>>,
    player: Query<
        (
            &GlobalTransform,
            &Collider,
            &Velocity,
            Option<&super::Homing>,
        ),
        With<Player>,
    >,
    mut vis_query: Query<&mut Visibility, Without<Hook>>,
    show_hook: Res<ShowHook>,
    mut local_vis: Local<Visibility>,
) {
    let Ok((mut hook_visibility, mut hook_transform, hook)) = hook.get_single_mut() else {
        return;
    };

    let Ok((player, player_collider, player_velocity, homing)) = player.get_single() else {
        return;
    };

    if *local_vis != show_hook.0 {
        *local_vis = show_hook.0;
        *hook_visibility = show_hook.0;
        for entity in hook.chains.iter() {
            if let Ok(mut vis) = vis_query.get_mut(*entity) {
                *vis = show_hook.0;
            }
        }
    }

    // let axis_pair = action.clamped_axis_pair(&Action::Aim);
    // let target = if axis_pair != Vec2::ZERO {
    //     let mut viable_heuristic = viable
    //         .0
    //         .iter()
    //         .map(|t| {
    //             (
    //                 t.entity,
    //                 (t.translation - player.translation().xy())
    //                     .normalize_or_zero()
    //                     .dot(Vec2::new(axis_pair.x, axis_pair.y).normalize_or_zero()),
    //             )
    //         })
    //         .filter(|(_, dot)| dot.is_sign_positive())
    //         .collect::<Vec<_>>();
    //
    //     if viable_heuristic.is_empty() {
    //         viable.0.first().map(|t| t.entity)
    //     } else {
    //         viable_heuristic.sort_unstable_by(|(_, dot_a), (_, dot_b)| {
    //             if (dot_a.abs() - dot_b.abs()).abs() < 0.3 {
    //                 Ordering::Equal
    //             } else {
    //                 dot_a.total_cmp(dot_b)
    //             }
    //         });
    //         viable_heuristic.first().map(|(entity, _)| *entity)
    //     }
    // } else {
    //     viable.0.first().map(|t| t.entity)
    // };

    let mut moved = false;
    if let Some(targ_selection) = homing.map(|h| h.target()) {
        if let Ok((_, target, target_collider)) = collider_targets.get(targ_selection) {
            moved = true;
            let target = target.compute_transform();
            let abs_target = target_collider.absolute(&target);

            hook_transform.translation.x = abs_target.center().x;
            hook_transform.translation.y = abs_target.center().y;

            let abs_player = player_collider.global_absolute(player);

            let vector = abs_target.center() - abs_player.center();
            let segments = vector / hook.chains.len() as f32;

            for (i, chain) in hook.chains.iter().enumerate() {
                let Ok(mut chain) = chains.get_mut(*chain) else {
                    continue;
                };

                chain.translation = (abs_player.center() + segments * i as f32).extend(10.);
            }
        }
    }

    //if action.just_pressed(&Action::Interact) && homing.is_none() {
    //    commands.spawn((
    //        AudioPlayer::new(server.load("audio/sfx/hook.wav")),
    //        PlaybackSettings::DESPAWN,
    //    ));
    //    commands
    //        .entity(player_entity)
    //        .insert(super::Homing::new(targ_entity, player_velocity.0));
    //}

    // move offscreen?? idk
    if !moved {
        for chain in hook.chains.iter() {
            let Ok(mut chain) = chains.get_mut(*chain) else {
                continue;
            };

            chain.translation.x = -1000.0;
            hook_transform.translation.x = -1000.0;
        }
    }
}

// pub(super) fn terminal_velocity(
//     mut commands: Commands,
//     player: Option<Single<(Entity, &Velocity), With<Player>>>,
//     server: Res<AssetServer>,
//     mut shielded: Local<Option<Entity>>,
// ) {
//     if let Some((entity, vel)) = player.map(|p| p.into_inner()) {
//         if vel.0.length_squared() >= TERMINAL_VELOCITY2_THRESHOLD {
//             if shielded.is_none() {
//                 let shield = commands
//                     .spawn(Sprite {
//                         image: server.load("sprites/shield.png"),
//                         anchor: Anchor::TopLeft,
//                         ..Default::default()
//                     })
//                     .id();
//                 commands
//                     .entity(entity)
//                     .insert(TerminalVelocity)
//                     .add_child(shield);
//                 *shielded = Some(shield);
//             }
//         } else if let Some(shield) = *shielded {
//             commands.entity(entity).remove::<TerminalVelocity>();
//             if let Some(mut entity) = commands.get_entity(shield) {
//                 entity.despawn();
//             }
//             *shielded = None;
//         }
//     }
// }

#[derive(Debug, Clone, Copy, Event)]
pub struct HookTargetCollision {
    pub target: Entity,
    pub kill_target: bool,
}

pub(super) fn collision_hook(
    mut commands: Commands,
    player: Query<(Entity, &Homing), (With<Player>, Without<Dead>)>,
    mut reader: EventReader<TriggerEnter>,
    mut writer: EventWriter<HookTargetCollision>,
) {
    let Ok((entity, homing)) = player.get_single() else {
        let _ = reader.clear();
        return;
    };

    for event in reader.read() {
        if event.trigger == entity {
            if homing.target() == event.target {
                commands.entity(entity).remove::<super::Homing>();
            }

            writer.send(HookTargetCollision {
                target: event.target,
                kill_target: true,
            });
        }
    }
}
