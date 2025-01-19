use super::{
    hook::{HookTarget, ViableTargets},
    input::XBOX_SELECTOR_MAP,
    Action, Collider, Homing, Player, Velocity,
};
use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
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

#[derive(Resource)]
pub(super) struct ActiveSelectors(Vec<Entity>);

pub(super) fn calculate_selectors(
    collider_targets: Query<(Entity, &GlobalTransform, &Collider), With<HookTarget>>,
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

    let viable: Vec<_> = collider_targets
        .iter()
        .filter_map(|(e, t, c)| {
            let center = c.global_absolute(t).center();
            if center.distance(player_center) < 500.0 {
                Some(super::hook::ViableTarget {
                    entity: e,
                    translation: center,
                })
            } else {
                None
            }
        })
        .collect();

    let mut collected_selectors: Vec<_> = selectors.iter().map(|(s, i)| (*s, *i)).collect();
    collected_selectors.sort_by_key(|pair| pair.0);
    let mut processed_targets = Vec::new();

    for viable in &viable {
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

    if let Some(best) = all_evaluations.first() {
        for (s, mut info) in selectors.iter_mut() {
            let position = best.1.iter().position(|p| p.0 == *s);

            if let Some(position) = position {
                let target = best.0[position].entity;
                selector_tick.0 += 1;
                info.tick = selector_tick.0;
                info.target = Some(target);
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
            commands
                .entity(player_entity)
                .insert(super::Homing::new(target, player_velocity.0));
        }
    }
}

pub(super) fn spawn_selectors(max: Res<MaxSelectors>, mut commands: Commands) {
    for i in 0..max.0 {
        commands.spawn(Selector(i));
    }
}

#[derive(Resource)]
pub(super) struct SelectorTextureCache {
    map: HashMap<InputType, SelectorTexture>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum InputType {
    XBox,
    Keyboard,
}

#[derive(Clone)]
struct SelectorTexture {
    image: Handle<Image>,
    atlas_map: HashMap<Selector, TextureAtlas>,
}

impl SelectorTexture {
    pub fn sprite(&self, selector: &Selector) -> Sprite {
        Sprite {
            image: self.image.clone(),
            anchor: Anchor::TopLeft,
            texture_atlas: Some(
                self.atlas_map
                    .get(selector)
                    .unwrap_or_else(|| panic!("unregistered selector {:?}", selector))
                    .clone(),
            ),
            ..Default::default()
        }
    }
}

pub(super) fn insert_texture_cache(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut map = HashMap::default();

    map.insert(
        InputType::XBox,
        SelectorTexture {
            image: server.load("sprites/xbox_selector.png"),
            atlas_map: {
                let layout = atlases.add(TextureAtlasLayout::from_grid(
                    UVec2::splat(32),
                    5,
                    1,
                    None,
                    None,
                ));

                let mut atlas_map = HashMap::default();
                for (selector, button) in XBOX_SELECTOR_MAP.iter() {
                    let index = match button {
                        GamepadButton::South => 1,
                        GamepadButton::East => 2,
                        GamepadButton::West => 3,
                        GamepadButton::North => 4,
                        _ => unreachable!(),
                    };

                    atlas_map.insert(
                        *selector,
                        TextureAtlas {
                            layout: layout.clone(),
                            index,
                        },
                    );
                }

                atlas_map
            },
        },
    );

    commands.insert_resource(SelectorTextureCache { map });
}

/// Selector sprite entity.
///
/// Child of a [`SelectorInfo`] `target`.
#[derive(Component)]
pub(super) struct SelectorSprite(Selector);

pub(super) fn add_selectors(
    mut commands: Commands,
    selector_query: Query<(&Selector, &SelectorInfo)>,
    mut selector_sprites: Query<(Entity, &Parent, &mut SelectorSprite)>,
    sprite_query: Query<(Entity, &Sprite)>,
    sprites: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    textures: Res<SelectorTextureCache>,
) {
    // despawn old sprites
    for (entity, parent, _) in &selector_sprites {
        if !selector_query
            .iter()
            .any(|(_, info)| info.target.is_some_and(|t| t == parent.get()))
        {
            commands.entity(entity).despawn();
        }
    }

    let mut populated_selectors = Vec::with_capacity(8);

    for (_, parent, mut selector) in &mut selector_sprites {
        if let Some((i, (s, _))) = selector_query
            .iter()
            .enumerate()
            .find(|(_, (_, info))| info.target.is_some_and(|t| t == parent.get()))
        {
            populated_selectors.push(i);

            if selector.0 != *s {
                selector.0 = *s;
            }
        }
    }

    let selector_texture = textures.map.get(&InputType::XBox).unwrap();

    // spawn new sprites
    for (_, (selector, info)) in selector_query
        .iter()
        .enumerate()
        .filter(|(i, _)| !populated_selectors.contains(i))
        .filter(|(_, (_, info))| info.target.is_some())
    {
        let target = info.target.unwrap();
        let Ok((entity, sprite)) = sprite_query.get(target) else {
            // todo!("non sprite fallback");
            warn!("selector on entity with no sprite??");
            continue;
        };

        if let Some(sprite_atlas) = &sprite.texture_atlas {
            if let Some(atlas) = atlases.get(&sprite_atlas.layout) {
                if let Some(rect) = atlas.textures.get(sprite_atlas.index) {
                    let x = rect.width() as f32 / 2. - 16.;
                    if let Some(mut entity) = commands.get_entity(entity) {
                        entity.with_child((
                            SelectorSprite(*selector),
                            selector_texture.sprite(selector),
                            Transform::from_xyz(x, 16., 0.),
                        ));
                    }
                } else {
                    todo!("atlas not loaded fallback");
                }
            } else {
                todo!("invalid index fallback");
            }
        } else if let Some(image) = sprites.get(&sprite.image) {
            let x = image.width() as f32 / 2. - 16.;
            if let Some(mut entity) = commands.get_entity(entity) {
                entity.with_child((
                    SelectorSprite(*selector),
                    selector_texture.sprite(selector),
                    Transform::from_xyz(x, 16., 0.),
                ));
            }
        } else {
            todo!("sprite not loaded fallback");
        }
    }
}
