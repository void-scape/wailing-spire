use bevy::prelude::*;
use std::time::Duration;

/// Despawns an entity after `Timer` is exhausted.
#[derive(Component)]
pub struct LifeTime(Timer);

impl LifeTime {
    pub fn secs(secs: f32) -> Self {
        Self(Timer::new(Duration::from_secs_f32(secs), TimerMode::Once))
    }
}

pub struct LifeTimePlugin;

impl Plugin for LifeTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_lifetime);
    }
}

fn update_lifetime(
    mut commands: Commands,
    mut lifetime_query: Query<(Entity, &mut LifeTime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in lifetime_query.iter_mut() {
        lifetime.0.tick(time.delta());
        if lifetime.0.just_finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
