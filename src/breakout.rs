use std::f32::consts::PI;

use bevy::prelude::*;

use iyes_loopless::prelude::*;

use crate::types::GameState;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub(crate) struct BreakoutConfig {
    pub(crate) court_size: [f32; 2],
    pub(crate) scale: f32,
    pub(crate) paddle_size: [f32; 2],
    pub(crate) paddle_speed: f32,
    pub(crate) angle_multiplier: f32,
    pub(crate) serve_speed: f32,
    pub(crate) num_bricks: [usize; 2],
    pub(crate) bricks_top_offset: f32,
    pub(crate) brick_height: f32,
    pub(crate) brick_padding: f32,
    pub(crate) ball_size: f32,
}

impl Default for BreakoutConfig {
    fn default() -> Self {
        Self {
            court_size: [300., 450.],
            scale: 0.9,
            paddle_size: [40., 10.],
            paddle_speed: 10.,
            angle_multiplier: 0.5,
            serve_speed: 2.,
            num_bricks: [14, 8],
            bricks_top_offset: 50.,
            brick_height: 10.,
            brick_padding: 2.,
            ball_size: 8.,
        }
    }
}

#[derive(SystemLabel)]
pub(crate) enum GameloopStage {
    Input,
    PaddleMovement,
    Serve,
    BallMovement,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BreakoutState {
    Serve,
    Playing,
}

#[derive(Component)]
pub(crate) struct Court;

#[derive(Component)]
pub(crate) struct Paddle;

#[derive(Component)]
pub(crate) struct Ball;

#[derive(Component, Deref, DerefMut, Default, Reflect)]
pub(crate) struct Velocity(pub(crate) Vec2);

#[derive(Component)]
pub(crate) struct Brick;

fn calculate_court_scale(window: &Window, config: &BreakoutConfig) -> f32 {
    let height_ratio = window.height() / config.court_size[1];
    let width_ratio = window.width() / config.court_size[0];
    (1. / height_ratio.min(width_ratio)) * (1. / config.scale)
}

pub(crate) fn setup_camera(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    windows: Res<Windows>,
) {
    let window = windows.primary();

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
    config: Res<BreakoutConfig>,
    windows: Res<Windows>,
) {
    let window = windows.primary();

    for mut projection in &mut query {
        projection.scale = calculate_court_scale(window, &config);
    }
}

pub(crate) fn setup_court(mut commands: Commands, config: Res<BreakoutConfig>) {
    let line_width = 10.;
    commands
        // Spawn court line
        .spawn((
            Name::new("Court line"),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(
                        config.court_size[0] + line_width,
                        config.court_size[1] + line_width,
                    )),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Spawn court
            parent.spawn((
                Court,
                Name::new("Court"),
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::from_array(config.court_size)),
                        ..default()
                    },
                    ..default()
                },
            ));

            // Spawn paddle
            parent.spawn((
                Paddle,
                Name::new("Paddle"),
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        0.,
                        -config.court_size[1] / 2. + 20.,
                        2.,
                    )),
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::from_array(config.paddle_size)),
                        ..default()
                    },
                    ..default()
                },
            ));

            // Spawn bricks
            let brick_size_with_padding = [
                config.court_size[0] / config.num_bricks[0] as f32,
                config.brick_height,
            ];

            let brick_colors = [Color::RED, Color::ORANGE, Color::GREEN, Color::YELLOW];
            for x in 0..config.num_bricks[0] {
                for y in 0..config.num_bricks[1] {
                    parent.spawn((
                        Brick,
                        SpriteBundle {
                            transform: Transform::from_translation(Vec3::new(
                                -config.court_size[0] / 2.
                                    + brick_size_with_padding[0] / 2.
                                    + x as f32 * brick_size_with_padding[0],
                                config.court_size[1] / 2.
                                    - brick_size_with_padding[1] / 2.
                                    - y as f32 * brick_size_with_padding[1]
                                    - config.bricks_top_offset,
                                2.,
                            )),
                            sprite: Sprite {
                                color: brick_colors[y / 2],
                                custom_size: Some(Vec2::new(
                                    brick_size_with_padding[0] - config.brick_padding / 2.,
                                    brick_size_with_padding[1] - config.brick_padding / 2.,
                                )),
                                ..default()
                            },
                            ..default()
                        },
                        Name::new(format!("Brick {}:{}", x, y)),
                    ));
                }
            }

            // Spawn ball
            parent.spawn((
                Ball,
                Name::new("Ball"),
                Velocity::default(),
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        0.,
                        -config.court_size[1] / 2. + 35.,
                        1.,
                    )),
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(config.ball_size, config.ball_size)),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

#[derive(Default)]
pub(crate) struct PaddleInput {
    pub(crate) move_direction: f32,
    pub(crate) serve: bool,
}

#[derive(Resource, Deref, DerefMut)]
pub(crate) struct PaddleInputs(pub(crate) Vec<PaddleInput>);

pub(crate) fn serve(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    mut ball_query: Query<(&mut Transform, &mut Velocity), (With<Ball>, Without<Paddle>)>,
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    inputs: Res<PaddleInputs>,
) {
    let paddle_transform = paddle_query.single_mut();
    let (mut ball_transform, mut ball_velocity) = ball_query.single_mut();

    ball_transform.translation = paddle_transform.translation + Vec3::new(0., 20., 0.);

    if inputs[0].serve {
        ball_velocity.0 = config.serve_speed * Vec2::new(0., 1.).normalize();
        commands.insert_resource(NextState(BreakoutState::Playing));
    }
}

