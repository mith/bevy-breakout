use std::f32::consts::PI;

use bevy::prelude::*;

use iyes_loopless::prelude::*;

use crate::{types::GameState, util::despawn_with};

pub(crate) const FONT_PATH: &str = "fonts/PublicPixel-z84yD.ttf";

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub(crate) struct BreakoutConfig {
    pub(crate) court_size: [f32; 2],
    pub(crate) scale: f32,
    pub(crate) paddle_size: [f32; 2],
    pub(crate) paddle_speed: f32,
    pub(crate) paddle_offset: f32,
    pub(crate) angle_multiplier: f32,
    pub(crate) serve_speed: f32,
    pub(crate) serve_offset: f32,
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
            paddle_speed: 1000.,
            paddle_offset: 20.,
            angle_multiplier: 0.5,
            serve_speed: 500.,
            serve_offset: 20.,
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
    Scoring,
}

#[derive(Resource)]
pub(crate) enum GameResult {
    Victory,
    GameOver,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BreakoutState {
    Start,
    Serve,
    Playing,
    Finished,
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
pub(crate) struct Brick {
    pub(crate) points: u32,
}
pub(crate) fn setup_court(mut commands: Commands, config: Res<BreakoutConfig>) {
    let line_width = 5.;
    commands
        // Spawn court
        .spawn((
            Court,
            Name::new("Court"),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0., -30., 1.)),
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(config.court_size[0], config.court_size[1])),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // Spawn courtline
            parent.spawn((
                Name::new("Court line"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::new(
                            config.court_size[0] + line_width * 2.,
                            config.court_size[1] + line_width,
                        )),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0., line_width * 0.5, -1.)),
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
                        -config.court_size[1] / 2. + config.paddle_offset,
                        1.,
                    )),
                    sprite: Sprite {
                        color: Color::WHITE,
                        custom_size: Some(Vec2::from_array(config.paddle_size)),
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

fn spawn_ball(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    paddle_query: Query<Entity, With<Paddle>>,
) {
    let paddle_entity = paddle_query.single();
    commands.entity(paddle_entity).add_children(|parent| {
        // Spawn ball
        parent.spawn((
            Ball,
            Name::new("Ball"),
            Velocity::default(),
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0.0, config.serve_offset, 0.)),
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

fn spawn_bricks(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    court_query: Query<Entity, With<Court>>,
) {
    // Spawn bricks
    let brick_size_with_padding = [
        config.court_size[0] / config.num_bricks[0] as f32,
        config.brick_height,
    ];

    let court = court_query.single();
    commands.entity(court).add_children(|parent| {
        let brick_colors = [Color::RED, Color::ORANGE, Color::GREEN, Color::YELLOW];
        for x in 0..config.num_bricks[0] {
            for y in 0..config.num_bricks[1] {
                {
                    parent.spawn((
                        Brick {
                            points: (config.num_bricks[1] - y) as u32,
                        },
                        Name::new(format!("Brick {}:{}", x, y)),
                        SpriteBundle {
                            transform: Transform::from_translation(Vec3::new(
                                -config.court_size[0] / 2.
                                    + brick_size_with_padding[0] / 2.
                                    + x as f32 * brick_size_with_padding[0],
                                config.court_size[1] / 2.
                                    - brick_size_with_padding[1] / 2.
                                    - y as f32 * brick_size_with_padding[1]
                                    - config.bricks_top_offset,
                                1.,
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
                    ));
                }
            }
        }
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
    fixed_timestep: Res<FixedTimesteps>,
    config: Res<BreakoutConfig>,
    mut ball_query: Query<(Entity, &mut Transform, &mut Velocity), (With<Ball>, Without<Paddle>)>,
    court_query: Query<Entity, With<Court>>,
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
) {
    let (ball_entity, mut ball_transform, mut ball_velocity) = ball_query.single_mut();
    let paddle_translation = paddle_query.single_mut().translation;
    let court_entity = court_query.single();
    commands.entity(court_entity).add_child(ball_entity);
    let current_timestep = fixed_timestep.get("fixed_timestep").unwrap();
    ball_velocity.0 = config.serve_speed
        * current_timestep.timestep().as_secs_f32()
        * Vec2::new(0., 1.).normalize();
    ball_transform.translation = Vec3::new(
        paddle_translation.x,
        paddle_translation.y + config.serve_offset,
        1.,
    );
    commands.insert_resource(NextState(BreakoutState::Playing));
}

pub(crate) fn paddle_movement(
    time: Res<Time>,
    mut paddle_query: Query<&mut Transform, With<Paddle>>,
    config: Res<BreakoutConfig>,
    inputs: Res<PaddleInputs>,
) {
    let half_court_width = config.court_size[0] / 2.;
    let half_paddle_width = config.paddle_size[0] / 2.;

    for mut transform in &mut paddle_query {
        let input = &inputs[0];

        let translation = &mut transform.translation;
        translation.x += input.move_direction * config.paddle_speed * time.delta_seconds();
        translation.x = translation.x.clamp(
            -half_court_width + half_paddle_width,
            half_court_width - half_paddle_width,
        );
    }
}

pub(crate) struct BrickCollisionEvent {
    brick_entity: Entity,
}

pub(crate) struct BottomCollisionEvent;

pub(crate) fn ball_movement(
    court_query: Query<&Sprite, With<Court>>,
    mut ball_query: Query<
        (&mut Transform, &mut Velocity, &Sprite),
        (With<Ball>, Without<Paddle>, Without<Brick>),
    >,
    paddle_query: Query<(&Transform, &Sprite), With<Paddle>>,
    brick_query: Query<(Entity, &Transform, &Sprite), With<Brick>>,
    mut brick_collision_events: EventWriter<BrickCollisionEvent>,
    mut bottom_collision_events: EventWriter<BottomCollisionEvent>,
) {
    let court_sprite_size = court_query.single().custom_size.unwrap();
    let half_court_width = court_sprite_size.x / 2.;
    let half_court_height = court_sprite_size[1] / 2.;

    for (mut ball_transform, mut ball_velocity, ball_sprite) in &mut ball_query {
        let half_ball_size = ball_sprite.custom_size.unwrap().x / 2.;
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
            // Send a BottomCollisionEvent and reset the ball
            bottom_collision_events.send(BottomCollisionEvent);
        } else if ball_translation.y > half_court_height - half_ball_size {
            // Hit the top of the court
            ball_translation.y = half_court_height - half_ball_size;
            ball_velocity.y = -ball_velocity.y;
        }

        // Check for paddle collision
        for (paddle_transform, paddle_sprite) in &paddle_query {
            let paddle_sprite_size = paddle_sprite.custom_size.unwrap();
            let paddle_translation = &paddle_transform.translation;
            let paddle_half_width = paddle_sprite_size.x / 2.;
            let paddle_half_height = paddle_sprite_size.y / 2.;
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

        let mut new_velocity = ball_velocity.0;

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
                brick_collision_events.send(BrickCollisionEvent { brick_entity });

                let left_diff = (ball_left - brick_right).abs();
                let right_diff = (ball_right - brick_left).abs();
                let top_diff = (ball_top - brick_bottom).abs();
                let bottom_diff = (ball_bottom - brick_top).abs();

                if left_diff.min(right_diff) < top_diff.min(bottom_diff) {
                    // Hit the brick from the left or right
                    new_velocity.x = -ball_velocity.x;
                } else {
                    // Hit the brick from the top or bottom
                    new_velocity.y = -ball_velocity.y;
                }
            }
        }
        ball_velocity.0 = new_velocity;
    }
}

#[derive(Resource)]
pub(crate) struct Lives(pub u32);

impl Default for Lives {
    fn default() -> Self {
        Self(3)
    }
}

pub(crate) fn lives(
    mut commands: Commands,
    mut lives: ResMut<Lives>,
    mut bottom_collision_events: EventReader<BottomCollisionEvent>,
    mut ball_query: Query<(&mut Transform, &mut Velocity), With<Ball>>,
) {
    for _ in bottom_collision_events.iter() {
        lives.0 = lives.0.saturating_sub(1);

        if lives.0 == 0 {
            commands.insert_resource(GameResult::GameOver);
            commands.insert_resource(NextState(BreakoutState::Finished));
        } else {
            for (mut ball_transform, mut ball_velocity) in &mut ball_query.iter_mut() {
                ball_transform.translation = Vec3::new(0., 0., 0.);
                ball_velocity.0 = Vec2::new(0., 0.);
            }
            commands.insert_resource(NextState(BreakoutState::Serve));
        }
    }
}

#[derive(Component)]
struct FinishedText;

pub(crate) fn show_game_finished(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_result: Res<GameResult>,
    score: Res<Score>,
) {
    commands
        .spawn((
            FinishedText,
            Name::new("Finished text"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Game result text"),
                TextBundle::from_section(
                    match *game_result {
                        GameResult::Victory => "Victory",
                        GameResult::GameOver => "Game over",
                    },
                    TextStyle {
                        font: asset_server.load(FONT_PATH),
                        font_size: 30.,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(15.)),
                    ..default()
                }),
            ));

            parent.spawn((
                Name::new("Final score"),
                TextBundle::from_section(
                    format!("Score: {}", score.0),
                    TextStyle {
                        font: asset_server.load(FONT_PATH),
                        font_size: 20.,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(15.)),
                    ..default()
                }),
            ));

            parent.spawn((
                Name::new("Restart prompt"),
                TextBundle::from_section(
                    "click to restart",
                    TextStyle {
                        font: asset_server.load(FONT_PATH),
                        font_size: 10.,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::vertical(Val::Px(15.)),
                    ..default()
                }),
            ));
        });
}

pub(crate) fn reset_lives(mut lives: ResMut<Lives>) {
    *lives = default();
}

#[derive(Resource, Default)]
pub(crate) struct Score(pub u32);

pub(crate) fn reset_score(mut score: ResMut<Score>) {
    *score = default();
}

pub(crate) fn brick_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut brick_collision_events: EventReader<BrickCollisionEvent>,
    brick_query: Query<&Brick>,
) {
    for BrickCollisionEvent { brick_entity } in brick_collision_events.iter() {
        let brick_points = brick_query.get(*brick_entity).unwrap().points;
        score.0 += brick_points;

        commands.entity(*brick_entity).despawn_recursive();
    }
}

pub(crate) fn bricks_cleared(brick_query: Query<&Brick>) -> bool {
    brick_query.is_empty()
}

pub(crate) fn finish_game(mut commands: Commands) {
    commands.insert_resource(GameResult::Victory);
    commands.insert_resource(NextState(BreakoutState::Finished));
}

pub(crate) fn restart_game(mut commands: Commands) {
    commands.insert_resource(NextState(BreakoutState::Start));
}

pub(crate) fn start_serve(mut commands: Commands) {
    commands.insert_resource(NextState(BreakoutState::Serve));
}

pub(crate) fn clear_game_result(mut commands: Commands) {
    commands.remove_resource::<GameResult>();
}

pub(crate) struct BreakoutPlugin;

impl Plugin for BreakoutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BreakoutConfig>()
            .register_type::<BreakoutConfig>()
            .register_type::<Velocity>()
            .init_resource::<Lives>()
            .init_resource::<Score>()
            .add_event::<BrickCollisionEvent>()
            .add_event::<BottomCollisionEvent>()
            .add_loopless_state(BreakoutState::Start)
            .add_enter_system(GameState::Ingame, setup_court)
            .add_exit_system(GameState::Ingame, despawn_with::<Court>)
            .add_enter_system(BreakoutState::Start, spawn_bricks)
            .add_exit_system(BreakoutState::Playing, despawn_with::<Ball>)
            .add_enter_system(BreakoutState::Serve, spawn_ball)
            .add_enter_system(BreakoutState::Finished, show_game_finished)
            .add_exit_system(BreakoutState::Finished, despawn_with::<Brick>)
            .add_exit_system(BreakoutState::Finished, despawn_with::<FinishedText>)
            .add_exit_system(BreakoutState::Finished, reset_lives)
            .add_exit_system(BreakoutState::Finished, reset_score)
            .add_exit_system(BreakoutState::Finished, clear_game_result);
    }
}
