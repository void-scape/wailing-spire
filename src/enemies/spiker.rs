use crate::player::combo::ComboCollision;
use crate::player::health::HookedDamage;
use crate::spire;
use crate::{animation::AnimationController, TILE_SIZE};
use bevy::prelude::*;
use physics::{layers, prelude::*};
use selector::SelectorTarget;

const SPEED: f32 = 25.;

#[derive(Default, Component)]
#[require(AnimationController<SpikerAnimation>(animation_controller))]
#[require(Velocity, DynamicBody, Collider(collider), layers::TriggersWith<layers::Player>)]
#[require(layers::CollidesWith<layers::Wall>)]
#[require(SelectorTarget, ComboCollision, HookedDamage)]
#[require(super::DespawnHooked)]
#[require(PatrolTarget)]
pub struct Spiker;

#[derive(Default, Component)]
pub enum PatrolTarget {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum SpikerAnimation {
    Idle,
}

fn animation_controller() -> AnimationController<SpikerAnimation> {
    AnimationController::new_with(4., [(SpikerAnimation::Idle, (0, 2))], SpikerAnimation::Idle)
}

fn collider() -> Collider {
    Collider::from_rect(
        Vec2::new(TILE_SIZE / 4., -TILE_SIZE / 2.),
        Vec2::splat(TILE_SIZE / 2.),
    )
}

pub fn update(
    mut spiker_query: Query<(
        &mut Sprite,
        &mut Velocity,
        &spire::Spiker,
        &Transform,
        &mut PatrolTarget,
    )>,
) {
    for (mut sprite, mut vel, spiker, transform, mut spiker_target) in spiker_query.iter_mut() {
        let target = match *spiker_target {
            PatrolTarget::Start => spiker.patrol_start,
            PatrolTarget::End => spiker.patrol_end,
        } * TILE_SIZE;

        if (transform.translation.x - target.x).abs() < TILE_SIZE / 4. {
            *spiker_target = match *spiker_target {
                PatrolTarget::Start => PatrolTarget::End,
                PatrolTarget::End => PatrolTarget::Start,
            };
        } else {
            let sign = (transform.translation.x - target.x).signum();
            vel.0.x = -sign * SPEED;
            sprite.flip_x = sign.is_sign_positive();
        }
    }
}
