pub struct SpirePlugin;
impl ::bevy::app::Plugin for SpirePlugin {
    fn build(&self, app: &mut ::bevy::app::App) {
        app.register_type::<Test>();
        app.register_type::<TileSolid>();
        app.register_type::<Knight>();
        app.register_type::<DynamicCameraAnchor>();
        app.register_type::<MovingPlatform>();
        ::bevy_ldtk_scene::extract::tiles::TileSetAppExt::register_tileset_component(
            app,
            ::bevy_ldtk_scene::extract::tiles::TileSetUid(1i64),
            TileSolid,
            &[
                0u32,
                1u32,
                2u32,
                3u32,
                4u32,
                5u32,
                6u32,
                7u32,
                8u32,
                9u32,
                10u32,
                11u32,
                12u32,
                13u32,
                15u32,
                16u32,
                17u32,
                18u32,
                20u32,
                21u32,
                22u32,
                23u32,
                24u32,
                25u32,
                26u32,
                27u32,
                28u32,
                29u32,
                30u32,
                31u32,
                32u32,
                33u32,
                35u32,
                40u32,
                41u32,
                42u32,
                43u32,
                44u32,
                45u32,
                46u32,
                47u32,
                48u32,
                49u32,
                50u32,
                51u32,
                52u32,
                53u32,
                54u32,
                55u32,
                60u32,
                61u32,
                62u32,
                63u32,
                64u32,
                65u32,
                66u32,
                67u32,
                68u32,
                69u32,
                70u32,
                71u32,
                72u32,
                73u32,
                80u32,
                81u32,
                82u32,
                83u32,
                84u32,
                85u32,
                86u32,
                87u32,
                88u32,
                89u32,
                100u32,
                101u32,
                102u32,
                103u32,
                104u32,
                105u32,
                106u32,
                107u32,
                108u32,
                109u32,
                120u32,
                121u32,
                122u32,
                123u32,
                124u32,
                125u32,
                126u32,
                127u32,
                128u32,
                129u32,
                140u32,
                141u32,
                142u32,
                143u32,
                144u32,
                145u32,
                146u32,
                147u32,
                148u32,
                149u32,
                160u32,
                161u32,
                162u32,
                163u32,
                164u32,
                165u32,
                166u32,
                167u32,
                168u32,
                169u32,
            ],
        );
        let system = app.register_system(entities_0);
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(|| ::bevy_ldtk_scene::process::entities::LevelEntityRegistry::default());
        registry.entities.insert(::bevy_ldtk_scene::world::LevelUid(0i64), system);
        ::bevy_ldtk_scene::process::ProcessAppExt::register_processor::<
            ::bevy_ldtk_scene::process::composites::CompositeProcessor,
        >(app);
    }
}
mod tile_solid {
    use ::bevy::prelude::ReflectComponent;
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[reflect(Component)]
    pub struct TileSolid;
}
#[allow(unused)]
pub use tile_solid::TileSolid;
mod test {
    use ::bevy::prelude::ReflectComponent;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[derive(::serde::Serialize, ::serde::Deserialize)]
    #[reflect(Component)]
    pub enum Test {
        Test0,
    }
}
#[allow(unused)]
pub use test::Test;
mod knight {
    use ::bevy::prelude::ReflectComponent;
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[reflect(Component)]
    pub struct Knight;
}
#[allow(unused)]
pub use knight::Knight;
mod dynamic_camera_anchor {
    use ::bevy::prelude::ReflectComponent;
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[reflect(Component)]
    pub struct DynamicCameraAnchor {
        pub radius: f32,
        pub speed: f32,
    }
}
#[allow(unused)]
pub use dynamic_camera_anchor::DynamicCameraAnchor;
mod moving_platform {
    use ::bevy::prelude::ReflectComponent;
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[derive(::bevy::ecs::component::Component, ::bevy::reflect::Reflect)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[reflect(Component)]
    pub struct MovingPlatform;
}
#[allow(unused)]
pub use moving_platform::MovingPlatform;
fn entities_0(
    mut commands: ::bevy::prelude::Commands,
    server: ::bevy::prelude::Res<::bevy::prelude::AssetServer>,
    mut atlases: ::bevy::prelude::ResMut<
        ::bevy::prelude::Assets<::bevy::prelude::TextureAtlasLayout>,
    >,
) {
    let layout_0 = atlases
        .add(::bevy::prelude::TextureAtlasLayout {
            size: bevy::prelude::UVec2::new(256u32, 256u32),
            textures: vec![
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 0u32),
                max : ::bevy::prelude::UVec2::new(32u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32, 0u32),
                max : ::bevy::prelude::UVec2::new(64u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32, 0u32),
                max : ::bevy::prelude::UVec2::new(96u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32, 0u32),
                max : ::bevy::prelude::UVec2::new(128u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32, 0u32),
                max : ::bevy::prelude::UVec2::new(160u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32, 0u32),
                max : ::bevy::prelude::UVec2::new(192u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32, 0u32),
                max : ::bevy::prelude::UVec2::new(224u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32, 0u32),
                max : ::bevy::prelude::UVec2::new(256u32, 32u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 32u32),
                max : ::bevy::prelude::UVec2::new(32u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32, 32u32),
                max : ::bevy::prelude::UVec2::new(64u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32, 32u32),
                max : ::bevy::prelude::UVec2::new(96u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32, 32u32),
                max : ::bevy::prelude::UVec2::new(128u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                32u32), max : ::bevy::prelude::UVec2::new(160u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                32u32), max : ::bevy::prelude::UVec2::new(192u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                32u32), max : ::bevy::prelude::UVec2::new(224u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                32u32), max : ::bevy::prelude::UVec2::new(256u32, 64u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 64u32),
                max : ::bevy::prelude::UVec2::new(32u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32, 64u32),
                max : ::bevy::prelude::UVec2::new(64u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32, 64u32),
                max : ::bevy::prelude::UVec2::new(96u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32, 64u32),
                max : ::bevy::prelude::UVec2::new(128u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                64u32), max : ::bevy::prelude::UVec2::new(160u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                64u32), max : ::bevy::prelude::UVec2::new(192u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                64u32), max : ::bevy::prelude::UVec2::new(224u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                64u32), max : ::bevy::prelude::UVec2::new(256u32, 96u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 96u32),
                max : ::bevy::prelude::UVec2::new(32u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32, 96u32),
                max : ::bevy::prelude::UVec2::new(64u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32, 96u32),
                max : ::bevy::prelude::UVec2::new(96u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32, 96u32),
                max : ::bevy::prelude::UVec2::new(128u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                96u32), max : ::bevy::prelude::UVec2::new(160u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                96u32), max : ::bevy::prelude::UVec2::new(192u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                96u32), max : ::bevy::prelude::UVec2::new(224u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                96u32), max : ::bevy::prelude::UVec2::new(256u32, 128u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 128u32),
                max : ::bevy::prelude::UVec2::new(32u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32,
                128u32), max : ::bevy::prelude::UVec2::new(64u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32,
                128u32), max : ::bevy::prelude::UVec2::new(96u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32,
                128u32), max : ::bevy::prelude::UVec2::new(128u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                128u32), max : ::bevy::prelude::UVec2::new(160u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                128u32), max : ::bevy::prelude::UVec2::new(192u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                128u32), max : ::bevy::prelude::UVec2::new(224u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                128u32), max : ::bevy::prelude::UVec2::new(256u32, 160u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 160u32),
                max : ::bevy::prelude::UVec2::new(32u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32,
                160u32), max : ::bevy::prelude::UVec2::new(64u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32,
                160u32), max : ::bevy::prelude::UVec2::new(96u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32,
                160u32), max : ::bevy::prelude::UVec2::new(128u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                160u32), max : ::bevy::prelude::UVec2::new(160u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                160u32), max : ::bevy::prelude::UVec2::new(192u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                160u32), max : ::bevy::prelude::UVec2::new(224u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                160u32), max : ::bevy::prelude::UVec2::new(256u32, 192u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 192u32),
                max : ::bevy::prelude::UVec2::new(32u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32,
                192u32), max : ::bevy::prelude::UVec2::new(64u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32,
                192u32), max : ::bevy::prelude::UVec2::new(96u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32,
                192u32), max : ::bevy::prelude::UVec2::new(128u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                192u32), max : ::bevy::prelude::UVec2::new(160u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                192u32), max : ::bevy::prelude::UVec2::new(192u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                192u32), max : ::bevy::prelude::UVec2::new(224u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                192u32), max : ::bevy::prelude::UVec2::new(256u32, 224u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(0u32, 224u32),
                max : ::bevy::prelude::UVec2::new(32u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(32u32,
                224u32), max : ::bevy::prelude::UVec2::new(64u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(64u32,
                224u32), max : ::bevy::prelude::UVec2::new(96u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(96u32,
                224u32), max : ::bevy::prelude::UVec2::new(128u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(128u32,
                224u32), max : ::bevy::prelude::UVec2::new(160u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(160u32,
                224u32), max : ::bevy::prelude::UVec2::new(192u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(192u32,
                224u32), max : ::bevy::prelude::UVec2::new(224u32, 256u32), },
                ::bevy::prelude::URect { min : ::bevy::prelude::UVec2::new(224u32,
                224u32), max : ::bevy::prelude::UVec2::new(256u32, 256u32), }
            ],
        });
    commands
        .spawn((
            ::bevy_ldtk_scene::process::entities::LevelEntity,
            ::bevy_ldtk_scene::extract::entities::EntityUid(71i64),
            ::bevy::prelude::Transform::from_xyz(192f32, -144f32, 6f32),
            DynamicCameraAnchor {
                radius: 128f32,
                speed: 1000f32,
            },
        ));
    commands
        .spawn((
            ::bevy_ldtk_scene::process::entities::LevelEntity,
            ::bevy_ldtk_scene::extract::entities::EntityUid(70i64),
            ::bevy::prelude::Transform::from_xyz(110f32, -176f32, 4f32),
            ::bevy::prelude::Sprite {
                image: server.load("ldtk/../sprites/characters/knight.png"),
                rect: None,
                texture_atlas: Some(::bevy::prelude::TextureAtlas {
                    index: 0,
                    layout: layout_0.clone(),
                }),
                anchor: ::bevy::sprite::Anchor::TopLeft,
                ..Default::default()
            },
            Knight,
        ));
    commands
        .spawn((
            ::bevy_ldtk_scene::process::entities::LevelEntity,
            ::bevy_ldtk_scene::extract::entities::EntityUid(75i64),
            ::bevy::prelude::Transform::from_xyz(64f32, -96f32, 4f32),
            ::bevy::prelude::Sprite {
                image: server
                    .load(
                        "ldtk/../sprites/tilesets/Inca_front_by_Kronbits-extended.png",
                    ),
                rect: Some(::bevy::prelude::Rect::new(32f32, 80f32, 64f32, 96f32)),
                texture_atlas: None,
                anchor: ::bevy::sprite::Anchor::TopLeft,
                ..Default::default()
            },
            MovingPlatform,
        ));
}
