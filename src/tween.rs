use bevy::prelude::*;
use bevy_tween::bevy_time_runner::TimeRunner;

#[derive(Component)]
pub struct DespawnFinished;

pub fn despawn_finished_tweens(
    mut commands: Commands,
    tween_query: Query<(Entity, &TimeRunner), With<DespawnFinished>>,
) {
    for (entity, runner) in tween_query.iter() {
        if runner.is_completed() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
