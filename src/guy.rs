//! The guy.

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    animation::ToggleVisibility,
    bomb::{BombState, BombTick},
    events::{
        BombDisarmedEvent, DisarmCancelledEvent, DisarmProgressEvent, ExplodedEvent, ExplosiveKind,
        GuyHurtEvent,
    },
    helper::{BaseTranslation, DelayedRemoval},
    movement::{move_towards, Gravity, MaxSpeed, SpatialPosition, SpatialVelocity},
    progress_bar::spawn_progress_bar,
};

/// Bundle for all components of the fat guy.
#[derive(Bundle)]
pub struct GuyBundle {
    pub state: GuyState,
    pub position: SpatialPosition,
    pub velocity: SpatialVelocity,
    pub max_speed: MaxSpeed,
    pub performance: GuyPerformance,
    pub destination: GuyDestination,
    pub base_translation: BaseTranslation,
    pub animation_timer: GuyAnimationTimer,
    pub sprite_sheet: SpriteSheetBundle,
}

#[derive(Debug, Default, PartialEq, Component)]
pub enum GuyState {
    /// Stopped, not doing anything
    #[default]
    Idle,
    /// walking towards somewhere
    Running,
    /// disarming a bomb
    Disarming {
        /// entity ID of the bomb
        bomb_entity: Entity,
        /// from 0 to 1
        progress: f32,
    },
    /// sad, let a bomb explode
    Ouch,
    /// yay
    Victorious,
    /// boo
    Loser,
}

pub const GUY_BASE_SPEED: f32 = 150.;
pub const GUY_BASE_PERFORMANCE: f32 = 0.25;

/// The speed at which guy defuses bombs
#[derive(Debug, Component)]
pub struct GuyPerformance(pub f32);

impl Default for GuyPerformance {
    fn default() -> Self {
        GuyPerformance(GUY_BASE_PERFORMANCE)
    }
}

/// The position that the character should move towards,
/// as defined by the user
#[derive(Debug, Component)]
pub struct GuyDestination(pub Vec2);

pub fn setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tex_guy: Handle<Image> = asset_server.load("img/fatguy.png");

    let texture_atlas = TextureAtlas::from_grid(tex_guy, Vec2::new(24.0, 32.0), 3, 7, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    spawn_guy(
        commands,
        texture_atlas_handle,
        &mut meshes,
        &mut materials,
        Vec2::new(180., 300.),
    );
}

fn spawn_guy(
    mut commands: Commands,
    texture_atlas_handle: Handle<TextureAtlas>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    pos: Vec2,
) -> Entity {
    let scale = 2.0;
    let guy_id = commands
        .spawn(GuyBundle {
            state: GuyState::Idle,
            position: SpatialPosition(pos.extend(0.)),
            velocity: SpatialVelocity::default(),
            max_speed: MaxSpeed(GUY_BASE_SPEED),
            animation_timer: GuyAnimationTimer::default(),
            performance: GuyPerformance::default(),
            destination: GuyDestination(pos),
            base_translation: BaseTranslation(Vec2::new(0., -22.)),
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform {
                    scale: Vec3::splat(scale),
                    translation: pos.extend(0.5),
                    ..default()
                },
                ..default()
            },
        })
        // additional components for temporary effects
        .insert((
            ToggleVisibility::default(),
            DelayedRemoval::<ToggleVisibility>::new(Duration::from_millis(750)),
        ))
        .id();

    let progress_bar = spawn_progress_bar(&mut commands, meshes, materials);

    commands.entity(guy_id).add_child(progress_bar);

    guy_id
}

#[derive(Debug, Component)]
pub struct GuyAnimationTimer {
    main_timer: Timer,
    ouch_timer: Timer,
    left: bool,
}

impl Default for GuyAnimationTimer {
    fn default() -> Self {
        GuyAnimationTimer {
            main_timer: Timer::from_seconds(0.10, TimerMode::Repeating),
            ouch_timer: Timer::from_seconds(0.20, TimerMode::Repeating),
            left: false,
        }
    }
}

const GUY_SPRITESHEET_UP_INDEX_START: usize = 0;
const GUY_SPRITESHEET_RIGHT_INDEX_START: usize = 3;
const GUY_SPRITESHEET_DOWN_INDEX_START: usize = 6;
const GUY_SPRITESHEET_LEFT_INDEX_START: usize = 9;
const GUY_SPRITESHEET_DEFUSE_INDEX_START: usize = 12;
const GUY_SPRITESHEET_DEFUSE_HURRY_INDEX_START: usize = 15;
const GUY_SPRITESHEET_OUCH_INDEX_START: usize = 18;
const GUY_SPRITESHEET_VICTORIOUS_INDEX: usize = 20;

