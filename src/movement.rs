use bevy::prelude::*;

use crate::audio::BounceAudio;

#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

pub fn apply_velocity(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

pub fn apply_velocity_to_text_styles(time: Res<Time>, mut query: Query<(&Velocity, &mut Style)>) {
    for (velocity, mut text_style) in query.iter_mut() {
        let _ = text_style
            .position
            .bottom
            .try_add_assign(Val::Px(velocity.0.y * time.delta_seconds()));
        let _ = text_style
            .position
            .left
            .try_add_assign(Val::Px(velocity.0.x * time.delta_seconds()));
    }
}

/// The maximum speed at which something can move
#[derive(Debug, Component)]
pub struct MaxSpeed(pub f32);

pub fn move_towards(
    vel: &mut SpatialVelocity,
    speed: &MaxSpeed,
    source_translation: &Vec3,
    target: &Vec2,
) -> bool {
    let pos = Vec2::new(source_translation.x, source_translation.y);
    let relative = *target - pos;

    if vel.0.z != 0. {
        return false;
    }

    if relative.length_squared() > 1. {
        vel.0 = relative.normalize().extend(0.) * speed.0;
        true
    } else {
        *vel = SpatialVelocity::default();
        false
    }
}

/// Full position, for entities which may also be above the ground.
#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct SpatialPosition(pub Vec3);

pub fn spatial_position_to_transform(mut query: Query<(&mut Transform, &SpatialPosition)>) {
    for (mut transform, pos) in &mut query {
        transform.translation.x = pos.0.x;
        transform.translation.y = pos.0.y + pos.0.z * 0.5;
    }
}

#[derive(Debug, Default, Component, Deref, DerefMut)]
pub struct SpatialVelocity(pub Vec3);

pub fn apply_spatial_velocity(
    time: Res<Time>,
    mut query: Query<(&mut SpatialPosition, &SpatialVelocity)>,
) {
    for (mut pos, velocity) in query.iter_mut() {
        pos.x += velocity.x * time.delta_seconds();
        pos.y += velocity.y * time.delta_seconds();
        pos.z += velocity.z * time.delta_seconds();
    }
}

/// A gravity pull,
/// made as a component so that we can control gravity per entity
/// and remove it at will
#[derive(Deref, DerefMut, Component)]
pub struct Gravity(pub f32);

impl Default for Gravity {
    fn default() -> Self {
        Gravity(4000.)
    }
}

/// apply gravity pull
pub fn apply_gravity(time: Res<Time>, mut query: Query<(&mut SpatialVelocity, &Gravity)>) {
    let delta = time.delta_seconds();
    for (mut velocity, gravity) in &mut query {
        velocity.z -= gravity.0 * delta;
    }
}

/// implement floor collision
pub fn collide_on_floor(
    audio: Res<Audio>,
    mut query: Query<(
        &mut SpatialPosition,
        &mut SpatialVelocity,
        Option<&BounceAudio>,
    )>,
) {
    for (mut pos, mut vel, bounce_sound) in &mut query {
        if pos.z <= 0. {
            pos.z = 0.;
            // check whether we're falling fast
            if vel.z < -500. {
                // bounce! (dampened)
                vel.z = -vel.z * 0.325;

                // play effect
                if let Some(sound) = bounce_sound {
                    audio.play(sound.0.clone());
                }
            } else if vel.z <= -1e-11 {
                // not fast enough, just stop velocity altogether
                vel.0 = Vec3::new(0., 0., 0.);
            }
        }
    }
}

const BOUND_W: f32 = 380.;
const BOUND_H: f32 = 496.;

pub fn apply_boundaries(mut query: Query<&mut SpatialPosition>) {
    for mut pos in &mut query {
        pos.0.x = pos.0.x.clamp(0., BOUND_W);
        pos.0.y = pos.0.y.clamp(0., BOUND_H);
    }
}
