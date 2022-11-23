use std::marker::PhantomData;

use bevy::{prelude::*, utils::Duration};

#[derive(Debug, Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

/// A fixed component not affected by Z depth adjustments
#[derive(Debug, Component)]
pub struct Fixed;

#[derive(Debug, Default, Component)]
pub struct BaseTranslation(pub Vec2);

pub fn z_depth(mut query: Query<(&mut Transform, &BaseTranslation, Without<Fixed>)>) {
    for (mut transform, base_translation, _) in query.iter_mut() {
        transform.translation.z = 24. - (transform.translation.y + base_translation.0.y) * 0.00001;
    }
}

/// For entities that are destroyed at the given time of death
#[derive(Debug, Component)]
pub struct TimeToLive(Timer);

impl TimeToLive {
    pub fn new(ttl: Duration) -> Self {
        TimeToLive(Timer::new(ttl, TimerMode::Once))
    }
}

impl Default for TimeToLive {
    fn default() -> Self {
        TimeToLive::new(Duration::from_secs(1))
    }
}

pub fn destroy_on_ttl(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimeToLive)>,
) {
    for (entity, mut ttl) in &mut query {
        ttl.0.tick(time.delta());
        if ttl.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// a component which is inserted on an entity only after a while
#[derive(Component)]
pub struct DelayedComponent<T: Component> {
    component: Option<T>,
    timer: Timer,
}

impl<T: Component> DelayedComponent<T> {
    pub fn new(component: T, after: Duration) -> Self {
        DelayedComponent {
            component: Some(component),
            timer: Timer::new(after, TimerMode::Once),
        }
    }
}

/// system: do delayed insertion
pub fn delayed_insertion<T>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedComponent<T>)>,
) where
    T: Component,
{
    for (entity, mut delayed_component) in &mut query {
        delayed_component.timer.tick(time.delta());
        if delayed_component.timer.just_finished() {
            let Some(component) = delayed_component.component.take() else {
                warn!("Component for entity {:?} already inserted", &entity);
                continue;
            };
            commands
                .entity(entity)
                // insert component
                .insert(component)
                // remove delayed component
                .remove::<DelayedComponent<T>>();
        }
    }
}

/// a component which removes a component by type after a while
#[derive(Component)]
pub struct DelayedRemoval<T: Component> {
    component: PhantomData<T>,
    timer: Timer,
}

impl<T: Component> DelayedRemoval<T> {
    pub fn new(after: Duration) -> Self {
        DelayedRemoval {
            component: PhantomData,
            timer: Timer::new(after, TimerMode::Once),
        }
    }
}

/// system: do delayed removal of a component by type
pub fn delayed_removal<T>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedRemoval<T>)>,
) where
    T: Component,
{
    for (entity, mut delayed_component) in &mut query {
        delayed_component.timer.tick(time.delta());
        if delayed_component.timer.finished() {
            commands
                .entity(entity)
                // remove component
                .remove::<T>()
                // remove itself
                .remove::<DelayedRemoval<T>>();
        }
    }
}

/// A component describing a generic event
/// to be triggered once at a later time.
#[derive(Component)]
pub struct ScheduledEvent<E: Send> {
    timer: Timer,
    event: E,
}

impl<E: Send> ScheduledEvent<E> {
    pub fn new(event: E, after: Duration) -> Self {
        ScheduledEvent {
            timer: Timer::new(after, TimerMode::Once),
            event,
        }
    }
}

#[derive(Default, Component)]
pub struct DespawnOnTrigger;

/// system: run scheduled events if it's their time to trigger
pub fn run_scheduled_events<E: 'static + Send + Sync + Clone>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScheduledEvent<E>, Option<&DespawnOnTrigger>)>,
    mut event_writer: EventWriter<E>,
) {
    for (entity, mut scheduled_event, despawn_on_trigger) in &mut query {
        scheduled_event.timer.tick(time.delta());

        if scheduled_event.timer.just_finished() {
            event_writer.send(scheduled_event.event.clone());
            if despawn_on_trigger.is_some() {
                commands.entity(entity).despawn();
            } else {
                commands
                    .entity(entity)
                    .remove::<ScheduledEvent<E>>()
                    .remove::<crate::spawner::SpawnerCooldown>()
                    .remove::<crate::spawner::PendingThrow>();
            }
        }
    }
}
