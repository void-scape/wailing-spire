use crate::player::hook::HookTargetCollision;
use crate::{animation::AnimationPlugin, spire};
use bevy::prelude::*;
use bevy_pixel_gfx::screen_shake;

pub mod dino;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationPlugin::<dino::DinoAnimation>::default())
            .register_required_components::<spire::Dino, dino::Dino>()
            .add_systems(Update, (dino::flip_dino, hook_collisions));
    }
}

#[derive(Default, Component)]
pub struct DespawnHooked;

fn hook_collisions(
    mut commands: Commands,
    mut reader: EventReader<HookTargetCollision>,
    mut screen_shake: ResMut<screen_shake::ScreenShake>,
    despawn_query: Query<&DespawnHooked>,
) {
    for collision in reader.read().filter(|c| c.shield_up()) {
        if despawn_query.get(collision.entity()).is_ok() {
            commands.entity(collision.entity()).despawn_recursive();
            screen_shake
                .max_offset(75.)
                .camera_decay(0.9)
                .trauma_decay(1.2)
                .shake();
        }
    }
}
