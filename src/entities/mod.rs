use bevy::prelude::*;

pub mod wall_hook;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(wall_hook::WallHookPlugin);
    }
}

#[macro_export]
macro_rules! impl_plugin {
    ($name:ident, $body:expr) => {
        pub struct $name;

        impl bevy::prelude::Plugin for $name {
            fn build(&self, app: &mut bevy::prelude::App) {
                let body = $body;
                body(app);
            }
        }
    };
}
