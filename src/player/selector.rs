use super::{
    input::{ActiveInputType, InputType, CONTROLLER_SELECTOR_MAP, KEYBOARD_SELECTOR_MAP},
    Action, Homing, Player, Velocity,
};
use bevy::{prelude::*, sprite::Anchor, utils::HashMap};
use leafwing_input_manager::prelude::ActionState;
use selector::{MaxSelectors, Selector, SelectorInfo, SelectorTarget};

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

pub(super) fn clear_removed_entities(
    mut q: Query<&mut SelectorInfo>,
    targets: Query<Entity, With<SelectorTarget>>,
) {
    for mut info in q.iter_mut() {
        if info.target.is_some_and(|t| targets.get(t).is_err()) {
            info.target = None;
        }
    }
}

#[derive(Resource)]
pub(super) struct SelectorTextureCache {
    map: HashMap<InputType, SelectorTexture>,
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

    let layout = atlases.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        5,
        1,
        None,
        None,
    ));

    map.insert(
        InputType::Keyboard,
        SelectorTexture {
            image: server.load("sprites/keyboard_selector.png"),
            atlas_map: {
                let mut atlas_map = HashMap::default();
                for (selector, button) in KEYBOARD_SELECTOR_MAP.iter() {
                    let index = match button {
                        KeyCode::KeyH => 1,
                        KeyCode::KeyJ => 2,
                        KeyCode::KeyK => 3,
                        KeyCode::KeyL => 4,
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

    map.insert(
        InputType::Controller,
        SelectorTexture {
            image: server.load("sprites/xbox_selector.png"),
            atlas_map: {
                let mut atlas_map = HashMap::default();
                for (selector, button) in CONTROLLER_SELECTOR_MAP.iter() {
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
    mut sprite_query: Query<(Entity, &mut Sprite)>,
    sprites: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    textures: Res<SelectorTextureCache>,
    input: Res<ActiveInputType>,
) {
    let selector_texture = textures
        .map
        .get(&input.ty())
        .expect("unregistered input type");

    if input.is_changed() {
        for (entity, _, selector) in selector_sprites.iter() {
            if let Ok((_, mut sprite)) = sprite_query.get_mut(entity) {
                *sprite = selector_texture.sprite(&selector.0);
            }
        }
    }

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
            warn_once!("selector on entity with no sprite??");
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
