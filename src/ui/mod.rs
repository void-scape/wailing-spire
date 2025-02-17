use bevy::prelude::*;
use bevy_pixel_gfx::pixel_perfect::{OuterCamera, HIGH_RES_LAYER};

use crate::{health::Health, player::Player};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui).add_systems(
            PostUpdate,
            (move_root, update_health).before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct HealthNode;

/// Converts a world-space pixel into a UI pixel.
fn uipx(px: f32) -> Val {
    Val::Px(px * 6.0)
}

fn spawn_ui(mut commands: Commands, assets: Res<AssetServer>) {
    commands
        .spawn((
            UiRoot,
            Node {
                width: Val::Percent(20.0),
                height: Val::Percent(20.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            // BackgroundColor(Color::srgb(0.8, 0.8, 1.)),
            HIGH_RES_LAYER,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: uipx(34.0),
                        height: uipx(6.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(Color::srgb(0.213, 0.1, 0.183)),
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: uipx(32.0),
                            height: uipx(4.0),
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                HealthNode,
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::ZERO,
                                    top: Val::ZERO,
                                    bottom: Val::ZERO,
                                    width: Val::Percent(100.0),
                                    ..Default::default()
                                },
                                BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
                            ));

                            parent
                                .spawn(Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::ZERO,
                                    top: Val::ZERO,
                                    bottom: Val::ZERO,
                                    right: Val::ZERO,
                                    ..Default::default()
                                })
                                .with_child(ImageNode {
                                    image: assets.load("sprites/health-box.png"),
                                    ..Default::default()
                                });
                        });
                });
        });
}

fn move_root(
    mut root: Query<&mut Transform, With<UiRoot>>,
    cam: Query<&Transform, (With<OuterCamera>, Without<UiRoot>)>,
) {
    let Ok(mut root) = root.get_single_mut() else {
        return;
    };

    let Ok(cam) = cam.get_single() else {
        return;
    };

    root.translation = cam.translation;
}

fn update_health(mut node: Query<&mut Node, With<HealthNode>>, player: Query<&Health>) {
    let Ok(health) = player.get_single() else {
        return;
    };

    let Ok(mut node) = node.get_single_mut() else {
        return;
    };

    node.width = Val::Percent(health.proportion() * 100.0);
}
