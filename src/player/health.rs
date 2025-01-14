use super::{Action, Player};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

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
        (Entity, &mut Health, &mut ActionState<Action>),
        (With<Player>, Without<Dead>),
    >,
) {
    let Ok((entity, mut health, mut action_state)) = player.get_single_mut() else {
        return;
    };

    if health.dead() {
        println!("You Died!");

        commands.spawn((
            AudioPlayer::new(server.load("audio/sfx/death.wav")),
            PlaybackSettings::DESPAWN,
        ));

        action_state.disable_all_actions();
        commands.entity(entity).insert(Dead);
    }
}
