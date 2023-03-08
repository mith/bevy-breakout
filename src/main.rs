use bevy::prelude::*;
use camera::ScalingCameraPlugin;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use breakout::BreakoutPlugin;
use counters::CountersPlugin;
use local::LocalPlugin;
mod breakout;
mod camera;
mod collision;
mod counters;
mod local;
mod util;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(TaskPoolPlugin {
                // This seems to resolve stuttering in Bevy 0.10 on Linux
                task_pool_options: TaskPoolOptions::with_num_threads(1),
            }),
    )
    .insert_resource(ClearColor(Color::BLACK));

    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin);

    app.add_plugin(BreakoutPlugin)
        .add_plugin(LocalPlugin)
        .add_plugin(CountersPlugin)
        .add_plugin(ScalingCameraPlugin);

    app.run();
}
