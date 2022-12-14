//! Module containing all wave descriptors
//!

use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_ecs_tilemap::{
    prelude::TilemapId,
    tiles::{TilePos, TileTextureIndex},
};

use crate::{
    background::Background,
    events::{
        BombThrownEvent, CoffeeThrownEvent, DynamiteThrownEvent, NextWaveEvent, WaveFinishedEvent,
    },
    guy::{GuyDestination, GuyState},
    helper::ScheduledEvent,
    ingame::{Wave, WaveUi},
    menu::NORMAL_BUTTON,
    movement::{SpatialPosition, SpatialVelocity},
    scores::{spawn_stats, GameScores, RemarkUi},
    spawner::{PendingThrow, RandomEventProducer, Spawner, SpawnerCooldown},
    DefaultFont,
};

pub static WAVE_DESCRIPTORS: &[fn(Commands) -> ()] = &[
    spawn_wave_0,
    spawn_wave_1,
    spawn_wave_2,
    spawn_wave_3,
    spawn_wave_4,
    spawn_wave_5,
    spawn_wave_6,
];

// wave 0 to serve as tutorial
pub fn spawn_wave_0(mut commands: Commands) {
    commands.spawn((
        ScheduledEvent::new(DynamiteThrownEvent, Duration::from_millis(750)),
        PendingThrow,
    ));
    commands.spawn((
        ScheduledEvent::new(BombThrownEvent, Duration::from_millis(3_400)),
        PendingThrow,
    ));
}

pub fn spawn_wave_1(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(10),
        RandomEventProducer::new(0.42, DynamiteThrownEvent),
    ));
    commands.spawn((
        ScheduledEvent::new(BombThrownEvent, Duration::from_millis(16_000)),
        PendingThrow,
    ));
}

pub fn spawn_wave_2(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(18),
        RandomEventProducer::new(0.45, DynamiteThrownEvent),
    ));
    commands.spawn((
        ScheduledEvent::new(BombThrownEvent, Duration::from_millis(12_000)),
        PendingThrow,
    ));
    commands.spawn((
        ScheduledEvent::new(BombThrownEvent, Duration::from_millis(24_000)),
        PendingThrow,
    ));
    commands.spawn((
        ScheduledEvent::new(CoffeeThrownEvent, Duration::from_millis(8_500)),
        PendingThrow,
    ));
}

pub fn spawn_wave_3(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(22),
        RandomEventProducer::new(0.5, DynamiteThrownEvent),
    ));
    commands.spawn((
        Spawner::new_essential(3),
        RandomEventProducer::new(0.08, BombThrownEvent),
    ));
    commands.spawn((
        Spawner::new_nonessential(2),
        RandomEventProducer::new(0.055, CoffeeThrownEvent),
    ));
}

pub fn spawn_wave_4(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(12),
        RandomEventProducer::new(0.4, DynamiteThrownEvent),
    ));
    commands.spawn((
        Spawner::new_essential(6),
        RandomEventProducer::new(0.09, BombThrownEvent),
    ));
    commands.spawn((
        Spawner::new_nonessential(2),
        RandomEventProducer::new(0.06, CoffeeThrownEvent),
    ));
}

pub fn spawn_wave_5(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(26),
        RandomEventProducer::new(0.3, DynamiteThrownEvent),
    ));
    commands.spawn((
        Spawner::new_essential(16),
        RandomEventProducer::new(0.12, BombThrownEvent),
    ));
    commands.spawn((
        Spawner::new_nonessential(4),
        RandomEventProducer::new(0.06, CoffeeThrownEvent),
    ));
}

pub fn spawn_wave_6(mut commands: Commands) {
    commands.spawn((
        Spawner::new_essential(96),
        RandomEventProducer::new(0.64, DynamiteThrownEvent),
    ));
    commands.spawn((
        Spawner::new_essential(24),
        RandomEventProducer::new(0.14, BombThrownEvent),
    ));
    commands.spawn((
        Spawner::new_nonessential(8),
        RandomEventProducer::new(0.075, CoffeeThrownEvent),
    ));
}

/// Marker component for entities representing the end of the wave.
#[derive(Component)]
pub struct WaveFinished;

/// An update tick counter for the wave detection check.
/// A silly cheap trick to reduce the amount of checks.
#[derive(Debug, Default)]
pub struct TickCounter(u32);

impl TickCounter {
    const TICKS_TO_TRIGGER: u32 = 16;

    fn tick(&mut self) -> bool {
        self.0 = (self.0 + 1) % Self::TICKS_TO_TRIGGER;
        self.0 == Self::TICKS_TO_TRIGGER - 1
    }
}

