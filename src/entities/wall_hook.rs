use crate::{impl_plugin, physics::prelude::Collider, player::HookTarget, spire, TILE_SIZE};
pub use bevy::prelude::*;

impl_plugin!(WallHookPlugin, |app: &mut App| {
    app.register_required_components::<spire::WallHook, HookTarget>()
        .register_required_components_with::<spire::WallHook, Collider>(|| {
            Collider::from_rect(Vec2::ZERO, Vec2::splat(TILE_SIZE * 2.))
        });
});
