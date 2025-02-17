#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_ldtk_scene::{levels::Stack, prelude::*, process::tiles::LevelTileSets};
use bevy_pixel_gfx::pixel_perfect::CanvasDimensions;
use health::Dead;
//use map::MapGen;
use physics::{
    gravity::Gravity,
    layers::{self},
    spatial::SpatialHash,
};
use player::{hook::OccludeHookTarget, PlayerHurtBox};
use spire::*;

mod animation;
mod enemies;
mod levels;
mod entities;
mod health;
mod lifetime;
mod map;
mod player;
mod spikes;
#[allow(unused)]
mod spire;
mod tween;

const WIDTH: f32 = 440.;
const HEIGHT: f32 = 248.;
const TILE_SIZE: f32 = 16.;
const LEVEL_SIZE: Vec2 = Vec2::splat(20.);

fn main() {
    App::default()
        .add_plugins((
            bevy_tween::DefaultTweenPlugins,
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            bevy_pixel_gfx::PixelGfxPlugin(CanvasDimensions::new(WIDTH as u32, HEIGHT as u32)),
            bevy_ldtk_scene::LdtkScenePlugin,
            player::PlayerPlugin,
            physics::PhysicsPlugin,
            spire::SpirePlugin,
            enemies::EnemyPlugin,
            entities::EntityPlugin,
            spikes::SpikePlugin,
            bevy_framepace::FramepacePlugin,
            bevy_enoki::EnokiPlugin,
            lifetime::LifeTimePlugin,
            health::HealthPlugin,
        ))
        .register_required_components::<spire::TileSolid, physics::collision::TilesetCollider>()
        .add_systems(Update, tween::despawn_finished_tweens)
        // .insert_resource(AlignCanvasToCamera(false))
        .register_required_components_with::<LevelTileSets, SpatialHash>(|| SpatialHash::new(32.))
        .register_required_components::<LevelTileSets, layers::Wall>()
        .register_required_components::<LevelTileSets, OccludeHookTarget>()
        .insert_resource(GlobalVolume::new(0.5))
        .insert_resource(Gravity(Vec2::NEG_Y * 10.))
        .insert_resource(ClearColor(srgb_from_hex(0x0d001a)))
        .add_systems(Update, close_on_escape)
        .add_systems(Startup, startup)
        .add_systems(Last, reset)
        .run();
}

pub fn srgb_from_hex(color: u32) -> Color {
    Color::srgb_u8(
        ((color >> 16) & 0xff) as u8,
        ((color >> 8) & 0xff) as u8,
        (color & 0xff) as u8,
    )
}

fn close_on_escape(mut reader: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for input in reader.read() {
        if input.state == ButtonState::Pressed && input.key_code == KeyCode::Escape {
            writer.send(AppExit::Success);
        }
    }
}

fn long() -> LevelLoader {
    LevelLoader::levels(Stack((Start, Up, Up, Up, Up, Up, Up, Up, Top)))
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    //let map = MapGen::<3, 5>::default().gen().trim_edge();
    //map::draw(&map);

    commands.spawn((
        HotWorld(server.load("ldtk/spire.ldtk")),
        World(server.load("ldtk/spire.ron")),
        // LevelLoader::levels_with_offset((StartLevel, RightLevel, UpLevel), Vec2::ZERO),
        // LevelLoader::levels_with_offset(map, Vec2::ZERO),
        long(),
    ));

    commands.spawn((
        AudioPlayer::new(server.load("audio/music/ambience.wav")),
        PlaybackSettings::LOOP,
    ));
}

fn reset(
    mut commands: Commands,
    server: Res<AssetServer>,
    world: Query<Entity, With<bevy_ldtk_scene::World>>,
    player: Query<&Dead, With<PlayerHurtBox>>,
) {
    if !player.is_empty() {
        let entity = world.single();
        commands.entity(entity).despawn_recursive();
        commands.spawn((
            HotWorld(server.load("ldtk/spire.ldtk")),
            World(server.load("ldtk/spire.ron")),
            // LevelLoader::levels_with_offset((StartLevel, RightLevel, UpLevel), Vec2::ZERO),
            // LevelLoader::levels_with_offset(map, Vec2::ZERO),
            long(),
        ));
    }
}
