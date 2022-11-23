//! coffee logic. guy picks it up for a speed and perforamnce boost

use crate::animation::ToggleVisibility;
use crate::audio::GameSoundSources;
use crate::events::CoffeePickedUpEvent;
use crate::guy::{GuyPerformance, GuyState, GUY_BASE_PERFORMANCE, GUY_BASE_SPEED};
use crate::helper::BaseTranslation;
use crate::helper::{DelayedComponent, TimeToLive};
use crate::movement::{Gravity, MaxSpeed, SpatialPosition, SpatialVelocity};
use crate::{poptext, DefaultFont};
use bevy::prelude::*;
use bevy::utils::Duration;

#[derive(Default, Component)]
pub struct Coffee;

#[derive(Default, Bundle)]
pub struct CoffeeBundle {
    pub coffe: Coffee,
    pub time_to_live: TimeToLive,
    pub position: SpatialPosition,
    pub velocity: SpatialVelocity,
    pub base_translation: BaseTranslation,
    pub sprite: SpriteBundle,
}

#[derive(Resource, Deref)]
pub struct CoffeeTexture(pub Handle<Image>);

/// system: set up coffee
pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let tex: Handle<Image> = asset_server.load("img/coffee.png");

    commands.insert_resource(CoffeeTexture(tex.clone()));
}

pub fn spawn_coffee(
    commands: &mut Commands,
    texture: Handle<Image>,
    pos: Vec3,
    velocity: Vec3,
) -> Entity {
    let scale = 2.0;
    let position = pos;
    commands
        .spawn(CoffeeBundle {
            position: SpatialPosition(position),
            velocity: SpatialVelocity(velocity),
            base_translation: BaseTranslation(Vec2::from_array([0., -1.])),
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    scale: Vec3::splat(scale),
                    // translate off screen,
                    // letting spatial position system take care of it
                    translation: Vec3::new(0., 9e7, 0.125),
                    ..default()
                },
                ..default()
            },
            time_to_live: TimeToLive::new(Duration::from_millis(3_500)),
            ..default()
        })
        // extra components
        .insert((
            // gravity, removed once it's on the ground
            Gravity::default(),
            // blink shortly before it disappears
            DelayedComponent::new(ToggleVisibility::default(), Duration::from_millis(3_000)),
        ))
        .id()
}

/// system: if guy is close to coffee, drink it
pub fn detect_guy_touch_coffee(
    mut commands: Commands,
    font: Res<DefaultFont>,
    audio: Res<Audio>,
    sound_sources: Res<GameSoundSources>,
    mut query_guy: Query<(
        Entity,
        &mut GuyPerformance,
        &mut MaxSpeed,
        &GuyState,
        &SpatialPosition,
        &BaseTranslation,
        Option<&CoffeeEffect>,
    )>,
    query_coffee: Query<(Entity, &SpatialPosition), With<Coffee>>,
    mut event_writer: EventWriter<CoffeePickedUpEvent>,
) {
    let Ok((guy_entity,
        mut guy_perf,
        mut max_speed,
        guy_state,
        guy_pos,
        base_translation,
        coffee_effect)
    ) = query_guy.get_single_mut() else {
        return;
    };

    // do not pick up coffee in these states
    if matches!(
        guy_state,
        GuyState::Disarming { .. } | GuyState::Ouch | GuyState::Victorious
    ) {
        return;
    }

    const DIST_REACH: f32 = 12.;
    const DIST_SQR_REACH: f32 = DIST_REACH * DIST_REACH;

    for (entity, pos) in &query_coffee {
        let dist_sqr = (guy_pos.0 + base_translation.0.extend(0.)).distance_squared(pos.0);
        if dist_sqr <= DIST_SQR_REACH {
            // grab it!
            event_writer.send(CoffeePickedUpEvent(entity));

            // emit sound effect
            audio.play(sound_sources.drink.cast_weak());

            // if guy is not under the effect of coffee yet
            if coffee_effect.is_none() {
                // apply coffee effect
                guy_perf.0 += GUY_BASE_PERFORMANCE;
                max_speed.0 += GUY_BASE_SPEED / 2.;
            }

            // insert coffee effec component so that it wears out
            commands
                .entity(guy_entity)
                .insert(CoffeeEffect::new(Duration::from_secs(9)));

            poptext::spawn_popup_text(
                &mut commands,
                font.0.clone(),
                pos.0.truncate(),
                "COFFEE!",
                16.,
                Color::WHITE,
            );

            commands.entity(entity).despawn();
        }
    }
}

/// Component applied on guy after drinking coffee
#[derive(Component)]
pub struct CoffeeEffect {
    timer: Timer,
}

impl CoffeeEffect {
    pub fn new(duration: Duration) -> Self {
        CoffeeEffect {
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}

/// system: coffee effect wears off after a while
pub fn coffee_effect_wear_off(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut CoffeeEffect,
        &mut GuyPerformance,
        &mut MaxSpeed,
    )>,
) {
    for (entity, mut effect, mut guy_perf, mut max_speed) in &mut query {
        effect.timer.tick(time.delta());

        if effect.timer.just_finished() {
            // revert performance parameters
            guy_perf.0 -= GUY_BASE_PERFORMANCE;
            max_speed.0 -= GUY_BASE_SPEED / 2.;

            commands.entity(entity).remove::<CoffeeEffect>();
        }
    }
}
