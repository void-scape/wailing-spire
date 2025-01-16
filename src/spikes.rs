use crate::{
    physics::{
        layers::RegisterCollisionLayer,
        prelude::{Collider, Collision},
        spatial::{SpatialData, SpatialHash, StaticBodyData},
    },
    player::health::{Dead, Health},
    spire, TILE_SIZE,
};
use bevy::prelude::*;

pub struct SpikePlugin;

impl Plugin for SpikePlugin {
    fn build(&self, app: &mut App) {
        app.register_collision_layer::<Spike>()
            .add_systems(Update, kill_player)
            .add_systems(Last, build_spikes);
    }
}

#[derive(Component)]
pub struct Spike;

fn build_spikes(
    mut commands: Commands,
    spike_query: Query<(Entity, &GlobalTransform), Added<spire::TileSpike>>,
) {
    let mut map: SpatialHash<StaticBodyData> = SpatialHash::new(TILE_SIZE * 2.);

    let height = 4.;
    for (_, transform) in spike_query.iter() {
        let collider = Collider::from_rect(
            Vec2::new(0., -TILE_SIZE + height),
            Vec2::new(TILE_SIZE, height),
        );

        let entity = commands
            .spawn((transform.compute_transform(), collider, Spike))
            .id();
        map.insert(SpatialData {
            collider: collider.global_absolute(transform),
            data: None,
            entity,
        })
    }

    if !map.is_empty() {
        // this weirdness is because the parent of these tiles already has a spatial map and we want this
        // to despawn when the level reloads so we just make it a child of something.
        commands
            .entity(spike_query.iter().next().map(|(entity, _)| entity).unwrap())
            .with_child((map, Spike));
    }
}

fn kill_player(
    mut player: Query<(&mut Health, &Collision), (With<crate::player::Player>, Without<Dead>)>,
    spike_query: Query<&Spike>,
) {
    let Ok((mut health, collision)) = player.get_single_mut() else {
        return;
    };

    if collision
        .entities()
        .iter()
        .any(|e| spike_query.get(*e).is_ok())
    {
        health.damage_all();
    }
}
