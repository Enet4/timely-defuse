use crate::events::{
    BombDisarmedEvent, DynamiteDefusedEvent, ExplodedEvent, ExplosiveKind, GuyHurtEvent,
};
use bevy::{prelude::*, time::Stopwatch};

#[derive(Debug, Default, Resource)]
pub struct GameScores {
    pub dynamites_disarmed: u32,
    pub bombs_disarmed: u32,
    pub score: i32,
}

impl GameScores {
    #[inline]
    pub fn add_dynamite_defused(&mut self) {
        self.dynamites_disarmed += 1;
        self.score += 1;
    }

    #[inline]
    pub fn add_bomb_disarmed(&mut self) {
        self.bombs_disarmed += 1;
        self.score += 10;
    }
}

/// system: on bomb disarmed, update score
pub fn on_disarm_bomb(
    mut scores: ResMut<GameScores>,
    mut event_reader: EventReader<BombDisarmedEvent>,
    mut query: Query<&mut Text, With<GameScoreUi>>,
) {
    for _ in event_reader.iter() {
        scores.add_bomb_disarmed();
        update_score(&scores, &mut query);
    }
}

/// system: if bomb explodes, lose points
pub fn on_bomb_explode(
    mut scores: ResMut<GameScores>,
    mut event_reader: EventReader<ExplodedEvent>,
    mut query: Query<&mut Text, With<GameScoreUi>>,
) {
    for ev in event_reader.iter() {
        if ev.kind == ExplosiveKind::Bomb {
            scores.score -= 2;
            update_score(&scores, &mut query);
        }
    }
}

pub fn on_guy_hurt(
    mut scores: ResMut<GameScores>,
    mut event_reader: EventReader<GuyHurtEvent>,
    mut query: Query<&mut Text, With<GameScoreUi>>,
) {
    for ev in event_reader.iter() {
        scores.score -= match ev.from {
            ExplosiveKind::Bomb => 3,
            ExplosiveKind::Dynamite => 1,
        };
        update_score(&scores, &mut query);
    }
}

/// system: on dynamite picked up, update score
pub fn on_pickup_dynamite(
    mut scores: ResMut<GameScores>,
    mut event_reader: EventReader<DynamiteDefusedEvent>,
    mut query: Query<&mut Text, With<GameScoreUi>>,
) {
    for _ in event_reader.iter() {
        scores.add_dynamite_defused();
        update_score(&scores, &mut query);
    }
}

fn update_score(scores: &GameScores, query: &mut Query<&mut Text, With<GameScoreUi>>) {
    for mut text in query {
        text.sections[0].value = scores.score.to_string();
    }
}

#[derive(Debug, Default, Component)]
pub struct GameScoreUi;

#[derive(Default, Bundle)]
pub struct GameScoreUiBundle {
    pub game_score_ui: GameScoreUi,
    pub text_bundle: TextBundle,
}

pub fn spawn_game_score_ui(commands: &mut Commands, font: Handle<Font>) -> Entity {
    let text_bundle = TextBundle::from_section(
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        "0",
        TextStyle {
            font,
            font_size: 32.0,
            color: Color::WHITE,
        },
    )
    // Set the alignment of the Text
    .with_text_alignment(TextAlignment::CENTER_LEFT)
    // Set the style of the TextBundle itself.
    .with_style(Style {
        align_self: AlignSelf::FlexEnd,
        position_type: PositionType::Absolute,
        position: UiRect {
            top: Val::Px(6.0),
            right: Val::Px(80.0),
            ..default()
        },
        ..default()
    });

    commands
        .spawn(GameScoreUiBundle {
            game_score_ui: GameScoreUi,
            text_bundle,
        })
        .id()
}

/// marker component for the whole stats UI

#[derive(Default, Component)]
pub struct Stats {
    pub stopwatch: Stopwatch,
}

#[derive(Default, Bundle)]
pub struct StatsBundle {
    pub stats: Stats,
    pub node: NodeBundle,
}

/// marker component for the number of bombs disarmed
#[derive(Default, Component)]
pub struct BombsScoreUi;

