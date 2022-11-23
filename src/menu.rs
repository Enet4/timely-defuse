use bevy::prelude::*;
use bevy::utils::Duration;
use bevy_ecs_tilemap::{
    prelude::TilemapId,
    tiles::{TilePos, TileTextureIndex},
};

use crate::{
    animation::spawn_fade_in_black_screen, audio::GameSoundSources, background::Background,
    AppState, DefaultFont, DelayedStateChange,
};

pub const NORMAL_BUTTON: Color = Color::rgb(0.3, 0.3, 0.8);
pub const HOVER_BUTTON: Color = Color::rgb(0.4, 0.5, 0.9);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Res<Windows>,
    query_background: Query<(Entity, &mut Transform), With<Background>>,
    query_tiles: Query<(&TilemapId, &TilePos, &mut TileTextureIndex)>,
) {
    let window = windows.get_primary().unwrap();

    // text font
    let font: Handle<Font> = asset_server.load("font/Pixelme.ttf");

    // install as default font, for others to use
    commands.insert_resource(DefaultFont(font.clone()));

    // cool title
    let title: Handle<Image> = asset_server.load("img/cool-title.png");

    // reset background
    crate::background::reset_background(query_background, query_tiles, 0, 1);

    // UI camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::from_array([
            window.width() as f32 / 2.,
            window.height() as f32 / 2.,
            999.,
        ])),
        ..Default::default()
    });

    // UI title
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    size: Size {
                        width: Val::Auto,
                        height: Val::Auto,
                    },
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Px(20.),
                        bottom: Val::Px(24.),
                    },
                    ..default()
                },
                image_mode: bevy::ui::widget::ImageMode::KeepAspect,
                image: UiImage(title),
                ..default()
            });

            // UI start button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        // center button
                        margin: UiRect::all(Val::Auto),
                        size: Size::new(Val::Px(160.0), Val::Px(64.0)),
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
                        "Start",
                        TextStyle {
                            font,
                            font_size: 38.0,
                            color: Color::rgba(1., 1., 0.8, 1.0),
                        },
                    ));
                });
        });
}

pub fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    audio: Res<Audio>,
    sound_sources: Res<GameSoundSources>,
    transition_entity: Query<Entity, With<DelayedStateChange>>,
) {
    for (interaction, mut color, _children) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();

                audio.play(sound_sources.click.cast_weak());

                // ensure that we don't spawn more than one
                if transition_entity.get_single().is_err() {
                    // schedule app state transition
                    let e = spawn_fade_in_black_screen(&mut commands, Duration::from_millis(400));
                    commands.entity(e).insert(DelayedStateChange::new(
                        AppState::InGame,
                        Duration::from_millis(750),
                    ));
                }
            }
            Interaction::Hovered => {
                *color = HOVER_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn animate_background(time: Res<Time>, mut query: Query<&mut Transform, With<Background>>) {
    for mut transform in &mut query {
        transform.translation.x += 38. * time.delta_seconds();
        if transform.translation.x > 50. {
            transform.translation.x -= 96.;
        }
    }
}

pub fn destroy(
    mut commands: Commands,
    mut query: Query<Entity, (Without<TilemapId>, Without<Background>)>,
) {
    // destroy every component except for backgtound entity and tiles.
    // Note: this takes advantage of the fact that
    // background is the only thing using tilemaps
    for entity in &mut query {
        commands.entity(entity).despawn();
    }
}
