#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_ldtk_scene::{prelude::*, process::tiles::LevelTileSets};
use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;
use physics::{
    gravity::Gravity,
    spatial::{SpatialHash, StaticBodyData},
};
use spire::*;

mod animation;
mod enemies;
mod entity_registry;
mod physics;
mod player;
mod spire;
mod test;

const WIDTH: f32 = 320.;
const HEIGHT: f32 = 180.;
const TILE_SIZE: f32 = 16.;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_pixel_gfx::PixelGfxPlugin(CanvasDimensions::new(WIDTH as u32, HEIGHT as u32)),
            bevy_ldtk_scene::LdtkScenePlugin,
            player::PlayerPlugin,
            entity_registry::EntityRegistryPlugin,
            physics::PhysicsPlugin,
            spire::SpirePlugin,
            enemies::EnemyPlugin,
        ))
        // .insert_resource(AlignCanvasToCamera(false))
        .register_required_components_with::<LevelTileSets, SpatialHash<StaticBodyData>>(|| {
            SpatialHash::<StaticBodyData>::new(32.)
        })
        .insert_resource(Gravity(Vec2::NEG_Y * 15.))
        .add_systems(Update, close_on_escape)
        .add_systems(Startup, startup)
        .add_systems(Update, despawn)
        .run();
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        HotWorld(server.load("ldtk/spire.ldtk")),
        World(server.load("ldtk/spire.ron")),
        LevelLoader::levels_with_offset((StartLevel, RightLevel, UpLevel), Vec2::ZERO),
    ));
}

fn despawn(mut reader: EventReader<KeyboardInput>, loader: Option<Single<&mut LevelLoader>>) {
    if reader
        .read()
        .any(|i| !i.repeat && i.key_code == KeyCode::KeyR && i.state == ButtonState::Pressed)
    {
        if let Some(mut loader) = loader {
            println!("despawning right");
            loader.despawn(RightLevel::uid());
        }
    }
}
