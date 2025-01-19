use super::{hook::ViableTargets, Action, Collider, Homing, Player, Velocity};
use bevy::prelude::*;
use itertools::Itertools;
use leafwing_input_manager::prelude::ActionState;

/// A selector ID.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[require(SelectorInfo)]
pub struct Selector(pub usize);

/// The maximum number of hook selectors,
/// which may differ depending on the platform.
#[derive(Debug, Resource)]
pub struct MaxSelectors(pub usize);

/// A count for ordering selectors by
/// last-used.
#[derive(Resource, Default)]
pub struct SelectorTick(usize);

#[derive(Default, Component, Clone, Copy)]
pub struct SelectorInfo {
    pub target: Option<Entity>,
    tick: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct SelectorScore {
    // prefer to not move selectors
    stability: f32,
    // prefer selectors that haven't been changed recently
    recency: f32,

    // prefer selectors that have previously selected an entity
    familiarity: f32,
}

#[derive(Clone, Copy, Debug)]
struct ProcessedTarget {
    entity: Entity,
    distance: f32,
    vertical_distance: f32,
    direction: Vec2,
    selector: Option<Selector>,
}

#[derive(Default, Debug)]
struct TargetScores {
    distance: f32,
    angle: f32,
    selector: SelectorScore,
    distance_above_player: f32,
}

impl TargetScores {
    /// The sum of all scores.
    pub fn sum(&self) -> f32 {
        self.distance
            + self.angle
            + self.selector.familiarity
            + self.selector.recency
            + self.selector.stability * 2.0
            + self.distance_above_player
    }
}

pub(super) fn calculate_selectors(
    collider_targets: Query<(Entity, &GlobalTransform, &Collider)>,
    player: Query<(&ActionState<Action>, &GlobalTransform, &Collider), With<Player>>,
    viable: Res<ViableTargets>,
    mut selectors: Query<(&Selector, &mut SelectorInfo)>,
    mut selector_tick: ResMut<SelectorTick>,
    max_selectors: Res<MaxSelectors>,
) {
    let Ok((_, ptrans, pcoll)) = player.get_single() else {
        return;
    };

    let player_center = pcoll.global_absolute(ptrans).center();

    let mut collected_selectors: Vec<_> = selectors.iter().map(|(s, i)| (*s, *i)).collect();
    collected_selectors.sort_by_key(|pair| pair.0);
    let mut processed_targets = Vec::new();

    for viable in &viable.0 {
        let Ok((_, vtrans, vcoll)) = collider_targets.get(viable.entity) else {
            continue;
        };

        let center = vcoll.global_absolute(vtrans).center();

        let distance = player_center.distance(center);
        let direction = (center - player_center).normalize_or_zero();

        processed_targets.push(ProcessedTarget {
            entity: viable.entity,
            distance,
            vertical_distance: center.y - player_center.y,
            direction,
            selector: selectors
                .iter()
                .find_map(|(s, i)| i.target.is_some_and(|e| e == viable.entity).then_some(*s)),
        });
    }

    let pool_size = viable.0.len().min(max_selectors.0);
    if pool_size == 0 {
        return;
    }

    let mut all_evaluations = Vec::new();
    let mut greatest_distance = 0.0;
    let mut greatest_distance_above = 0.0;
    for group in processed_targets.iter().permutations(pool_size) {
        let mut eval = TargetScores::default();

        for (i, target) in group.iter().enumerate() {
            let selector = Selector(i);

            eval.distance += target.distance.abs();
            eval.distance_above_player += target.vertical_distance;
            if target.selector.is_some_and(|s| s == selector) {
                eval.selector.stability += 1.0 / pool_size as f32;
            }
        }

        if eval.distance > greatest_distance {
            greatest_distance = eval.distance;
        }

        if eval.distance_above_player > greatest_distance_above {
            greatest_distance_above = eval.distance_above_player;
        }

        all_evaluations.push((group, eval));
    }

    // now we normalize the distance scores
    for score in all_evaluations.iter_mut() {
        if greatest_distance != 0.0 {
            score.1.distance /= greatest_distance;
        }

        if greatest_distance_above != 0.0 {
            score.1.distance_above_player /= greatest_distance_above;
        }
    }

    all_evaluations.sort_unstable_by(|a, b| a.1.sum().total_cmp(&b.1.sum()).reverse());

    if let Some(best) = all_evaluations.first() {
        for (s, mut info) in selectors.iter_mut() {
            if let Some(target) = best.0.get(s.0) {
                selector_tick.0 += 1;
                info.tick = selector_tick.0;
                info.target = Some(target.entity);
            } else {
                info.target = None;
            }
        }
    }

    // info!("evals: {:#?}", all_evaluations);
    // info!("selectors: {:#?}", &selectors.0);
}

pub(super) fn trigger_hook(
    player: Query<(Entity, &Velocity, &ActionState<Action>, Option<&Homing>), With<Player>>,
    selectors: Query<(&Selector, &SelectorInfo)>,
    max_selectors: Res<MaxSelectors>,
    server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Ok((player_entity, player_velocity, action, homing)) = player.get_single() else {
        return;
    };

    let selector = (0..max_selectors.0).find_map(|i| {
        let selector = Selector(i);

        if action.just_pressed(&Action::Hook(selector)) {
            Some(selector)
        } else {
            None
        }
    });

    if let Some(selector) = selector {
        let Some(info) = selectors
            .iter()
            .find_map(|(s, i)| (*s == selector).then_some(i))
        else {
            return;
        };

        let Some(target) = info.target else {
            return;
        };

        if homing.is_none() {
            commands.spawn((
                AudioPlayer::new(server.load("audio/sfx/hook.wav")),
                PlaybackSettings::DESPAWN,
            ));
            commands.entity(player_entity).insert(super::Homing {
                target,
                starting_velocity: player_velocity.0,
            });
        }
    }
}

pub(super) fn spawn_selectors(max: Res<MaxSelectors>, mut commands: Commands) {
    let colors = [
        Color::srgb(1.0, 1.0, 0.0),
        Color::srgb(0.0, 1.0, 0.0),
        Color::srgb(0.0, 0.0, 1.0),
        Color::srgb(1.0, 0.0, 0.0),
    ];

    for i in 0..max.0 {
        // let progress = (i as f32 / max.0 as f32) * std::f32::consts::TAU;
        // let r = progress.sin() * 0.5 + 0.5;
        // let g = (progress + std::f32::consts::PI * (1.0 / 3.0)).sin() * 0.5 + 0.5;
        // let b = (progress + std::f32::consts::PI * (1.0 / 3.0)).sin() * 0.5 + 0.5;
        // let color = Color::srgb(r, g, b);
        let color = colors[i % colors.len()];

        commands.spawn((Sprite::from_color(color, Vec2::new(6.0, 6.0)), Selector(i)));
    }
}

pub(super) fn move_selectors(
    mut sprites: Query<(&Selector, &SelectorInfo, &mut Transform)>,
    targets: Query<(&GlobalTransform, &Collider), Without<Selector>>,
) {
    for (_, info, mut sprite_trans) in sprites.iter_mut() {
        let Some(target_entity) = info.target else {
            sprite_trans.translation.x = -1000.0;
            return;
        };

        if let Ok((target, collider)) = targets.get(target_entity) {
            let center = collider.global_absolute(target).center();
            sprite_trans.translation.x = center.x;
            sprite_trans.translation.y = center.y + 12.0;
            sprite_trans.translation.z = 10.0;
        } else {
            sprite_trans.translation.x = -1000.0;
        }
    }
}
