use crate::spire;
use bevy::prelude::*;
use bevy_pixel_gfx::anchor::DynamicCameraAnchor;

pub struct EntityRegistryPlugin;

impl Plugin for EntityRegistryPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components_with::<spire::DynamicCameraAnchor, DynamicCameraAnchor>(
            || DynamicCameraAnchor::new(128., 1000.),
        )
        .add_systems(Update, dyn_camera_anchor);
    }
}

fn dyn_camera_anchor(
    mut commands: Commands,
    ldtk: Query<(Entity, &spire::DynamicCameraAnchor), Added<spire::DynamicCameraAnchor>>,
) {
    for (entity, anchor) in ldtk.iter() {
        commands.entity(entity).with_child((
            Transform::default(),
            DynamicCameraAnchor::new(anchor.radius, anchor.speed),
        ));
    }
}
