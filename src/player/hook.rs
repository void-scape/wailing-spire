use super::{Action, Collider, CollidesWith, Player, Velocity};
use bevy::{math::NormedVectorSpace, prelude::*};
use leafwing_input_manager::prelude::*;

const TARGET_THRESHOLD: f32 = 1024.0;

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
    mut hook: Query<(&mut Transform, &Hook), (Without<Chain>, Without<Player>)>,
    mut chains: Query<&mut Transform, (With<Chain>, Without<Player>)>,
    targets: Query<(Entity, &GlobalTransform, &Collider), Without<Hook>>,
    mut player: Query<(&ActionState<Action>, &Transform, &Collider, &mut Velocity), With<Player>>,
    viable: Res<ViableTargets>,
    mut commands: Commands,
) {
    let Ok((mut hook_transform, hook)) = hook.get_single_mut() else {
        return;
    };
    let Some(closest) = viable.0.first() else {
        return;
    };
    let Ok((targ_entity, target, target_collider)) = targets.get(*closest) else {
        return;
    };
    let Ok((action, player, player_collider, mut player_velocity)) = player.get_single_mut() else {
        return;
    };

    let target = target.compute_transform();
    let abs_target = target_collider.absolute(&target);

    hook_transform.translation.x = abs_target.center().x;
    hook_transform.translation.y = abs_target.center().y;

    let abs_player = player_collider.absolute(player);

    let vector = abs_target.center() - abs_player.center();
    let segments = vector / hook.chains.len() as f32;

    for (i, chain) in hook.chains.iter().enumerate() {
        let Ok(mut chain) = chains.get_mut(*chain) else {
            continue;
        };

        chain.translation = (abs_player.center() + segments * i as f32).extend(10.);
    }

    let unit = vector.normalize_or_zero() * 30.;

    if action.pressed(&Action::Interact) {
        player_velocity.0 += unit;

        if abs_player.collides_with(&abs_target) {
            commands.entity(targ_entity).despawn_recursive();
        }
    }
}
