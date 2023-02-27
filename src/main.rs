use bevy::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::ScalingCameraPlugin;
use iyes_loopless::prelude::*;

use breakout::BreakoutPlugin;
use counters::CountersPlugin;
use local::LocalPlugin;
use types::GameState;
mod breakout;
mod camera;
mod counters;
mod local;
mod types;
mod util;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            fit_canvas_to_parent: true,
            #[cfg(not(feature = "inspector"))]
            cursor_visible: false,
            ..default()
        },
        ..default()
    }))
    .insert_resource(ClearColor(Color::BLACK));

    #[cfg(feature = "inspector")]
    app.add_plugin(WorldInspectorPlugin);

    app.add_loopless_state(GameState::Ingame)
        .add_plugin(BreakoutPlugin)
        .add_plugin(LocalPlugin)
        .add_plugin(CountersPlugin)
        .add_plugin(ScalingCameraPlugin);

    app.run();
}
