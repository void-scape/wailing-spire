use std::collections::VecDeque;

use bevy::prelude::*;
use itertools::Itertools;
use physics::collision::Collider;

/// A marker component that will include this entity
/// in the selector calculations.
#[derive(Debug, Default, Component)]
pub struct SelectorTarget;

/// The source that selectors are relative to.
///
/// This will generally just be the player.
#[derive(Debug, Default, Component)]
pub struct SelectorSource;

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

#[derive(Default, Component)]
pub struct SelectorInfo {
    pub target: Option<Entity>,
    tick: usize,
    history: VecDeque<Entity>,
}

impl SelectorInfo {
    fn set_target(&mut self, target: Entity, tick: &mut SelectorTick) {
        self.clear_target();
        tick.0 += 1;
        self.tick = tick.0;

        self.target = Some(target);
    }

    fn clear_target(&mut self) {
        if let Some(last_target) = self.target.take() {
            if !self.history.contains(&last_target) {
                if self.history.len() > 5 {
                    self.history.pop_front();
                }
                self.history.push_back(last_target);
            }
        }
    }
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
            + self.selector.familiarity * 2.0
            + self.selector.recency
            + self.selector.stability * 2.0
            + self.distance_above_player
    }
}

pub fn calculate_selectors(
    collider_targets: Query<(Entity, &GlobalTransform, &Collider), With<SelectorTarget>>,
    player: Query<(&GlobalTransform, &Collider), With<SelectorSource>>,
    mut selectors: Query<(&Selector, &mut SelectorInfo)>,
    mut selector_tick: ResMut<SelectorTick>,
    max_selectors: Res<MaxSelectors>,
) {
    let Ok((ptrans, pcoll)) = player.get_single() else {
        return;
    };

    let player_center = pcoll.global_absolute(ptrans).center();

    let viable: Vec<_> = collider_targets
        .iter()
        .filter_map(|(e, t, c)| {
            let center = c.global_absolute(t).center();
            if center.distance(player_center) < 256.0 {
                Some((e, center))
            } else {
                None
            }
        })
        .collect();

    let mut collected_selectors: Vec<_> = selectors.iter().map(|(s, i)| (*s, i)).collect();
    collected_selectors.sort_by_key(|pair| pair.0);
    let mut processed_targets = Vec::new();

    for viable in &viable {
        let Ok((_, vtrans, vcoll)) = collider_targets.get(viable.0) else {
            continue;
        };

        let center = vcoll.global_absolute(vtrans).center();

        let distance = player_center.distance(center);
        let direction = (center - player_center).normalize_or_zero();

        processed_targets.push(ProcessedTarget {
            entity: viable.0,
            distance,
            vertical_distance: center.y - player_center.y,
            direction,
            selector: selectors
                .iter()
                .find_map(|(s, i)| i.target.is_some_and(|e| e == viable.0).then_some(*s)),
        });
    }

    let pool_size = viable.len().min(max_selectors.0);
    if pool_size == 0 {
        return;
    }

    let mut all_evaluations = Vec::new();
    let mut greatest_distance = 0.0;
    let mut greatest_distance_above = 0.0;
    for group in processed_targets.iter().permutations(pool_size) {
        for selector_group in collected_selectors.iter().permutations(pool_size) {
            let mut eval = TargetScores::default();

            for (target, (selector, info)) in group.iter().zip(&selector_group) {
                eval.distance += target.distance.abs();
                eval.distance_above_player += target.vertical_distance;
                if target.selector.is_some_and(|s| s == *selector) {
                    eval.selector.stability += 1.0 / pool_size as f32;
                }

                if info.history.contains(&target.entity) {
                    eval.selector.familiarity += 1.0 / pool_size as f32;
                }
            }

            if eval.distance > greatest_distance {
                greatest_distance = eval.distance;
            }

            if eval.distance_above_player > greatest_distance_above {
                greatest_distance_above = eval.distance_above_player;
            }

            all_evaluations.push((group.clone(), selector_group, eval));
        }
    }

    // now we normalize the distance scores
    for score in all_evaluations.iter_mut() {
        if greatest_distance != 0.0 {
            score.2.distance /= greatest_distance;
        }

        if greatest_distance_above != 0.0 {
            score.2.distance_above_player /= greatest_distance_above;
        }
    }

    all_evaluations.sort_unstable_by(|a, b| a.2.sum().total_cmp(&b.2.sum()).reverse());
    let best = all_evaluations.into_iter().next();

    if let Some((targets, scored_selectors, _)) = best {
        let scored_selectors = scored_selectors
            .into_iter()
            .map(|s| s.0)
            .collect::<Vec<_>>();

        for (s, mut info) in selectors.iter_mut() {
            let position = scored_selectors.iter().position(|p| *p == *s);

            if let Some(position) = position {
                info.set_target(targets[position].entity, &mut selector_tick);
            } else {
                info.clear_target();
            }
        }
    }
}
