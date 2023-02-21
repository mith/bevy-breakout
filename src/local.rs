use std::time::Duration;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    breakout::{
        ball_movement, paddle_movement, serve, BreakoutState, GameloopStage, Paddle, PaddleInputs,
    },
    types::GameState,
    util::cursor_pos_in_world,
};

#[derive(Default, Resource)]
struct CursorPos(pub(crate) Vec3);

fn update_cursor_pos(
    windows: Res<Windows>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    if let Some(cursor_moved) = cursor_moved_events.iter().last() {
        for (cam_t, cam) in &camera_query {
            *cursor_pos = CursorPos(cursor_pos_in_world(
                &windows,
                cursor_moved.position,
                cam_t,
                cam,
            ))
        }
    }
}

fn mouse_input(
    mouse_button_input: Res<Input<MouseButton>>,
    mut inputs: ResMut<PaddleInputs>,
    paddle_query: Query<&GlobalTransform, With<Paddle>>,
    cursor_pos: Res<CursorPos>,
    projection_query: Query<&OrthographicProjection>,
) {
    inputs[0].move_direction = 0.;
    inputs[0].serve = false;

    if mouse_button_input.pressed(MouseButton::Left) {
        inputs[0].serve = true;
    }

    if let Ok(paddle_transform) = paddle_query.get_single() {
        let projection = projection_query.get_single().unwrap();
        let cursor_paddle_diff =
            (cursor_pos.0.x - paddle_transform.translation().x) * projection.scale;
        let max_move = cursor_paddle_diff.abs().min(4.);
        inputs[0].move_direction = (cursor_paddle_diff * 0.1).clamp(-max_move, max_move);
    }
}

pub(crate) struct LocalPlugin;

impl Plugin for LocalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPos(Vec3::ZERO));

        let timestep_label = &"fixed_timestep";
        app.add_fixed_timestep(Duration::from_millis(4), timestep_label)
            .add_fixed_timestep_system(
                timestep_label,
                0,
                update_cursor_pos
                    .run_in_state(GameState::Ingame)
                    .before(GameloopStage::Input),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                mouse_input
                    .run_in_state(GameState::Ingame)
                    .label(GameloopStage::Input),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                paddle_movement
                    .run_in_state(GameState::Ingame)
                    .after(GameloopStage::Input)
                    .label(GameloopStage::PaddleMovement),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                serve
                    .run_in_state(GameState::Ingame)
                    .run_in_state(BreakoutState::Serve)
                    .after(GameloopStage::PaddleMovement)
                    .label(GameloopStage::Serve),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                ball_movement
                    .run_in_state(GameState::Ingame)
                    .run_in_state(BreakoutState::Playing)
                    .after(GameloopStage::PaddleMovement)
                    .label(GameloopStage::BallMovement),
            );
    }
}
