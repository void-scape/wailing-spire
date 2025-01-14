use bevy::prelude::*;
use core::marker::PhantomData;

/// A marker component that can be placed on dynamic bodies
/// to enable collisions between the dynamic body and the
/// target static or dynamic bodies.
#[derive(Component, Debug)]
pub struct CollidesWith<T>(PhantomData<T>);

impl<T> Default for CollidesWith<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[derive(Component, Default, Debug)]
pub struct Wall;

#[derive(Component, Default, Debug)]
pub struct Enemy;

#[derive(Component, Default, Debug)]
pub struct Player;

pub trait RegisterCollisionLayer {
    fn register_collision_layer<T: Component>(&mut self) -> &mut Self;
}

impl RegisterCollisionLayer for App {
    fn register_collision_layer<T: Component>(&mut self) -> &mut Self {
        self.add_systems(
            super::Physics,
            (
                super::collision::handle_collisions::<T>,
                super::collision::handle_dynamic_body_collsions::<T>,
                super::collision::update_grounded::<T>,
                super::collision::update_brushing::<T>,
            )
                .chain()
                .in_set(super::PhysicsSystems::Collision),
        )
    }
}
