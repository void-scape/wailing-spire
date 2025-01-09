use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=assets/ldtk/wailing-spire.ldtk");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    for module in bevy_ldtk_scene::comp::walk_ldtk_dir(
        "assets/ldtk",
        bevy_ldtk_scene::comp::build_ldtk_world_mod,
    )
    .unwrap()
    .into_iter()
    {
        File::create(
            PathBuf::new()
                .join(&out_dir)
                .join(format!("{}_mod.rs", &module.world_name)),
        )
        .and_then(|mut file| file.write(module.data.as_bytes()))
        .unwrap_or_else(|e| {
            panic!(
                "Error while writing {} mod to file: {e}",
                &module.world_name
            )
        });
    }
}
