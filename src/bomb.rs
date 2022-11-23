use bevy::prelude::*;

use crate::{
    animation::{OneShotAnimationTimer, ToggleVisibility},
    audio::{BounceAudio, GameSoundSources},
    events::{BombDisarmedEvent, ExplodedEvent, ExplosiveKind},
    helper::{BaseTranslation, DelayedComponent, TimeToLive},
    movement::{Gravity, SpatialPosition, SpatialVelocity},
    poptext::spawn_popup_text,
    DefaultFont,
};
use bevy::utils::Duration;

#[derive(Default, Bundle)]
pub struct BombBundle {
    pub bomb: BombState,
    pub position: SpatialPosition,
    pub velocity: SpatialVelocity,
    pub bomb_tick: BombTick,
    pub base_translation: BaseTranslation,
    pub sprite_sheet: SpriteSheetBundle,
}

/// Identifies an entity as a bomb and tells its state
#[derive(Debug, Default, Eq, Hash, PartialEq, Component)]
pub enum BombState {
    #[default]
    Idle,
    Disarmed,
}

/// Component that provides bomb ticking feedback
#[derive(Debug, Component)]
pub struct BombTick {
    pub ticks_left: u32,
    timer: Timer,
}

impl BombTick {
    fn new(seconds_to_explode: u32) -> Self {
        Self {
            ticks_left: seconds_to_explode,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

impl Default for BombTick {
    fn default() -> Self {
        Self::new(10)
    }
}

#[derive(Debug, Resource, Deref)]
pub struct BombTextureAtlas(Handle<TextureAtlas>);

#[derive(Debug, Resource, Deref)]
pub struct BombExplosionTextureAtlas(Handle<TextureAtlas>);

/// system: setup
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // load bomb spritesheet
    let tex_bomb: Handle<Image> = asset_server.load("img/tnt.png");
    let atlas = TextureAtlas::from_grid(tex_bomb, Vec2::new(30.0, 18.0), 4, 1, None, None);
    let bomb_texture_atlas_handle = texture_atlases.add(atlas);

    // save as resource for later
    commands.insert_resource(BombTextureAtlas(bomb_texture_atlas_handle.clone()));

    // load bomb explosion spritesheet
    let tex_explosion: Handle<Image> = asset_server.load("img/Explosion.png");
    let atlas = TextureAtlas::from_grid(tex_explosion, Vec2::new(96.0, 96.0), 12, 1, None, None);
    let explosion_texture_atlas_handle = texture_atlases.add(atlas);

    // save as resource for later
    commands.insert_resource(BombExplosionTextureAtlas(explosion_texture_atlas_handle));
}

pub fn spawn_bomb(
    commands: &mut Commands,
    texture_atlas_handle: Handle<TextureAtlas>,
    bounce_sound: Handle<AudioSource>,
    pos: Vec3,
    velocity: Vec3,
    seconds: u32,
) -> Entity {
    let scale = 2.0;
    commands
        .spawn(BombBundle {
            position: SpatialPosition(pos),
            velocity: SpatialVelocity(velocity),
            bomb_tick: BombTick::new(seconds),
            base_translation: BaseTranslation(Vec2::new(0., -4.)),
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform {
                    scale: Vec3::splat(scale),
                    // translate off screen,
                    // letting spatial position system take care of it
                    translation: Vec3::new(0., 9e7, 0.125),
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert((Gravity::default(), BounceAudio(bounce_sound)))
        .id()
}

/// system: handle bomb ticking, with visual and audio feedback
pub fn bomb_tick(
    mut commands: Commands,
    time: Res<Time>,
    audio: Res<Audio>,
    default_font: Res<DefaultFont>,
    mut query: Query<(
        Entity,
        &BombState,
        &mut BombTick,
        &SpatialPosition,
        &Transform,
    )>,
    mut event_writer: EventWriter<ExplodedEvent>,
    explosion_texture_atlas: Res<BombExplosionTextureAtlas>,
    sound_sources: Res<GameSoundSources>,
) {
    for (entity, bomb_state, mut bomb_tick, position, transform) in &mut query {
        if bomb_tick.ticks_left == 0 || *bomb_state == BombState::Disarmed {
            continue;
        }

        bomb_tick.timer.tick(time.delta());
        if bomb_tick.timer.just_finished() {
            // bomb tick
            bomb_tick.ticks_left -= 1;

            // TODO sound effect

            // popup text with time left
            let color = match bomb_tick.ticks_left {
                0 => Color::rgba_u8(0xFF, 0x20, 0x00, 0xFF),
                1..=3 => Color::rgba_u8(0xF0, 0xA7, 0x10, 0xFF),
                _ => Color::rgba_u8(0xFF, 0xFF, 0x20, 0xFF),
            };

            if bomb_tick.ticks_left == 0 {
                explode(
                    &mut commands,
                    entity,
                    &audio,
                    position,
                    explosion_texture_atlas.clone(),
                    sound_sources.bomb_explosion.clone(),
                    &mut event_writer,
                );
            }

            spawn_popup_text(
                &mut commands,
                default_font.0.clone(),
                transform.translation.truncate(),
                bomb_tick.ticks_left.to_string(),
                28.,
                color,
            );
        }
    }
}

/// system: detect that the bomb was disarmed
pub fn on_disarm_bomb(
    mut commands: Commands,
    audio: Res<Audio>,
    sound_sources: Res<GameSoundSources>,
    mut event_reader: EventReader<BombDisarmedEvent>,
    mut query: Query<(&mut BombState, &mut TextureAtlasSprite)>,
) {
    for BombDisarmedEvent(bomb_entity) in event_reader.iter() {
        // find bomb by ID
        match query.get_mut(*bomb_entity) {
            Ok((mut state, mut sprite)) => {
                // set it as disarmed
                *state = BombState::Disarmed;
                // emit sound effect
                audio.play(sound_sources.disarm.cast_weak());

                // set first frame of bomb defusing
                sprite.index = 1;
                commands.entity(*bomb_entity).insert((
                    // engage animation timer so that it does the disarmed animation
                    OneShotAnimationTimer::new(0.12, 2),
                    // make it blink after a while
                    DelayedComponent::new(
                        ToggleVisibility::default(),
                        Duration::from_millis(1_200),
                    ),
                    // make it disappear after a while
                    TimeToLive::new(Duration::from_millis(1_800)),
                ));
            }
            Err(e) => {
                error!("Could not find bomb to disarm: {}", e);
            }
        }
    }
    event_reader.clear();
}

fn explode(
    commands: &mut Commands,
    bomb_entity: Entity,
    audio: &Audio,
    position: &SpatialPosition,
    explosion_texture_atlas: Handle<TextureAtlas>,
    explosion_audio: Handle<AudioSource>,
    event_writer: &mut EventWriter<ExplodedEvent>,
) {
    // spawn big explosion thingy at the same place
    spawn_explosion(&mut *commands, explosion_texture_atlas, position.0);

    // emit sound effect
    audio.play(explosion_audio);

    // emit event
    event_writer.send(ExplodedEvent {
        kind: ExplosiveKind::Bomb,
        position: position.0,
    });

    // remove bomb
    commands.entity(bomb_entity).despawn();
}

#[derive(Bundle)]
pub struct BombExplosionBundle {
    pub sprite_sheet: SpriteSheetBundle,
    pub animation: OneShotAnimationTimer,
    pub ttl: TimeToLive,
    pub position: SpatialPosition,
}

pub fn spawn_explosion(
    commands: &mut Commands,
    texture_atlas: Handle<TextureAtlas>,
    position: Vec3,
) -> Entity {
    commands
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
            animation: OneShotAnimationTimer::new(0.04, 12),
            ttl: TimeToLive::new(Duration::from_millis(380)),
            position: SpatialPosition(position),
        })
        .id()
}