pub fn animate_guy(
    time: Res<Time>,
    mut query: Query<(
        &mut GuyAnimationTimer,
        &mut TextureAtlasSprite,
        &GuyState,
        &SpatialVelocity,
    )>,
    bomb_query: Query<&BombTick>,
) {
    for (mut timer, mut sprite, guy_state, velocity) in &mut query {
        match *guy_state {
            GuyState::Idle => {
                sprite.index = GUY_SPRITESHEET_DOWN_INDEX_START + 1;
            }
            GuyState::Running => {
                timer.main_timer.tick(time.delta());
                if timer.main_timer.just_finished() {
                    // identify direction based on velocity
                    let base_index = if velocity.0.x.abs() >= velocity.0.y.abs() {
                        // horizontally (left or right)
                        if velocity.0.x < 0. {
                            // left
                            GUY_SPRITESHEET_LEFT_INDEX_START
                        } else {
                            // right
                            GUY_SPRITESHEET_RIGHT_INDEX_START
                        }
                    } else {
                        // vertically (up or down)
                        if velocity.0.y < 0. {
                            // down
                            GUY_SPRITESHEET_DOWN_INDEX_START
                        } else {
                            GUY_SPRITESHEET_UP_INDEX_START
                        }
                    };

                    if sprite.index < base_index || sprite.index > base_index + 2 {
                        sprite.index = base_index;
                    } else {
                        match (sprite.index - base_index, timer.left) {
                            (0, _) => {
                                sprite.index += 1;
                                timer.left = false;
                            }
                            (1, false) => {
                                sprite.index += 1;
                            }
                            (1, true) => {
                                sprite.index -= 1;
                            }
                            (2, _) => {
                                sprite.index -= 1;
                                timer.left = true;
                            }
                            _ => {
                                // unreachable, do nothing
                                debug_assert!(false)
                            }
                        }
                    }
                }
            }
            GuyState::Disarming { bomb_entity, .. } => {
                timer.main_timer.tick(time.delta());

                // choose animation based on bomb ticks left
                let base_index = {
                    // get ticks left
                    if let Ok(bomb_tick) = bomb_query.get(bomb_entity) {
                        if bomb_tick.ticks_left <= 1 {
                            GUY_SPRITESHEET_DEFUSE_HURRY_INDEX_START
                        } else {
                            GUY_SPRITESHEET_DEFUSE_INDEX_START
                        }
                    } else {
                        warn!("Bomb being disarmed ({:?}) is missing!", bomb_entity);
                        GUY_SPRITESHEET_DEFUSE_INDEX_START
                    }
                };

                if sprite.index < base_index {
                    sprite.index = base_index;
                }

                if timer.main_timer.just_finished() {
                    sprite.index += 1;
                    if sprite.index > base_index + 2 {
                        sprite.index = base_index;
                    }
                }
            }
            GuyState::Ouch | GuyState::Loser => {
                timer.ouch_timer.tick(time.delta());
                if sprite.index < GUY_SPRITESHEET_OUCH_INDEX_START
                    || sprite.index > GUY_SPRITESHEET_OUCH_INDEX_START + 1
                {
                    sprite.index = GUY_SPRITESHEET_OUCH_INDEX_START;
                }

                // 2-frame animation, switch between them
                if timer.ouch_timer.just_finished() {
                    sprite.index ^= 1;
                }
            }
            GuyState::Victorious => {
                sprite.index = GUY_SPRITESHEET_VICTORIOUS_INDEX;
            }
        }
    }
}

/// system to move guy to the given destination
pub fn walk_to_destination(
    event_writer: EventWriter<DisarmCancelledEvent>,
    mut query: Query<(
        &mut SpatialVelocity,
        &mut GuyState,
        &MaxSpeed,
        &SpatialPosition,
        &GuyDestination,
    )>,
) {
    if let Ok((mut vel, mut guy_state, guy_speed, position, destination)) = query.get_single_mut() {
        walk_towards(
            &mut vel,
            &mut guy_state,
            &guy_speed,
            &position,
            &destination,
            event_writer,
        );
    }
}

pub(crate) fn walk_towards(
    guy_vel: &mut SpatialVelocity,
    guy_state: &mut GuyState,
    guy_speed: &MaxSpeed,
    guy_pos: &Vec3,
    target: &GuyDestination,
    mut cancel_event_writer: EventWriter<DisarmCancelledEvent>,
) {
    // do not walk on ouch or victorious
    if *guy_state == GuyState::Ouch || *guy_state == GuyState::Victorious {
        return;
    }

    let will_move = move_towards(guy_vel, guy_speed, guy_pos, &target.0);

    if will_move {
        if let GuyState::Disarming { bomb_entity, .. } = guy_state {
            cancel_event_writer.send(DisarmCancelledEvent(*bomb_entity));
        }

        *guy_state = GuyState::Running;
    } else if *guy_state == GuyState::Running {
        *guy_state = GuyState::Idle;
    }
}

