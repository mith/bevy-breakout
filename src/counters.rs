use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    breakout::{BreakoutConfig, Lives, Score, FONT_PATH},
    types::GameState,
    util::despawn_with,
};
#[derive(Component)]
pub(crate) struct Counters;

#[derive(Component)]
pub(crate) struct LivesCounter;

#[derive(Component)]
pub(crate) struct ScoreCounter;

pub(crate) fn setup_counters(
    mut commands: Commands,
    config: Res<BreakoutConfig>,
    asset_server: Res<AssetServer>,
    lives: Res<Lives>,
    score: Res<Score>,
) {
    let font = asset_server.load(FONT_PATH);
    let style = TextStyle {
        font,
        font_size: 15.,
        color: Color::WHITE,
    };
    let counter_offset = 15.;

    commands
        .spawn((
            Counters,
            Name::new("Counters"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },

                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(config.court_size[0]), Val::Auto),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                })
                .add_children(|counter_container| {
                    counter_container.spawn((
                        Name::new("Lives counter"),
                        LivesCounter,
                        TextBundle {
                            text: Text::from_sections([
                                TextSection::new("lives:", style.clone()),
                                TextSection::new(lives.0.to_string(), style.clone()),
                            ]),
                            style: Style {
                                margin: UiRect::all(Val::Px(counter_offset)),
                                ..default()
                            },
                            ..default()
                        },
                    ));

                    counter_container.spawn((
                        Name::new("Score counter"),
                        ScoreCounter,
                        TextBundle {
                            text: Text::from_sections([
                                TextSection::new("score:", style.clone()),
                                TextSection::new(score.0.to_string(), style),
                            ]),
                            style: Style {
                                margin: UiRect::all(Val::Px(counter_offset)),
                                ..default()
                            },
                            ..default()
                        },
                    ));
                })
        });
}

pub(crate) fn update_lives_counter(
    lives: Res<Lives>,
    mut lives_counter_query: Query<&mut Text, With<LivesCounter>>,
) {
    if lives.is_changed() {
        let mut lives_counter = lives_counter_query.single_mut();
        lives_counter.sections[1].value = lives.0.to_string();
    }
}

pub(crate) fn update_score_counter(
    score: Res<Score>,
    mut score_counter_query: Query<&mut Text, With<ScoreCounter>>,
) {
    if score.is_changed() {
        let mut score_counter = score_counter_query.single_mut();
        score_counter.sections[1].value = score.0.to_string();
    }
}

pub(crate) struct CountersPlugin;

impl Plugin for CountersPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Ingame, setup_counters)
            .add_exit_system(GameState::Ingame, despawn_with::<Counters>)
            .add_system(update_lives_counter.run_in_state(GameState::Ingame))
            .add_system(update_score_counter.run_in_state(GameState::Ingame));
    }
}
