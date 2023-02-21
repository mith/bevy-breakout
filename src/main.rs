use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use breakout::setup_camera;
use iyes_loopless::prelude::*;
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
            ..default()
        },
        ..default()
    }))
    // .add_plugin(WorldInspectorPlugin)
    .insert_resource(ClearColor(Color::BLACK))
    .add_startup_system(setup_camera)
    .add_loopless_state(GameState::Ingame)
    .add_plugin(breakout::BreakoutPlugin)
    .add_plugin(local::LocalPlugin);

    app.run();
}
