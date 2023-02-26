use bevy::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_loopless::prelude::*;

use breakout::setup_camera;
use types::GameState;

mod breakout;
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

    app.add_startup_system(setup_camera)
        .add_loopless_state(GameState::Ingame)
        .add_plugin(breakout::BreakoutPlugin)
        .add_plugin(local::LocalPlugin);

    app.run();
}
