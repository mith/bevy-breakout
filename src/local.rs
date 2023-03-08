use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    breakout::{
        ball_movement, brick_collision, bricks_cleared, finish_game, lives, paddle_movement,
        restart_game, serve, start_serve, BottomCollisionEvent, BreakoutState, BrickCollisionEvent,
        Paddle, PaddleInputs,
    },
    util::cursor_position_in_world,
};

#[derive(Default, Resource)]
struct CursorPosition(pub(crate) Vec3);

fn update_cursor_pos(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    if let Some(cursor_moved) = cursor_moved_events.iter().last() {
        for (camera_transform, camera) in &camera_query {
            *cursor_position = CursorPosition(cursor_position_in_world(
                primary_window.single(),
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

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug)]
struct BallMovement;

fn game_finished(state: Res<State<BreakoutState>>) -> bool {
    state.0 == BreakoutState::Finished
}

pub(crate) struct LocalPlugin;

impl Plugin for LocalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition(Vec3::ZERO))
            .insert_resource(PaddleInputs(vec![default()]));
        app.add_system(update_cursor_pos.before(mouse_input))
            .add_system(mouse_input)
            .add_system(
                paddle_movement
                    .run_if(not(game_finished))
                    .after(mouse_input),
            )
            .add_system(
                serve
                    .in_set(OnUpdate(BreakoutState::Serve))
                    .run_if(serve_button_pressed)
                    .after(paddle_movement),
            )
            .add_system(
                restart_game
                    .in_set(OnUpdate(BreakoutState::Finished))
                    .run_if(left_mouse_button_just_pressed),
            )
            .add_system(
                start_serve
                    .in_set(OnUpdate(BreakoutState::Start))
                    .run_if(not(left_mouse_button_pressed)),
            )
            .add_system(
                finish_game
                    .in_set(OnUpdate(BreakoutState::Playing))
                    .run_if(bricks_cleared),
            );

        app.insert_resource(FixedTime::new(Duration::from_millis(1)))
            .add_systems(
                (
                    ball_movement.in_set(OnUpdate(BreakoutState::Playing)),
                    lives.run_if(on_event::<BottomCollisionEvent>()),
                    brick_collision.run_if(on_event::<BrickCollisionEvent>()),
                    apply_system_buffers,
                )
                    .chain()
                    .in_schedule(CoreSchedule::FixedUpdate),
            );
    }
}
