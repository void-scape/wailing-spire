use crate::{animation::AnimationController, physics::prelude::*, TILE_SIZE};
use bevy::prelude::*;
use rand::Rng;

const SPEED: f32 = 100.;

#[derive(Default, Component)]
#[require(AnimationController<DinoAnimation>(animation_controller))]
#[require(Velocity(velocity), DynamicBody, Collider(collider))]
#[require(layers::CollidesWith<layers::Wall>)]
#[require(crate::player::HookTarget)]
pub struct Dino;

impl Dino {
    const RIGHT: Vec2 = Vec2::new(SPEED, 0.);
    const LEFT: Vec2 = Vec2::new(-SPEED, 0.);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum DinoAnimation {
    Idle,
}

fn velocity() -> Velocity {
    Velocity(if rand::thread_rng().gen::<bool>() {
        Dino::RIGHT
    } else {
        Dino::LEFT
    })
}

fn animation_controller() -> AnimationController<DinoAnimation> {
    AnimationController::new_with(12., [(DinoAnimation::Idle, (0, 13))], DinoAnimation::Idle)
}

fn collider() -> Collider {
    Collider::from_rect(
        Vec2::new(TILE_SIZE * 0.5, -TILE_SIZE * 0.8),
        Vec2::splat(TILE_SIZE / 2.),
    )
}

pub fn flip_dino(
    mut dino_query: Query<(&mut Sprite, &mut Velocity), (With<Dino>, With<Collision>)>,
) {
    for (mut sprite, mut vel) in dino_query.iter_mut() {
        match vel.0 {
            Dino::LEFT => {
                *vel = Velocity(Dino::RIGHT);
                sprite.flip_x = false;
            }
            Dino::RIGHT => {
                *vel = Velocity(Dino::LEFT);
                sprite.flip_x = true;
            }
            _ => unreachable!(),
        }
    }
}
