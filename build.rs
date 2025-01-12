use bevy_ldtk_scene::world::ExtractLdtkWorld;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=assets/ldtk/wailing-spire.ldtk");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let world = ExtractLdtkWorld::new("assets/ldtk/spire.ldtk")
        .unwrap()
        .extract()
        .unwrap()
        .write(PathBuf::new().join(out_dir).join("spire.rs"))
        // .write(PathBuf::new().join("test.rs"))
        .unwrap()
        .world();
    world.save("assets/ldtk/spire.ron").unwrap();
}
