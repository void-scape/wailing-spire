use super::{Player, PlayerSettings};
use crate::TILE_SIZE;
use bevy::prelude::*;
use bevy_ldtk_scene::extract::levels::LevelMeta;
use bevy_ldtk_scene::levels::Level;
use bevy_pixel_gfx::camera::MainCamera;

#[derive(Resource, Default)]
pub struct CameraConstraints {
    pub top: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub right: Option<f32>,
}

#[derive(Component)]
pub struct CurrentLevel(pub LevelMeta);

pub fn update_current_level(
    mut commands: Commands,
    player: Query<(Entity, &GlobalTransform), With<Player>>,
    level_query: Query<(&GlobalTransform, &Level)>,
) {
    let Ok((entity, player)) = player.get_single() else {
        return;
    };

    if let Some(level) = level_query
        .iter()
        .find(|(t, l)| l.meta().rect(t).contains(player.translation().xy()))
        .map(|(_, l)| l)
    {
        commands.entity(entity).insert(CurrentLevel(*level.meta()));
    }
}

pub fn move_camera(
    mut cam: Query<&mut Transform, With<MainCamera>>,
    player: Query<(&GlobalTransform, &CurrentLevel), (With<Player>, Without<MainCamera>)>,
    level_query: Query<(&GlobalTransform, &Level)>,
    settings: Res<PlayerSettings>,
    constraints: Res<CameraConstraints>,
) {
    let Ok(mut cam) = cam.get_single_mut() else {
        return;
    };

    let Ok((player, level)) = player.get_single() else {
        return;
    };

    if let Some(level_transform) = level_query
        .iter()
        .find(|(_, l)| l.uid() == level.0.uid)
        .map(|(t, _)| t)
    {
        let x = level.0.size.x / 2. + level_transform.translation().x;
        // let x = player.translation().x;
        let target_position = Vec3::new(x, player.translation().y + TILE_SIZE * 1.5, 0.);
        let delta = target_position - cam.translation;

        cam.translation += delta * settings.camera_speed;

        if let Some(left) = constraints.left {
            if cam.translation.x < left {
                cam.translation.x = left;
            }
        }

        if let Some(right) = constraints.right {
            if cam.translation.x > right {
                cam.translation.x = right;
            }
        }

        if let Some(top) = constraints.top {
            if cam.translation.y > top {
                cam.translation.y = top;
            }
        }

        if let Some(bottom) = constraints.bottom {
            if cam.translation.y < bottom {
                cam.translation.y = bottom;
            }
        }
    }
}