pub(crate) fn paddle_movement(
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    config: Res<BreakoutConfig>,
    inputs: Res<PaddleInputs>,
) {
    let half_court_width = config.court_size[0] / 2.;
    let half_paddle_width = config.paddle_size[0] / 2.;

    for mut transform in &mut paddle_query {
        let input = &inputs[0];

        let translation = &mut transform.translation;
        translation.x += input.move_direction * config.paddle_speed;
        translation.x = translation.x.clamp(
            -half_court_width + half_paddle_width,
            half_court_width - half_paddle_width,
        );
    }
}

pub(crate) fn ball_movement(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    mut ball_query: Query<
        (&mut Transform, &mut Velocity),
        (With<Ball>, Without<Paddle>, Without<Brick>),
    >,
    paddle_query: Query<&Transform, With<Paddle>>,
    brick_query: Query<(Entity, &Transform, &Sprite), With<Brick>>,
) {
    let half_court_width = config.court_size[0] / 2.;
    let half_court_height = config.court_size[1] / 2.;
    let half_ball_size = config.ball_size / 2.;

    for (mut ball_transform, mut ball_velocity) in &mut ball_query {
        let ball_translation = &mut ball_transform.translation;
        ball_translation.x += ball_velocity.x;
        ball_translation.y += ball_velocity.y;

        if ball_translation.x < -half_court_width + half_ball_size {
            // Hit the left side of the court
            ball_translation.x = -half_court_width + half_ball_size;
            ball_velocity.x = -ball_velocity.x;
        } else if ball_translation.x > half_court_width - half_ball_size {
            // Hit the right side of the court
            ball_translation.x = half_court_width - half_ball_size;
            ball_velocity.x = -ball_velocity.x;
        }

        if ball_translation.y < -half_court_height + half_ball_size {
            // Hit the bottom of the court
            // back to serve state
            commands.insert_resource(NextState(BreakoutState::Serve));
        } else if ball_translation.y > half_court_height - half_ball_size {
            // Hit the top of the court
            ball_translation.y = half_court_height - half_ball_size;
            ball_velocity.y = -ball_velocity.y;
        }

        // Check for paddle collision
        for paddle_transform in &paddle_query {
            let paddle_translation = &paddle_transform.translation;
            let paddle_half_width = config.paddle_size[0] / 2.;
            let paddle_half_height = config.paddle_size[1] / 2.;
            let paddle_top = paddle_translation.y + paddle_half_height;
            let paddle_left = paddle_translation.x - paddle_half_width;
            let paddle_right = paddle_translation.x + paddle_half_width;

            let ball_left = ball_translation.x - half_ball_size;
            let ball_right = ball_translation.x + half_ball_size;
            let ball_bottom = ball_translation.y - half_ball_size;

            if ball_bottom < paddle_top && ball_left < paddle_right && ball_right > paddle_left {
                // Hit the paddle
                ball_translation.y = paddle_top;
                ball_velocity.y = -ball_velocity.y;
                // the distance from the center of the paddle, normalized to [-1, 1]
                let distance_from_center = (ball_translation.x - paddle_translation.x)
                    / (paddle_half_width + half_ball_size);
                // the angle of the ball velocity, based on the distance from the center
                let angle = distance_from_center * 0.25 * -PI;
                // rotate the ball velocity by the angle
                ball_velocity.0 =
                    Vec2::new(0., 1.).rotate(Vec2::from_angle(angle)) * ball_velocity.0.length();

                // translate the ball back so it's not inside the paddle
                ball_translation.y = paddle_top + half_ball_size + 0.1;
            }
        }

        // Check for brick collision
        for (brick_entity, brick_transform, brick_sprite) in &brick_query {
            let brick_translation = &brick_transform.translation;
            let brick_half_width = brick_sprite.custom_size.unwrap().x / 2.;
            let brick_half_height = brick_sprite.custom_size.unwrap().y / 2.;
            let brick_top = brick_translation.y + brick_half_height;
            let brick_bottom = brick_translation.y - brick_half_height;
            let brick_left = brick_translation.x - brick_half_width;
            let brick_right = brick_translation.x + brick_half_width;

            let ball_left = ball_translation.x - half_ball_size;
            let ball_right = ball_translation.x + half_ball_size;
            let ball_top = ball_translation.y + half_ball_size;
            let ball_bottom = ball_translation.y - half_ball_size;

            if ball_top > brick_bottom
                && ball_bottom < brick_top
                && ball_left < brick_right
                && ball_right > brick_left
            {
                // Hit the brick
                commands.entity(brick_entity).despawn_recursive();

                let left_diff = (ball_left - brick_right).abs();
                let right_diff = (ball_right - brick_left).abs();
                let top_diff = (ball_top - brick_bottom).abs();
                let bottom_diff = (ball_bottom - brick_top).abs();

                if left_diff.min(right_diff) < top_diff.min(bottom_diff) {
                    // Hit the brick from the left or right
                    ball_velocity.x = -ball_velocity.x;
                } else {
                    // Hit the brick from the top or bottom
                    ball_velocity.y = -ball_velocity.y;
                }
            }
        }
    }
}

pub(crate) struct BreakoutPlugin;

impl Plugin for BreakoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreakoutConfig>()
            .register_type::<BreakoutConfig>()
            .register_type::<Velocity>()
            .insert_resource(PaddleInputs(vec![default()]))
            .add_loopless_state(BreakoutState::Serve)
            .add_enter_system(GameState::Ingame, setup_court)
            .add_system(adjust_camera_scale.run_in_state(GameState::Ingame));
    }
}
