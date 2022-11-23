//! dynamite logic. guy needs to pick it up or it explodes

use crate::animation::{OneShotAnimationTimer, ToggleVisibility};
use crate::audio::{BounceAudio, GameSoundSources};
use crate::bomb::BombExplosionBundle;
use crate::events::{DynamiteDefusedEvent, ExplodedEvent, ExplosiveKind};
use crate::guy::GuyState;
use crate::helper::{DelayedComponent, TimeToLive};
use crate::movement::{Gravity, SpatialPosition, SpatialVelocity};
use crate::{animation::LoopedAnimationTimer, helper::BaseTranslation};
use crate::{poptext, DefaultFont};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Default, Component)]
pub struct Dynamite;

#[derive(Component, Deref, DerefMut)]
pub struct TimeToExplode(pub Timer);

impl TimeToExplode {
    pub fn new(after: Duration) -> Self {
        TimeToExplode(Timer::new(after, TimerMode::Once))
    }
}

impl Default for TimeToExplode {
    fn default() -> Self {
        TimeToExplode::new(Duration::from_secs(5))
    }
}

#[derive(Default, Bundle)]
pub struct DynamiteBundle {
    pub dynamite: Dynamite,
    pub time_to_explode: TimeToExplode,
    pub position: SpatialPosition,
    pub velocity: SpatialVelocity,
    pub base_translation: BaseTranslation,
    pub sprite_sheet: SpriteSheetBundle,
    pub animation_timer: LoopedAnimationTimer,
}

#[derive(Resource, Deref)]
pub struct DynamiteTextureAtlas(pub Handle<TextureAtlas>);

#[derive(Resource, Deref)]
pub struct DynamiteExplosionTextureAtlas(pub Handle<TextureAtlas>);

/// system: set up dynamite
pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let tex: Handle<Image> = asset_server.load("img/dynamite.png");

    let texture_atlas = TextureAtlas::from_grid(tex, Vec2::new(30.0, 18.0), 2, 1, None, None);
    let texture_atlas = texture_atlases.add(texture_atlas);

    commands.insert_resource(DynamiteTextureAtlas(texture_atlas.clone()));

    // load dynamite explosion spritesheet
    let tex_explosion: Handle<Image> = asset_server.load("img/dynamite-explosion.png");
    let atlas = TextureAtlas::from_grid(tex_explosion, Vec2::new(48.0, 48.0), 3, 1, None, None);
    let explosion_texture_atlas_handle = texture_atlases.add(atlas);

    // save as resource for later
    commands.insert_resource(DynamiteExplosionTextureAtlas(
        explosion_texture_atlas_handle,
    ));
}

pub fn spawn_dynamite(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    bounce_sound: Handle<AudioSource>,
    pos: Vec3,
    velocity: Vec3,
) -> Entity {
    let scale = 2.0;
    let position = pos;
    commands
        .spawn(DynamiteBundle {
            position: SpatialPosition(position),
            velocity: SpatialVelocity(velocity),
            base_translation: BaseTranslation(Vec2::from_array([0., -1.])),
            sprite_sheet: SpriteSheetBundle {
                texture_atlas,
                transform: Transform {
                    scale: Vec3::splat(scale),
                    // translate off screen,
                    // letting spatial position system take care of it
                    translation: Vec3::new(0., 9e7, 0.125),
                    ..default()
                },
                ..default()
            },
            animation_timer: LoopedAnimationTimer::new(Duration::from_millis(100)),
            ..default()
        })
        // extra components
        .insert((
            // gravity, removed once it's on the ground
            Gravity::default(),
            // bounce with this sound specifically
            BounceAudio(bounce_sound),
            // blink shortly before it explodes
            DelayedComponent::new(ToggleVisibility::default(), Duration::from_millis(4_100)),
        ))
        .id()
}

/// system: if guy is close to dynamite, defuse it
pub fn detect_guy_touch_dynamite(
    mut commands: Commands,
    query_guy: Query<(&GuyState, &SpatialPosition, &BaseTranslation)>,
    query_dynamite: Query<(Entity, &SpatialPosition), With<Dynamite>>,
    font: Res<DefaultFont>,
    audio: Res<Audio>,
    sound_sources: Res<GameSoundSources>,
    mut event_writer: EventWriter<DynamiteDefusedEvent>,
) {
    let Ok((guy_state, guy_pos, base_translation)) = query_guy.get_single() else {
        return;
    };

    // do not pick up dynamites in these states
    if matches!(
        guy_state,
        GuyState::Disarming { .. } | GuyState::Ouch | GuyState::Victorious
    ) {
        return;
    }

    const DIST_REACH: f32 = 30.;
    const DIST_SQR_REACH: f32 = DIST_REACH * DIST_REACH;

    for (entity, pos) in &query_dynamite {
        let dist_sqr = (guy_pos.0 + base_translation.0.extend(0.)).distance_squared(pos.0);
        if dist_sqr <= DIST_SQR_REACH {
            // grab it!
            event_writer.send(DynamiteDefusedEvent(entity));

            // play audio
            audio.play(sound_sources.woosh.cast_weak());

            poptext::spawn_popup_text(
                &mut commands,
                font.0.clone(),
                pos.0.truncate(),
                "defused",
                16.,
                Color::WHITE,
            );

            commands.entity(entity).despawn();
        }
    }
}

/// system: handle dybamite explosion, with visual and audio feedback
pub fn dynamite_tick(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<Audio>,
    sound_sources: Res<GameSoundSources>,
    explosion_texture_atlas: Res<DynamiteExplosionTextureAtlas>,
    mut query: Query<(Entity, &mut TimeToExplode, &SpatialPosition), With<Dynamite>>,
    mut event_writer: EventWriter<ExplodedEvent>,
) {
    for (entity, mut time_to_explode, position) in &mut query {
        time_to_explode.tick(time.delta());
        if time_to_explode.just_finished() {
            explode(
                &mut commands,
                entity,
                position,
                &audio,
                &sound_sources,
                explosion_texture_atlas.clone(),
                &mut event_writer,
            );
        }
    }
}

fn explode(
    commands: &mut Commands,
    bomb_entity: Entity,
    position: &SpatialPosition,
    audio: &Audio,
    sound_sources: &GameSoundSources,
    texture_atlas: Handle<TextureAtlas>,
    event_writer: &mut EventWriter<ExplodedEvent>,
) -> Entity {
    // spawn explosion thingy at the same place
    let e = commands
        .spawn(BombExplosionBundle {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas,
                transform: Transform {
                    translation: position.truncate().extend(0.25),
                    scale: Vec3::new(2., 2., 1.),
                    ..default()
                },
                ..default()
            },
            animation: OneShotAnimationTimer::new(0.04, 2),
            ttl: TimeToLive::new(Duration::from_millis(175)),
            position: SpatialPosition(position.0),
        })
        .id();

    // emit sound effect
    audio.play(sound_sources.thwack1.cast_weak());

    // emit event
    event_writer.send(ExplodedEvent {
        kind: ExplosiveKind::Dynamite,
        position: position.0,
    });

    // remove dynamite
    commands.entity(bomb_entity).despawn();

    // return explosion entity
    e
}
