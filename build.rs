use bevy_ldtk_scene::extract::world::{ExtractedComponent, FromLdtkWorld, IntoExtractedComponent};
use bevy_ldtk_scene::prelude::*;
use bevy_ldtk_scene::world::ExtractLdtkWorld;
use quote::{quote, TokenStreamExt};
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=assets/ldtk/wailing-spire.ldtk");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let world = ExtractLdtkWorld::new("assets/ldtk/spire.ldtk")
        .unwrap()
        .extract_with((
            ExtractComposites,
            ExtractEnums,
            ExtractEntityTypes,
            ExtractTileSets,
            ExtractCompEntities,
            ExtractEntityInstances,
            ExtractLevelUids,
            ExtractLevelExts,
        ))
        .unwrap()
        .write(PathBuf::new().join(out_dir).join("spire.rs"))
        // .write(PathBuf::new().join("test.rs"))
        .unwrap()
        .world();
    world.save("assets/ldtk/spire.ron").unwrap();
}

pub struct ExtractLevelExts;

impl IntoExtractedComponent<ExtractedLevelExts> for ExtractLevelExts {
    type Context = ();

    fn extract(self, world: &ExtractLdtkWorld, _: Self::Context) -> ExtractedLevelExts {
        ExtractedLevelExts(LevelNames::from_world(world))
    }
}

pub struct ExtractedLevelExts(LevelNames);

impl ExtractedComponent for ExtractedLevelExts {
    fn plugin(&self, output: &mut proc_macro2::TokenStream) {
        for name in self.0 .0.iter() {
            println!("{:#?}", name.to_string());
            output.append_all(quote! {
                {
                    let mut registry = app.world_mut().get_resource_or_insert_with(|| crate::levels::LevelRegistry::default());
                    registry.push(#name::default());
                }
            });
        }
    }
}
