use bevy::{math::NormedVectorSpace, prelude::*};

const TARGET_THRESHOLD: f32 = 128.0;

#[derive(Component)]
pub struct Hook;

#[derive(Component, Default, Debug)]
pub struct HookTarget;

#[derive(Resource, Debug, Default)]
pub struct ViableTargets(Vec<Entity>);

pub(super) fn spawn_hook(server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn((Hook, Sprite::from_image(server.load("hook.png"))));
}

pub(super) fn gather_viable_targets(
    targets: Query<(Entity, &Transform), With<HookTarget>>,
    player: Query<&Transform, With<super::Player>>,
    mut viable: ResMut<ViableTargets>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };

    let mut targets: Vec<_> = targets
        .iter()
        .map(|(e, t)| (e, t, t.translation.distance_squared(player.translation)))
        .filter(|t| t.2 < TARGET_THRESHOLD * TARGET_THRESHOLD)
        .collect();

    targets.sort_unstable_by(|a, b| a.2.total_cmp(&b.2));

    viable.0.clear();
    viable.0.extend(targets.drain(..).map(|t| t.0));
}

pub(super) fn move_hook(
    mut hook: Query<&mut Transform, With<Hook>>,
    targets: Query<&Transform, Without<Hook>>,
    viable: Res<ViableTargets>,
) {
    let Ok(mut hook) = hook.get_single_mut() else {
        return;
    };

    let Some(closest) = viable.0.first() else {
        return;
    };

    let Ok(target) = targets.get(*closest) else {
        return;
    };

    hook.translation = target.translation;
}
