use std::time::Duration;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    breakout::{
        ball_movement, brick_collision, bricks_cleared, finish_game, lives, paddle_movement,
        restart_game, serve, start_serve, BottomCollisionEvent, BreakoutState, BrickCollisionEvent,
        GameloopStage, Paddle, PaddleInputs,
    },
    types::GameState,
    util::cursor_position_in_world,
};

#[derive(Default, Resource)]
struct CursorPosition(pub(crate) Vec3);

fn update_cursor_pos(
    windows: Res<Windows>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    if let Some(cursor_moved) = cursor_moved_events.iter().last() {
        for (camera_transform, camera) in &camera_query {
            *cursor_position = CursorPosition(cursor_position_in_world(
                &windows,
                cursor_moved.position,
                camera_transform,
                camera,
            ))
        }
    }
}

fn mouse_input(
    mouse_button_input: Res<Input<MouseButton>>,
    mut inputs: ResMut<PaddleInputs>,
    paddle_query: Query<&GlobalTransform, With<Paddle>>,
    cursor_position: Res<CursorPosition>,
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
            (cursor_position.0.x - paddle_transform.translation().x) * projection.scale;
        let max_move = cursor_paddle_diff.abs().min(4.);
        inputs[0].move_direction = (cursor_paddle_diff * 0.1).clamp(-max_move, max_move);
    }
}

fn left_mouse_button_just_pressed(mouse_button_input: Res<Input<MouseButton>>) -> bool {
    mouse_button_input.just_pressed(MouseButton::Left)
}

fn left_mouse_button_pressed(mouse_button_input: Res<Input<MouseButton>>) -> bool {
    mouse_button_input.pressed(MouseButton::Left)
}

fn serve_button_pressed(paddle_inputs: Res<PaddleInputs>) -> bool {
    paddle_inputs[0].serve
}

pub(crate) struct LocalPlugin;

impl Plugin for LocalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition(Vec3::ZERO))
            .insert_resource(PaddleInputs(vec![default()]));
        app.add_system(
            update_cursor_pos
                .run_in_state(GameState::Ingame)
                .before(GameloopStage::Input),
        )
        .add_system(
            mouse_input
                .run_in_state(GameState::Ingame)
                .label(GameloopStage::Input),
        )
        .add_system(
            paddle_movement
                .run_in_state(GameState::Ingame)
                .run_not_in_state(BreakoutState::Finished)
                .after(GameloopStage::Input)
                .label(GameloopStage::PaddleMovement),
        )
        .add_system(
            serve
                .run_in_state(GameState::Ingame)
                .run_in_state(BreakoutState::Serve)
                .run_if(serve_button_pressed)
                .after(GameloopStage::PaddleMovement)
                .label(GameloopStage::Serve),
        )
        .add_system(
            restart_game
                .run_in_state(GameState::Ingame)
                .run_in_state(BreakoutState::Finished)
                .run_if(left_mouse_button_just_pressed),
        )
        .add_system(
            start_serve
                .run_in_state(GameState::Ingame)
                .run_in_state(BreakoutState::Start)
                .run_if_not(left_mouse_button_pressed),
        )
        .add_system(
            finish_game
                .run_in_state(GameState::Ingame)
                .run_in_state(BreakoutState::Playing)
                .run_if(bricks_cleared),
        );

        let timestep_label = &"fixed_timestep";
        app.add_fixed_timestep(Duration::from_millis(1), timestep_label)
            .add_fixed_timestep_system(
                timestep_label,
                0,
                ball_movement
                    .run_in_state(GameState::Ingame)
                    .run_in_state(BreakoutState::Playing)
                    .label(GameloopStage::BallMovement),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                lives
                    .run_in_state(GameState::Ingame)
                    .run_in_state(BreakoutState::Playing)
                    .run_on_event::<BottomCollisionEvent>()
                    .after(GameloopStage::BallMovement)
                    .label(GameloopStage::Scoring),
            )
            .add_fixed_timestep_system(
                timestep_label,
                0,
                brick_collision
                    .run_in_state(GameState::Ingame)
                    .run_in_state(BreakoutState::Playing)
                    .run_on_event::<BrickCollisionEvent>()
                    .after(GameloopStage::BallMovement)
                    .label(GameloopStage::Scoring),
            );
    }
}
