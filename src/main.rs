#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_ldtk_scene::{HotWorld, World};
use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;
use physics::gravity::Gravity;

mod animation;
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
        ))
        // .insert_resource(AlignCanvasToCamera(false))
        .insert_resource(Gravity(Vec2::NEG_Y * 15.))
        .add_systems(Update, close_on_escape)
        .add_systems(Startup, startup)
        // .add_systems(Update, restart)
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
        // LevelSelect::new(spire::level_one::LevelOne),
    ));
}

// fn restart(
//     mut commands: Commands,
//     mut reader: EventReader<KeyboardInput>,
//     server: Res<AssetServer>,
//     world_query: Query<Entity, With<World>>,
// ) {
//     if reader
//         .read()
//         .any(|i| !i.repeat && i.key_code == KeyCode::KeyR && i.state == ButtonState::Pressed)
//     {
//         for entity in world_query.iter() {
//             commands.entity(entity).despawn_recursive();
//         }
//
//         commands.spawn((
//             World(server.load("ldtk/spire.ldtk")),
//             LevelSelect::new(spire::level_one::LevelOne),
//         ));
//     }
// }
