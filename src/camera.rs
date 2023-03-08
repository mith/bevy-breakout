use bevy::{prelude::*, window::PrimaryWindow};

use crate::breakout::BreakoutConfig;

fn calculate_court_scale(window: &Window, config: &BreakoutConfig) -> f32 {
    let height_ratio = window.height() / config.court_size[1];
    let width_ratio = window.width() / config.court_size[0];
    (1. / height_ratio.min(width_ratio)) * (1. / config.scale)
}

pub(crate) fn setup_camera(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();

    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: calculate_court_scale(window, &config),
            ..default()
        },
        ..default()
    });
}

pub(crate) fn adjust_camera_scale(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    mut ui_scale: ResMut<UiScale>,
    config: Res<BreakoutConfig>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();

    let current_scale = calculate_court_scale(window, &config);
    for mut projection in &mut query {
        projection.scale = current_scale;
    }

    ui_scale.scale = 1. / current_scale as f64;
}

pub(crate) struct ScalingCameraPlugin;

impl Plugin for ScalingCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(adjust_camera_scale);
    }
}
