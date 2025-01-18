use super::{hook::HookTargetCollision, Action, Collision, Player, PlayerAnimation};
use crate::animation::AnimationController;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

/// Deal damage to player if hooked and collided with.
#[derive(Default, Component)]
pub struct HookedDamage;

#[derive(Component)]
pub struct Health {
    current: usize,
    max: usize,
    dead: bool,
}

impl Health {
    pub const PLAYER: Self = Health::full(3);

    pub const fn full(max: usize) -> Self {
        Self {
            current: max,
            dead: false,
            max,
        }
    }

    pub fn heal(&mut self, heal: usize) {
        self.current = (self.current + heal).max(self.max);
    }

    pub fn damage(&mut self, damage: usize) {
        self.current = self.current.saturating_sub(damage);
        self.dead = self.current == 0;
    }

    pub fn damage_all(&mut self) {
        self.current = 0;
        self.dead = true;
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn max(&self) -> usize {
        self.max
    }

    pub fn dead(&mut self) -> bool {
        if self.current == 0 {
            self.dead = true;
        }

        self.dead
    }
}

#[derive(Component)]
pub struct Dead;

pub(super) fn death(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut player: Query<
        (
            Entity,
            &mut Health,
            &mut ActionState<Action>,
            &mut AnimationController<PlayerAnimation>,
        ),
        (With<Player>, Without<Dead>),
    >,
) {
    let Ok((entity, mut health, mut action_state, mut animations)) = player.get_single_mut() else {
        return;
    };

    if health.dead() {
        println!("You Died!");

        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/death.wav")),
            PlaybackSettings::DESPAWN,
        ));

        animations.set_animation_one_shot(PlayerAnimation::Death);

        action_state.disable_all_actions();
        commands.entity(entity).insert(Dead);
    }
}

pub(super) fn hook_collision(
    mut player: Query<
        (
            // &Collision,
            &mut Health,
            &mut AnimationController<PlayerAnimation>,
        ),
        With<Player>,
    >,
    mut reader: EventReader<HookTargetCollision>,
    damage_query: Query<&HookedDamage>,
    active_collisions: Local<Vec<Entity>>,
) {
    let Ok((mut health, mut animations)) = player.get_single_mut() else {
        return;
    };

    for _ in reader
        .read()
        .filter(|c| c.shield_down() && damage_query.get(c.entity()).is_ok())
    {
        // TODO: trigger collision for health + trigger must leave before you get hit again +
        // kickback
        health.damage(1);
        animations.set_animation_one_shot(PlayerAnimation::Hit);
        println!("Ouch! [{}/{}]", health.current(), health.max());
    }
}
