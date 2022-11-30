use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use bevy::utils::Duration;
use bevy_ecs_tilemap::prelude::TilemapId;
use bevy_ecs_tilemap::tiles::{TilePos, TileTextureIndex};

use crate::animation::{spawn_fade_in_black_screen, BaseScale};
use crate::audio::GameSoundSources;
use crate::background::Background;
use crate::guy::GuyDestination;
use crate::helper::BaseTranslation;
use crate::menu::{HOVER_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::scores::GameScores;
use crate::{
    animation::{FadeOut, Wobbly},
    helper::TimeToLive,
    DefaultFont,
};
use crate::{AppState, DelayedStateChange};

#[derive(Default, Component)]
pub struct WaveUi;

#[derive(Default, Resource)]
pub struct Wave(pub u16);

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    windows: Res<Windows>,
    default_font: Option<Res<DefaultFont>>,
    query_background: Query<(Entity, &mut Transform), With<Background>>,
    query_tiles: Query<(&TilemapId, &TilePos, &mut TileTextureIndex)>,
) {
    let window = windows.get_primary().unwrap();
    // load assets

    // text font
    let font: Handle<Font> = default_font.map(|f| f.0.clone()).unwrap_or_else(|| {
        let f = asset_server.load("font/Pixelme.ttf");
        commands.insert_resource(DefaultFont(f.clone()));
        // install as default font, for others to use
        f
    });

    // build background
    crate::background::reset_background(query_background, query_tiles, 0, 1);

    let wave = 0;
    // initialize wave
    commands.insert_resource(Wave(wave));

    // initialize scores
    commands.insert_resource(GameScores::default());

    // wave number text
    commands.spawn((
        TextBundle::from_section(
            "WAVE 0",
            TextStyle {
                font: font.clone(),
                font_size: 32.,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::TOP_LEFT)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(8.),
                left: Val::Px(12.),
                ..default()
            },
            ..default()
        }),
        WaveUi,
    ));

    // 2D camera
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::from_array([
            window.width() as f32 / 2.,
            window.height() as f32 / 2.,
            999.,
        ])),
        ..Default::default()
    });

    crate::scores::spawn_game_score_ui(&mut commands, font);
    crate::waves::WAVE_DESCRIPTORS[0](commands);
}

pub fn touch_system_create_squares(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    touches: Res<Touches>,
) {
    for touch in touches.iter_just_pressed() {
        spawn_square(&mut commands, &mut meshes, &mut materials, touch.position());
    }
}

pub fn mouse_handler(
    mut commands: Commands,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.get_primary().unwrap();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = window.cursor_position() {
            spawn_square(&mut commands, &mut meshes, &mut materials, pos);
        }
    }
}

pub fn spawn_square(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
) {
    let base_scale = 16.;
    let transform = Transform {
        translation: pos.extend(-0.125),
        rotation: Default::default(),
        scale: Vec3::new(base_scale, base_scale, 1.),
    };
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform,
            material: materials.add(ColorMaterial::from(Color::CYAN)),
            ..Default::default()
        },
        // keep track of base scale
        BaseScale(base_scale),
        // make it "wobble"
        Wobbly,
        // fade out after a while
        FadeOut::new(Duration::from_millis(600)),
        // destroy after a while
        TimeToLive::new(Duration::from_millis(600)),
    ));
}

pub fn touch_set_destination(
    touches: Res<Touches>,
    mut query: Query<(&mut GuyDestination, &BaseTranslation)>,
) {
    for touch in touches.iter_just_pressed() {
        if let Ok((mut destination, base_translation)) = query.get_single_mut() {
            destination.0 = touch.position() - base_translation.0;
            // clamp to floor
            destination.0.y = destination.0.y.min(490.);
        }
    }
}

pub fn mouse_set_destination(
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut GuyDestination, &BaseTranslation)>,
) {
    let window = windows.get_primary().unwrap();
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = window.cursor_position() {
            if let Ok((mut destination, base_translation)) = query.get_single_mut() {
                destination.0 = pos - base_translation.0;
                // clamp to floor
                destination.0.y = destination.0.y.min(490.);
            }
        }
    }
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
                        AppState::MainMenu,
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
