#![allow(clippy::type_complexity)]

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_pixel_gfx::{camera::bind_camera, pixel_perfect::CanvasDimensions};
use physics::gravity::Gravity;

mod animation;
mod entity_registry;
mod physics;
mod player;
mod spire;

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
        ))
        // .insert_resource(AlignCanvasToCamera(false))
        .insert_resource(Gravity(Vec2::NEG_Y * 10.))
        .add_systems(Update, close_on_escape)
        .add_systems(Update, level_one)
        .run();
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn level_one(
    mut commands: Commands,
    mut reader: EventReader<KeyboardInput>,
    mut level_entity: Local<Option<Entity>>,
) {
    if level_entity.is_none()
        || reader
            .read()
            .any(|i| !i.repeat && i.key_code == KeyCode::KeyR && i.state == ButtonState::Pressed)
    {
        let entity = commands.spawn_empty().id();
        commands.run_system_cached_with(spire::level_one::spawn, entity);
        if level_entity.is_none() {
            commands.run_system_cached(bind_camera::<player::Player>);
        }

        level_entity.map(|e| commands.entity(e).despawn_recursive());
        *level_entity = Some(entity);
    }
}