/// system that handles disarming a bomb if guy is close enough to it
pub fn disarming_bomb(
    time: Res<Time>,
    mut query_guy: Query<(
        &mut GuyState,
        &mut GuyDestination,
        &SpatialPosition,
        &BaseTranslation,
        &GuyPerformance,
    )>,
    query_bombs: Query<(
        Entity,
        &SpatialPosition,
        &BaseTranslation,
        &BombState,
        &BombTick,
    )>,
    mut bomb_disarmed_ev_writer: EventWriter<BombDisarmedEvent>,
    mut disarm_progress_ev_writer: EventWriter<DisarmProgressEvent>,
) {
    let Ok((mut guy_state, mut guy_destination, guy_position, guy_base_translation, perf)) = query_guy.get_single_mut() else {
        return
    };

    match *guy_state {
        GuyState::Running | GuyState::Ouch | GuyState::Victorious | GuyState::Loser => {
            /* no-op */
        }
        GuyState::Idle => {
            let guy_pos = guy_position.truncate() + guy_base_translation.0;
            // look for bombs
            let nearest_bomb = query_bombs
                .iter()
                .filter(|(_, _, _, state, _)| **state == BombState::Idle)
                .map(|(entity, bomb_pos, bomb_base_translation, _, _)| {
                    let bomb_pos = bomb_pos.truncate() - bomb_base_translation.0;
                    let distance_sqr = guy_pos.distance_squared(bomb_pos);
                    (entity, bomb_pos, distance_sqr)
                })
                .filter(|(_, _, dist_sqr)| *dist_sqr < 750.)
                .min_by(|(_, _, dist1), (_, _, dist2)| dist1.total_cmp(dist2));

            if let Some((bomb_entity, bomb_pos, dist_sqr)) = nearest_bomb {
                if dist_sqr < 4. {
                    // start disarming!
                    *guy_state = GuyState::Disarming {
                        bomb_entity,
                        progress: 0.,
                    };
                    // TODO(audio) play sound effect
                }
                // not enough, but close. set destination
                guy_destination.0 = bomb_pos - guy_base_translation.0;
            }
        }
        GuyState::Disarming {
            bomb_entity,
            progress,
        } => {
            // continue disarming

            // check ticks left to enter hurry mode
            let perf = if let Ok((_, _, _, _, bomb_tick)) = query_bombs.get(bomb_entity) {
                if bomb_tick.ticks_left <= 1 {
                    perf.0 * 3.
                } else {
                    perf.0
                }
            } else {
                warn!("Bomb being disarmed ({:?}) is missing!", bomb_entity);
                perf.0
            };
            let new_progress = progress + (perf * time.delta_seconds());
            *guy_state = GuyState::Disarming {
                bomb_entity,
                progress: new_progress,
            };

            if new_progress >= 1. {
                // finished disarming!
                bomb_disarmed_ev_writer.send(BombDisarmedEvent(bomb_entity));
                *guy_state = GuyState::Idle;
            } else {
                disarm_progress_ev_writer.send(DisarmProgressEvent(new_progress));
            }
        }
    }
}

/// system: guy takes a hit if something explodes
pub fn take_hit(
    mut commands: Commands,
    mut guy_query: Query<(
        Entity,
        &mut GuyState,
        &mut SpatialVelocity,
        &SpatialPosition,
    )>,
    mut event_reader: EventReader<ExplodedEvent>,
    mut event_writer: EventWriter<GuyHurtEvent>,
) {
    let Ok((guy, mut guy_state, mut guy_vel, guy_pos)) = guy_query.get_single_mut() else {
        return
    };

    const BOMB_EXPLOSION_RADIUS: f32 = 164.;
    const DYNAMITE_EXPLOSION_RADIUS: f32 = 112.;

    for event in event_reader.iter() {
        let ExplodedEvent { kind, position } = event;

        let guy_pos = guy_pos.0;
        let diff_pos: Vec3 = guy_pos - *position;
        let (r, intensity) = match kind {
            ExplosiveKind::Dynamite => (DYNAMITE_EXPLOSION_RADIUS, 1.8),
            ExplosiveKind::Bomb => (BOMB_EXPLOSION_RADIUS, 3.6),
        };
        if diff_pos.length_squared() < r * r {
            *guy_state = GuyState::Ouch;

            // add pushback effect on guy
            guy_vel.0 = intensity * diff_pos + Vec3::new(0., 0., 400. * intensity);
            commands.entity(guy).insert(Gravity::default());

            // send event (so that it affects score)
            event_writer.send(GuyHurtEvent { from: *kind });

            // schedule guy recovery
            commands
                .entity(guy)
                .insert(GuyRecovery::new(Duration::from_secs_f32(0.92 * intensity)));
        }
    }
}

#[derive(Component)]
pub struct GuyRecovery(pub Timer);

impl GuyRecovery {
    pub fn new(duration: Duration) -> Self {
        GuyRecovery(Timer::new(duration, TimerMode::Once))
    }
}

impl Default for GuyRecovery {
    fn default() -> Self {
        GuyRecovery::new(Duration::from_secs(2))
    }
}

/// system: guy recovers from ouch after a while
pub fn recover(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut GuyState,
        &mut GuyRecovery,
        &mut SpatialVelocity,
    )>,
) {
    for (entity, mut guy_state, mut guy_recovery, mut velocity) in &mut query {
        guy_recovery.0.tick(time.delta());
        if guy_recovery.0.just_finished() {
            if *guy_state == GuyState::Ouch {
                *guy_state = GuyState::Idle;
            }
            velocity.0 = Vec3::new(0., 0., 0.);
            commands
                .entity(entity)
                .remove::<GuyRecovery>()
                .remove::<Gravity>();
        }
    }
}