/// system: grab existing spawners, see if they're done
pub fn detect_wave_finish(
    mut commands: Commands,
    query_wave_finished: Query<(), With<WaveFinished>>,
    // find all scheduled throws
    query_throws: Query<(), With<PendingThrow>>,
    // find all active entities in the world save for guy
    query_active_entities: Query<(), (With<SpatialPosition>, Without<GuyState>)>,
    // find all spawners
    mut query_spawners: Query<(Entity, &Spawner, Option<&SpawnerCooldown>)>,
    mut query_wave_ui: Query<&mut Text, With<WaveUi>>,
    mut event_writer: EventWriter<WaveFinishedEvent>,
    mut ticks: Local<TickCounter>,
) {
    if !ticks.tick() {
        return;
    }

    // no pending throws
    let c1 = query_throws.is_empty();
    if !c1 {
        return;
    }

    // no active entities but guy
    let c2 = query_active_entities.is_empty();
    if !c2 {
        return;
    }

    // spawners are down to 0 and not cooling down
    let c3 = query_spawners
        .iter_mut()
        .all(|(_, spawner, cooldown)| cooldown.is_none() && spawner.remaining == 0);
    if !c3 {
        return;
    }

    // if wave is already finished, we don't want to repeat this
    let c4 = query_wave_finished.is_empty();
    if !c4 {
        return;
    }

    info!("Wave finished");

    if let Ok(mut wave_ui_text) = query_wave_ui.get_single_mut() {
        wave_ui_text.sections[0].value += "\nCOMPLETE";
    }

    // emit end of wave event
    event_writer.send(WaveFinishedEvent);

    // remove all spawners
    for (e, _, _) in &mut query_spawners {
        commands.entity(e).despawn();
    }

    // schedule for next wave
    commands.spawn((
        ScheduledEvent::new(NextWaveEvent, Duration::from_secs(2)),
        WaveFinished,
    ));
}

/// system: on next wave event
pub fn on_next_wave(
    mut commands: Commands,
    mut wave: ResMut<Wave>,
    scores: Res<GameScores>,
    font: Res<DefaultFont>,
    mut event_reader: EventReader<NextWaveEvent>,
    mut query_wave_ui: Query<&mut Text, With<WaveUi>>,
    query_guy: Query<(Entity, &mut GuyState, &mut SpatialVelocity)>,
    query_wave_finished: Query<Entity, With<WaveFinished>>,
) {
    if let Some(_) = event_reader.iter().next() {
        wave.0 += 1;

        if let Some(wave_fn) = WAVE_DESCRIPTORS.get(wave.0 as usize) {
            info!("Next wave: {}", wave.0);

            if let Ok(mut wave_ui_text) = query_wave_ui.get_single_mut() {
                wave_ui_text.sections[0].value = if wave.0 as usize == WAVE_DESCRIPTORS.len() - 1 {
                    "FINAL WAVE".to_string()
                } else {
                    format!("WAVE {}", wave.0)
                };
            }

            for e in &query_wave_finished {
                commands.entity(e).despawn();
            }

            (wave_fn)(commands);
        } else {
            // The end!
            info!("Game over");
            spawn_game_over(&mut commands, scores, font, query_guy);
        }
    }
}

fn spawn_game_over(
    commands: &mut Commands,
    scores: Res<GameScores>,
    font: Res<DefaultFont>,
    mut query_guy: Query<(Entity, &mut GuyState, &mut SpatialVelocity)>,
) {
    if let Ok((guy_entity, mut guy_state, mut guy_velocity)) = query_guy.get_single_mut() {
        match scores.score {
            -999_999..=255 => {
                *guy_state = GuyState::Loser;
            }
            256..=999_999 => {
                *guy_state = GuyState::Victorious;
            }
            _ => {
                /* no-op */
                warn!("... What!?");
            }
        }
        // stop moving
        guy_velocity.0 = Vec3::new(0., 0., 0.);
        commands.entity(guy_entity).remove::<GuyDestination>();
    }

    // spawn container for all stats ui

    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Px(180.),
                    bottom: Val::Auto,
                },
                ..default()
            },
            ..default()
        })
        .with_children(|mut parent| {
            // spawn game stats thingy
            spawn_stats(&mut parent, font.0.clone());

            // spawn remark
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 24.,
                            color: Color::rgb(1., 1., 0.5),
                        },
                    ),
                    style: Style {
                        margin: UiRect {
                            top: Val::Px(8.),
                            bottom: Val::Px(16.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                })
                .insert(RemarkUi);

            // spawn main menu button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_self: AlignSelf::FlexEnd,
                        // center button
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            top: Val::Px(72.),
                            bottom: Val::Px(24.),
                        },
                        size: Size::new(Val::Px(180.0), Val::Px(62.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Main Menu",
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 34.0,
                            color: Color::rgba(1., 1., 0.8, 1.0),
                        },
                    ));
                });
        });
}

/// system: change the background depending on wave
pub fn change_background_per_wave(
    wave: Res<Wave>,
    query: Query<(&TilemapId, &TilePos, &mut TileTextureIndex)>,
    query_background: Query<(Entity, &mut Transform), With<Background>>,
    mut event_reader: EventReader<NextWaveEvent>,
) {
    for _ in event_reader.iter() {
        let (upper_i, lower_i) = match wave.0 {
            0 => (0, 1),
            1 => (0, 1),
            2 => (0, 1),
            3 => (0, 2),
            4 => (0, 2),
            5 => (3, 1),
            6 => (3, 1),
            _ => (0, 1),
        };

        crate::background::reset_background(query_background, query, upper_i, lower_i);
        break;
    }
}