/// marker component for the number of dynamites defused
#[derive(Default, Component)]
pub struct DynamitesScoreUi;

/// marker component for the total score
#[derive(Default, Component)]
pub struct TotalScoreUi;

pub fn spawn_stats(commands: &mut ChildBuilder, font: Handle<Font>) -> Entity {
    commands
        .spawn(StatsBundle {
            node: NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect {
                        top: Val::Px(48.),
                        bottom: Val::Px(96.),
                        left: Val::Px(30.),
                        right: Val::Px(30.),
                    },
                    size: Size {
                        width: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // column 1: labels
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        margin: UiRect {
                            right: Val::Px(12.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                })
                .with_children(|p| {
                    // 3 rows
                    p.spawn(TextBundle {
                        text: Text::from_section(
                            "Bombs disarmed:",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });
                    p.spawn(TextBundle {
                        text: Text::from_section(
                            "Dynamites defused:",
                            TextStyle {
                                font: font.clone(),
                                font_size: 24.,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });
                    p.spawn(TextBundle {
                        text: Text::from_section(
                            "Total score:",
                            TextStyle {
                                font: font.clone(),
                                font_size: 32.,
                                color: Color::WHITE,
                            },
                        ),
                        style: Style {
                            margin: UiRect {
                                top: Val::Px(12.),
                                ..default()
                            },
                            ..default()
                        },
                        ..default()
                    });
                });
            // column 2: the actual scores
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        margin: UiRect {
                            right: Val::Px(12.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                })
                .with_children(|p| {
                    // 3 rows
                    p.spawn((
                        TextBundle {
                            text: Text::from_section(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 24.,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        BombsScoreUi,
                    ));
                    p.spawn((
                        TextBundle {
                            text: Text::from_section(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 24.,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        },
                        DynamitesScoreUi,
                    ));
                    p.spawn((
                        TextBundle {
                            text: Text::from_section(
                                " ",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 32.,
                                    color: Color::WHITE,
                                },
                            ),
                            style: Style {
                                margin: UiRect {
                                    top: Val::Px(12.),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        },
                        TotalScoreUi,
                    ));
                });
        })
        .id()
}

/// system: update the final stats UI
pub fn update_stats(
    time: Res<Time>,
    scores: Res<GameScores>,
    mut query_stats: Query<&mut Stats>,
    mut query: ParamSet<(
        Query<&mut Text, With<BombsScoreUi>>,
        Query<&mut Text, With<DynamitesScoreUi>>,
        Query<&mut Text, With<TotalScoreUi>>,
    )>,
) {
    // update stopwatch and fetch time elapsed
    let Ok(mut stats) = query_stats.get_single_mut() else {
        return;
    };

    stats.stopwatch.tick(time.delta());
    let elapsed = stats.stopwatch.elapsed().as_secs_f32();

    // update each score value with an animation
    for mut text in &mut query.p0() {
        let base_time_to_appear = 0.;
        if elapsed < base_time_to_appear {
            break;
        }

        // value interpolation to the target score
        let score = scores.bombs_disarmed;
        let interval = 0.6;
        let value =
            (score as f32 * (((elapsed - base_time_to_appear) / interval).min(1.))).round() as i32;
        text.sections[0].value = value.to_string();
    }

    for mut text in &mut query.p1() {
        let base_time_to_appear = 0.65;
        if elapsed < base_time_to_appear {
            break;
        }

        // value interpolation to the target score
        let score = scores.dynamites_disarmed;
        let interval = 0.6;
        let value =
            (score as f32 * (((elapsed - base_time_to_appear) / interval).min(1.))).round() as i32;
        text.sections[0].value = value.to_string();
    }

    for mut text in &mut query.p2() {
        let base_time_to_appear = 1.3;
        if elapsed < base_time_to_appear {
            break;
        }

        // value interpolation to the target score
        let score = scores.score;
        let interval = 0.8;
        let value =
            (score as f32 * (((elapsed - base_time_to_appear) / interval).min(1.))).round() as i32;
        text.sections[0].value = value.to_string();
    }
}
