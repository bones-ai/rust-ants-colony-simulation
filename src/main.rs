use bevy::{
    core_pipeline::{tonemapping::Tonemapping, bloom::BloomSettings},
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    window::WindowMode,
};

use ants::{
    ant::{AntFollowCameraPos, AntPlugin},
    gui::{GuiPlugin, SimSettings},
    pathviz::PathVizPlugin,
    pheromone::PheromonePlugin,
    *,
};
use bevy_pancam::{PanCam, PanCamPlugin};

#[derive(Component)]
struct FollowCamera;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        // mode: WindowMode::Fullscreen,
                        focused: true,
                        resolution: (W, H).into(),
                        title: "Ants".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        // External plugins & systems
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(PanCamPlugin)
        // Default Resources
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
        .insert_resource(Msaa::Off)
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, ant_follow_camera)
        // Internal Plugins
        .add_plugins(AntPlugin)
        .add_plugins(PheromonePlugin)
        .add_plugins(PathVizPlugin)
        .add_plugins(GuiPlugin)
        .run();
}

fn ant_follow_camera(
    ant_pos: Res<AntFollowCameraPos>,
    sim_settings: Res<SimSettings>,
    mut camera_query: Query<&mut Transform, With<FollowCamera>>,
) {
    if !sim_settings.is_camera_follow {
        return;
    }

    let mut transform = camera_query.single_mut();
    transform.translation = vec3(ant_pos.0.x, ant_pos.0.y, ANT_Z_INDEX);
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                tonemapping: Tonemapping::TonyMcMapface,
                ..default()
            },
            BloomSettings::default(),
            FollowCamera,
        ))
        .insert(PanCam::default());

    // Ant colony sprite
    commands.spawn(SpriteBundle {
        texture: asset_server.load(SPRITE_ANT_COLONY),
        sprite: Sprite {
            color: Color::rgb(1.5, 1.5, 1.5),
            ..default()
        },
        transform: Transform::from_xyz(HOME_LOCATION.0, HOME_LOCATION.1, 2.0)
            .with_scale(Vec3::splat(HOME_SPRITE_SCALE)),
        ..Default::default()
    });

    // Food sprite
    commands.spawn(SpriteBundle {
        texture: asset_server.load(SPRITE_FOOD),
        sprite: Sprite {
            color: Color::rgb(1.5, 1.5, 1.5),
            ..default()
        },
        transform: Transform::from_xyz(FOOD_LOCATION.0, FOOD_LOCATION.1, 2.0)
            .with_scale(Vec3::splat(FOOD_SPRITE_SCALE)),
        ..Default::default()
    });
}
