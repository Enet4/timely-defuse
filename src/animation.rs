use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::utils::Duration;

/// For things that rotate a bit
#[derive(Debug, Component)]
pub struct Rotating {
    pub rpm: f32,
}

pub fn make_things_rotate(
    time: Res<Time>,
    mut square_transforms: Query<(&mut Transform, &Rotating)>,
) {
    for (mut transform, rotating) in &mut square_transforms {
        transform.rotate_z(rotating.rpm / 60. * std::f32::consts::TAU * time.delta_seconds());
    }
}

/// Component that keeps track of the entitie's baseline scale
#[derive(Debug, Component)]
pub struct BaseScale(pub f32);

/// For entities with a "wobbly" effect
#[derive(Debug, Component)]
pub struct Wobbly;

pub fn wobble(
    time: Res<Time>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>, With<Wobbly>)>,
) {
    let t = time.elapsed_seconds();
    let a1 = t * 9.;

    for (mut transform, base_scale, _) in &mut transforms {
        let base = base_scale.map(|scale| scale.0).unwrap_or(1.);
        let w = 1. + a1.sin() * 0.07;
        let h = 1. + (a1 + 0.1).cos() * 0.05;
        transform.scale = Vec3::new(base * w, base * h, 1.);
    }
}

/// An animation timer that runs for N frames then stops at the last frame.
#[derive(Debug, Component)]
pub struct OneShotAnimationTimer {
    timer: Timer,
    n_frames: u32,
}

impl OneShotAnimationTimer {
    pub fn new(seconds_per_frame: f32, n_frames: u32) -> Self {
        OneShotAnimationTimer {
            timer: Timer::from_seconds(seconds_per_frame, TimerMode::Repeating),
            n_frames,
        }
    }
}

/// system: run one-shot animation timers
pub fn animate_one_shot(
    time: Res<Time>,
    mut query: Query<(&mut OneShotAnimationTimer, &mut TextureAtlasSprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        if timer.n_frames == 0 {
            continue;
        }

        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            sprite.index += 1;
            timer.n_frames -= 1;

            if timer.n_frames == 0 {
                timer.timer.pause();
            }
        }
    }
}

#[derive(Component)]
pub struct ToggleVisibility(pub Timer);

impl Default for ToggleVisibility {
    fn default() -> Self {
        ToggleVisibility(Timer::new(Duration::from_millis(40), TimerMode::Repeating))
    }
}

/// system: toggle visibility, so that it looks like it is blinking fast
pub fn toggle_visibility(
    time: Res<Time>,
    mut query: Query<(&mut ToggleVisibility, &mut Visibility)>,
) {
    for (mut timer, mut visibility) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            visibility.is_visible = !visibility.is_visible;
        }
    }
}

pub fn detect_toggle_visibility_removal(
    removed: RemovedComponents<ToggleVisibility>,
    mut query: Query<&mut Visibility>,
) {
    for entity in &removed {
        if let Ok(mut visibility) = query.get_mut(entity) {
            visibility.is_visible = true;
        }
    }
}

/// An animation timer that runs through all frames in the sprite sheet
/// in a loop.
#[derive(Debug, Component)]
pub struct LoopedAnimationTimer(Timer);

impl LoopedAnimationTimer {
    pub fn new(interval: Duration) -> Self {
        LoopedAnimationTimer(Timer::new(interval, TimerMode::Repeating))
    }
}

impl Default for LoopedAnimationTimer {
    fn default() -> Self {
        LoopedAnimationTimer::new(Duration::from_millis(100))
    }
}

/// system: run looped animations
pub fn animate_loops(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut LoopedAnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            sprite.index = (sprite.index + 1)
                % texture_atlases
                    .get(texture_atlas_handle)
                    .unwrap()
                    .textures
                    .len();
        }
    }
}

/// A fade-out effect.
///
/// Note: combine this with [`TimeToLive`][crate::helper::TimeToLive]
/// so that it disappears afterwards.
#[derive(Debug, Component)]
pub struct FadeOut {
    duration: Duration,
    stopwatch: Stopwatch,
}

impl Default for FadeOut {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl FadeOut {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            stopwatch: Default::default(),
        }
    }
}

/// system: implement fade-out by adjusting the entity's material color
pub fn fade_out(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut FadeOut, &Handle<ColorMaterial>)>,
) {
    for (mut fade_out, mat) in &mut query {
        fade_out.stopwatch.tick(time.delta());
        let opacity =
            (1. - (fade_out.stopwatch.elapsed_secs() / fade_out.duration.as_secs_f32())).max(0.);
        if let Some(material) = materials.get_mut(mat) {
            material.color.set_a(opacity);
        } else {
            error!("Missing material for handle {:?}", mat);
        }
    }
}

/// system: implement fade-out on text by adjusting the entity's text color
pub fn fade_out_on_text(time: Res<Time>, mut query: Query<(&mut FadeOut, &mut Text)>) {
    for (mut fade_out, mut text) in &mut query {
        fade_out.stopwatch.tick(time.delta());
        let opacity =
            (1. - (fade_out.stopwatch.elapsed_secs() / fade_out.duration.as_secs_f32())).max(0.);

        for section in &mut text.sections {
            section.style.color.set_a(opacity);
        }
    }
}

/// A fade-in effect.
#[derive(Debug, Component)]
pub struct FadeIn {
    duration: Duration,
    stopwatch: Stopwatch,
}

impl Default for FadeIn {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl FadeIn {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            stopwatch: Default::default(),
        }
    }
}

// system: implement fade-in by adjusting the entity's material color
/*
pub fn fade_in_entity(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &mut FadeOut, &Handle<ColorMaterial>)>,
) {
    for (entity, mut fade_out, mat) in &mut query {
        fade_out.stopwatch.tick(time.delta());
        let opacity = (fade_out.stopwatch.elapsed_secs() / fade_out.duration.as_secs_f32()).max(1.);
        if let Some(material) = materials.get_mut(mat) {
            material.color.set_a(opacity);
        } else {
            error!("Missing material for handle {:?}", mat);
        }
        if opacity >= 1. {
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}
*/

/// system: implement fade-in by adjusting the UI node's background color
pub fn fade_in_ui(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeIn, &mut BackgroundColor)>,
) {
    for (entity, mut fade_in, mut background_color) in &mut query {
        fade_in.stopwatch.tick(time.delta());
        let opacity = (fade_in.stopwatch.elapsed_secs() / fade_in.duration.as_secs_f32()).min(1.);
        background_color.0.set_a(opacity);
        if opacity >= 1. {
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

pub fn spawn_fade_in_black_screen(commands: &mut Commands, duration: Duration) -> Entity {
    commands
        .spawn((
            NodeBundle {
                node: Node::default(),
                style: Style {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    size: Size {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                    },
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0., 0., 0., 0.)),
                focus_policy: bevy::ui::FocusPolicy::Block,
                visibility: Visibility { is_visible: true },
                z_index: ZIndex::Global(999),
                ..default()
            },
            FadeIn {
                duration,
                stopwatch: Stopwatch::new(),
            },
        ))
        .id()
}
