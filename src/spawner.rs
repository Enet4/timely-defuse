use bevy::prelude::*;
use bevy::utils::Duration;

use rand::{self, Rng as _, SeedableRng};
use rand_pcg;

use crate::{
    audio::GameSoundSources,
    bomb::BombTextureAtlas,
    coffee::CoffeeTexture,
    dynamite::DynamiteTextureAtlas,
    events::{BombThrownEvent, CoffeeThrownEvent, DynamiteThrownEvent},
    helper::ScheduledEvent,
};

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct Rng(rand_pcg::Pcg32);

impl Default for Rng {
    fn default() -> Self {
        Self(rand_pcg::Pcg32::from_seed(rand::random()))
    }
}

#[derive(Debug, Component)]
pub struct Spawner {
    /// how many items left to spawn
    pub remaining: u32,

    /// whether it needs to spawn all items for the wave to end
    pub essential: bool,
}

impl Spawner {
    #[inline]
    pub fn new_essential(remaining: u32) -> Self {
        Spawner {
            remaining,
            essential: true,
        }
    }

    #[inline]
    pub fn new_nonessential(remaining: u32) -> Self {
        Spawner {
            remaining,
            essential: false,
        }
    }
}

/// If present, the spawner has already scheduled an event
/// and should wait until it is triggered
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SpawnerCooldown;

#[derive(Component)]
pub struct RandomEventProducer<E> {
    pub distribution: rand_distr::Exp<f32>,
    pub event: E,
}

impl<E: Send> RandomEventProducer<E> {
    pub fn new(lambda: f32, event: E) -> Self {
        RandomEventProducer {
            distribution: rand_distr::Exp::new(lambda).unwrap(),
            event,
        }
    }
}

impl<E: Send + Clone> RandomEventProducer<E> {
    pub fn sample(&self, rng: &mut Rng) -> ScheduledEvent<E> {
        let after: f32 = rng.0.sample(self.distribution);
        let after = after.clamp(0.5, 25.);
        debug!("Random sampling: spawn after {} seconds", after);
        ScheduledEvent::new(self.event.clone(), Duration::from_secs_f32(after))
    }
}

#[derive(Bundle)]
pub struct SpawnerBundle<E: 'static + Send + Sync> {
    spawner: Spawner,
    event_producer: RandomEventProducer<E>,
}

/// Used alongisde a scheduled event
/// to mean that the event is to throw something
#[derive(Component)]
pub struct PendingThrow;

/// system: handle spawner logic of
/// producing randomly timed throwing events
pub fn handle_spawners<E: 'static + Send + Sync + Clone>(
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    mut query: Query<(Entity, &mut Spawner, &RandomEventProducer<E>), Without<SpawnerCooldown>>,
) {
    for (entity, mut spawner, event_producer) in &mut query {
        if spawner.remaining > 0 {
            // sample
            let scheduled_event = event_producer.sample(&mut rng);

            commands
                .entity(entity)
                .insert((PendingThrow, scheduled_event));

            spawner.remaining -= 1;

            // add cooldown
            commands.entity(entity).insert(SpawnerCooldown);
        }
    }
}

/// system: on dynamite thrown, spawn it
pub fn throw_dynamite(
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    texture_atlas: Res<DynamiteTextureAtlas>,
    sound_sources: Res<GameSoundSources>,
    mut event_reader: EventReader<DynamiteThrownEvent>,
) {
    for _ in event_reader.iter() {
        let pos = random_xy_position(&mut rng);

        crate::dynamite::spawn_dynamite(
            &mut commands,
            texture_atlas.0.clone(),
            sound_sources.thwack3.clone(),
            pos.extend(1200.),
            random_velocity_variations(&mut rng),
        );
    }
    event_reader.clear();
}

/// system: on bomb thrown, spawn it
pub fn throw_bomb(
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    texture_atlas: Res<BombTextureAtlas>,
    sound_sources: Res<GameSoundSources>,
    mut event_reader: EventReader<BombThrownEvent>,
) {
    for _ in event_reader.iter() {
        let pos = random_xy_position(&mut rng);

        crate::bomb::spawn_bomb(
            &mut commands,
            texture_atlas.clone(),
            sound_sources.thwack10.clone(),
            pos.extend(1200.),
            random_velocity_variations(&mut rng),
            12,
        );
    }
    event_reader.clear();
}

/// system: on coffee thrown, spawn it
pub fn throw_coffee(
    mut commands: Commands,
    mut rng: ResMut<Rng>,
    texture: Res<CoffeeTexture>,
    mut event_reader: EventReader<CoffeeThrownEvent>,
) {
    for _ in event_reader.iter() {
        let pos = random_xy_position(&mut rng);

        crate::coffee::spawn_coffee(
            &mut commands,
            texture.clone(),
            pos.extend(1200.),
            random_velocity_variations(&mut rng),
        );
    }
    event_reader.clear();
}

fn random_xy_position(rng: &mut Rng) -> Vec2 {
    let x_pos = rand_distr::Uniform::new(12., 348.);
    let y_pos = rand_distr::Uniform::new(22., 360.);
    let x = rng.0.sample(x_pos);
    let y = rng.0.sample(y_pos);
    Vec2::new(x, y)
}

fn random_velocity_variations(rng: &mut Rng) -> Vec3 {
    let vel_variations_dist = rand_distr::Uniform::new(-2., 2.);
    let vel_x = rng.0.sample(vel_variations_dist);
    let vel_y = rng.0.sample(vel_variations_dist);
    let vel_variations_z_dist = rand_distr::Uniform::new(-20., 0.);
    let vel_z = rng.sample(vel_variations_z_dist);

    Vec3::new(vel_x, vel_y, vel_z)
}
