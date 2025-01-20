use crate::{impl_plugin, spire, TILE_SIZE};
pub use bevy::prelude::*;
use physics::prelude::Collider;
use selector::SelectorTarget;

impl_plugin!(WallHookPlugin, |app: &mut App| {
    app.register_required_components::<spire::WallHook, SelectorTarget>()
        .register_required_components_with::<spire::WallHook, Collider>(|| {
            Collider::from_rect(Vec2::ZERO, Vec2::splat(TILE_SIZE * 2.))
        });
});
