use super::prelude::Acceleration;
use bevy::prelude::*;

/// Global definition of the gravity force.
#[derive(Debug, Clone, Copy, Resource)]
pub struct Gravity(pub Vec2);

/// An entity who is not falling.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Grounded;

/// An entity that's brushing a left or right wall.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct BrushingLeft;

/// An entity that's brushing a left or right wall.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct BrushingRight;

pub fn apply_gravity(
    gravity: Res<Gravity>,
    mut object_query: Query<&mut Acceleration, Without<Grounded>>,
) {
    for mut acceleration in object_query.iter_mut() {
        acceleration.apply_force(gravity.0);
    }
}
