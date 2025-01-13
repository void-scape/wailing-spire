use bevy::app::FixedMainScheduleOrder;
use bevy::sprite::Wireframe2dPlugin;
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub mod collision;
pub mod debug;
pub mod gravity;
pub mod spatial;
pub mod trigger;
pub mod velocity;

#[allow(unused)]
pub mod prelude {
    pub use super::collision::*;
    pub use super::gravity::*;
    pub use super::trigger::*;
    pub use super::velocity::*;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, ScheduleLabel)]
pub struct Physics;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, SystemSet)]
pub enum PhysicsSystems {
    Collision,
    Velocity,
}

#[derive(Debug)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(Physics);
        app.world_mut()
            .resource_mut::<FixedMainScheduleOrder>()
            .insert_after(FixedUpdate, Physics);

        app.add_plugins(Wireframe2dPlugin)
            .add_event::<trigger::TriggerEvent>()
            .insert_resource(trigger::TriggerLayerRegistry::default())
            .insert_resource(debug::ShowCollision(false))
            .add_systems(Update, collision::build_tile_set_colliders)
            .add_systems(
                Physics,
                (
                    (gravity::apply_gravity, velocity::apply_velocity)
                        .chain()
                        .in_set(PhysicsSystems::Velocity),
                    (
                        (trigger::register_trigger_layers, trigger::handle_triggers),
                        (
                            spatial::store_static_body_in_spatial_map,
                            collision::handle_collisions,
                            collision::handle_dynamic_body_collsions,
                            collision::update_grounded,
                            collision::update_brushing,
                        )
                            .chain(),
                        debug::debug_display_collider_wireframe,
                        debug::update_show_collision,
                        (
                            debug::debug_show_collision_color,
                            debug::debug_show_trigger_color,
                        )
                            .chain(),
                    )
                        .in_set(PhysicsSystems::Collision),
                ),
            )
            .configure_sets(
                Physics,
                PhysicsSystems::Velocity.before(PhysicsSystems::Collision),
            );
    }
}
