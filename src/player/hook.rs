use super::{
    health::{Dead, Health},
    Action, Collider, CollidesWith, Grounded, Player, Velocity,
};
use bevy::{prelude::*, sprite::Anchor};
use leafwing_input_manager::prelude::*;

const TARGET_THRESHOLD: f32 = 1024.0;
const REEL_SPEED: f32 = 30.;
const TERMINAL_VELOCITY2_THRESHOLD: f32 = 60_000.;
const COMBO_PITCH_FACTOR: f32 = 0.2;

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
) {
    if !player.is_empty() {
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

#[derive(Component, Default, Debug)]
pub struct HookTarget;

#[derive(Resource, Debug, Default)]
pub struct ViableTargets(Vec<Entity>);

// #[derive(Component)]
// pub struct SelectedTarget(Entity);

/// Player is moving fast enough to _kill_ enemies.
#[derive(Component)]
pub struct TerminalVelocity;

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
    targets: Query<(Entity, &GlobalTransform), With<HookTarget>>,
    player: Query<&GlobalTransform, With<super::Player>>,
    mut viable: ResMut<ViableTargets>,
) {
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
                    .distance_squared(player.compute_transform().translation),
            )
        })
        .filter(|t| t.2 < TARGET_THRESHOLD * TARGET_THRESHOLD)
        .collect();

    targets.sort_unstable_by(|a, b| a.2.total_cmp(&b.2));

    viable.0.clear();
    viable.0.extend(targets.drain(..).map(|t| t.0));
}

pub(super) fn move_hook(
    mut commands: Commands,
    mut hook: Query<(&mut Visibility, &mut Transform, &Hook), (Without<Chain>, Without<Player>)>,
    mut chains: Query<&mut Transform, (With<Chain>, Without<Player>)>,
    targets: Query<(Entity, &GlobalTransform, &Collider), Without<Hook>>,
    mut player: Query<
        (
            Entity,
            &ActionState<Action>,
            &GlobalTransform,
            &Collider,
            &mut Velocity,
            Option<&super::Homing>,
        ),
        With<Player>,
    >,
    mut vis_query: Query<&mut Visibility, Without<Hook>>,
    viable: Res<ViableTargets>,
    show_hook: Res<ShowHook>,
    mut local_vis: Local<Visibility>,
) {
    let Ok((mut hook_visibility, mut hook_transform, hook)) = hook.get_single_mut() else {
        return;
    };
    let Some(closest) = viable.0.first() else {
        return;
    };
    let Ok((targ_entity, target, target_collider)) = targets.get(*closest) else {
        *hook_visibility = Visibility::Hidden;
        return;
    };
    let Ok((player_entity, action, player, player_collider, mut player_velocity, homing)) =
        player.get_single_mut()
    else {
        *hook_visibility = Visibility::Hidden;
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

    if action.just_pressed(&Action::Interact) && homing.is_none() {
        commands.entity(player_entity).insert(super::Homing {
            target: targ_entity,
            starting_velocity: player_velocity.0,
        });
    }

    // if action.pressed(&Action::Interact) {
    //     commands
    //         .entity(player_entity)
    //         .insert(SelectedTarget(targ_entity));
    // } else {
    //     commands.entity(player_entity).remove::<SelectedTarget>();
    // }
}

pub(super) fn terminal_velocity(
    mut commands: Commands,
    player: Option<Single<(Entity, &Velocity), With<Player>>>,
    server: Res<AssetServer>,
    mut shielded: Local<Option<Entity>>,
) {
    if let Some((entity, vel)) = player.map(|p| p.into_inner()) {
        if vel.0.length_squared() >= TERMINAL_VELOCITY2_THRESHOLD {
            if shielded.is_none() {
                let shield = commands
                    .spawn(Sprite {
                        image: server.load("sprites/shield.png"),
                        anchor: Anchor::TopLeft,
                        ..Default::default()
                    })
                    .id();
                commands
                    .entity(entity)
                    .insert(TerminalVelocity)
                    .add_child(shield);
                *shielded = Some(shield);
            }
        } else if let Some(shield) = *shielded {
            commands.entity(entity).remove::<TerminalVelocity>();
            commands.entity(shield).despawn();
            *shielded = None;
        }
    }
}

#[derive(Debug, Event)]
pub struct HookKill(Entity);

pub(super) fn collision_hook(
    mut commands: Commands,
    targets: Query<(Entity, &GlobalTransform, &Collider)>,
    mut player: Query<
        (
            Entity,
            &GlobalTransform,
            &Collider,
            &super::Homing,
            Option<&TerminalVelocity>,
            &mut Health,
        ),
        With<Player>,
    >,
    mut writer: EventWriter<HookKill>,
) {
    let Ok((
        player_entity,
        player,
        player_collider,
        selected_target,
        terminal_velocity,
        mut health,
    )) = player.get_single_mut()
    else {
        return;
    };

    let Ok((targ_entity, target, target_collider)) = targets.get(selected_target.target) else {
        return;
    };

    let abs_target = target_collider.global_absolute(target);
    let abs_player = player_collider.global_absolute(player);

    // defer despawn until post_update
    if abs_player.expand(2.).collides_with(&abs_target) {
        if terminal_velocity.is_some() {
            commands.entity(player_entity).remove::<super::Homing>();
            writer.send(HookKill(targ_entity));
        } else {
            // TODO: trigger collision for health + trigger must leave before you get hit again +
            // kickback
            health.damage(1);
            println!("Ouch! [{}/{}]", health.current(), health.max());
        }
    }
}

pub(super) fn despawn_hook_kills(mut commands: Commands, mut reader: EventReader<HookKill>) {
    for kill in reader.read() {
        commands.entity(kill.0).despawn_recursive();
    }
}

#[derive(Debug, Default, Component)]
pub struct Combo(usize);

pub(super) fn combo(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<HookKill>,
    mut player: Query<(&mut Combo, Option<&Grounded>)>,
) {
    let Ok((mut combo, grounded)) = player.get_single_mut() else {
        return;
    };

    if grounded.is_some() {
        combo.0 = 0;
    }

    for _ in reader.read() {
        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/combo.wav")),
            PlaybackSettings::DESPAWN.with_speed(1. + combo.0 as f32 * COMBO_PITCH_FACTOR),
        ));

        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/kill.wav")),
            PlaybackSettings::DESPAWN,
        ));

        combo.0 += 1;
    }
}
