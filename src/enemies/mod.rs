use crate::{animation::AnimationPlugin, spire};
use bevy::prelude::*;

pub mod dino;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AnimationPlugin::<dino::DinoAnimation>::default())
            .register_required_components::<spire::Dino, dino::Dino>()
            .add_systems(Update, (dino::flip_dino_left, dino::flip_dino_right));
    }
}
