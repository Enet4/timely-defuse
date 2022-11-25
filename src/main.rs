use animation::ToggleVisibility;
use bevy::prelude::*;
use bevy::utils::Duration;
use bevy::window::PresentMode;
use bevy_ecs_tilemap::TilemapPlugin;
use events::{
    BombDisarmedEvent, BombThrownEvent, CoffeePickedUpEvent, CoffeeThrownEvent, CoffeeWornOffEvent,
    DisarmCancelledEvent, DisarmProgressEvent, DynamiteDefusedEvent, DynamiteThrownEvent,
    ExplodedEvent, GuyHurtEvent, NextWaveEvent, WaveFinishedEvent,
};

mod animation;
mod audio;
mod background;
mod bomb;
mod coffee;
mod dynamite;
mod events;
mod guy;
mod helper;
mod ingame;
mod menu;
mod movement;
mod poptext;
mod progress_bar;
mod scores;
mod spawner;
mod waves;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum AppState {
    MainMenu,
    InGame,
    Paused,
    Settings,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        width: 380.,
                        height: 660.,
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
                        title: "Timely Defuse".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TilemapPlugin)
        // add the app state type
        .add_state(AppState::MainMenu)
        .add_event::<BombDisarmedEvent>()
        .add_event::<ExplodedEvent>()
        .add_event::<DisarmProgressEvent>()
        .add_event::<DisarmCancelledEvent>()
        .add_event::<DynamiteDefusedEvent>()
        .add_event::<WaveFinishedEvent>()
        .add_event::<NextWaveEvent>()
        .add_event::<DynamiteThrownEvent>()
        .add_event::<BombThrownEvent>()
        .add_event::<CoffeeThrownEvent>()
        .add_event::<CoffeePickedUpEvent>()
        .add_event::<CoffeeWornOffEvent>()
        .add_event::<GuyHurtEvent>()
        .init_resource::<scores::GameScores>()
        .init_resource::<spawner::Rng>()
        .add_startup_system(background::setup)
        .add_startup_system(audio::setup)
        .add_system(handle_state_changes)
        .add_system(animation::fade_out)
        .add_system(animation::fade_out_on_text)
        .add_system(animation::wobble)
        .add_system(animation::make_things_rotate)
        .add_system(animation::fade_in_ui)
        .add_system(animation::animate_loops)
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(menu::setup))
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(menu::button_system)
                .with_system(menu::animate_background),
        )
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(menu::destroy))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(ingame::setup))
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(ingame::destroy))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(ingame::mouse_handler)
                .with_system(ingame::mouse_set_destination)
                .with_system(ingame::touch_system_create_squares)
                .with_system(ingame::touch_set_destination)
                .with_system(animation::animate_one_shot)
                .with_system(animation::animate_loops)
                .with_system(animation::toggle_visibility)
                .with_system(movement::apply_velocity)
                .with_system(movement::apply_velocity_to_text_styles)
                .with_system(movement::spatial_position_to_transform)
                .with_system(movement::apply_spatial_velocity)
                .with_system(movement::apply_gravity)
                .with_system(movement::collide_on_floor)
                .with_system(movement::apply_boundaries.after(movement::apply_velocity))
                .with_system(helper::destroy_on_ttl)
                .with_system(helper::z_depth)
                .with_system(progress_bar::update_progress_bar)
                .with_system(progress_bar::clear_progress_bar)
                .with_system(guy::animate_guy)
                .with_system(guy::walk_to_destination)
                .with_system(guy::recover)
                .with_system(guy::disarming_bomb)
                .with_system(
                    guy::take_hit
                        .after(bomb::bomb_tick)
                        .after(guy::walk_to_destination),
                )
                .with_system(dynamite::detect_guy_touch_dynamite)
                .with_system(dynamite::dynamite_tick)
                .with_system(bomb::on_disarm_bomb.after(guy::disarming_bomb))
                .with_system(bomb::bomb_tick.after(bomb::on_disarm_bomb))
                .with_system(coffee::detect_guy_touch_coffee)
                .with_system(coffee::coffee_effect_wear_off)
                .with_system(scores::on_disarm_bomb)
                .with_system(scores::on_pickup_dynamite)
                .with_system(scores::on_bomb_explode)
                .with_system(scores::on_guy_hurt)
                .with_system(scores::update_stats)
                .with_system(helper::run_scheduled_events::<DynamiteThrownEvent>)
                .with_system(helper::run_scheduled_events::<BombThrownEvent>)
                .with_system(helper::run_scheduled_events::<CoffeeThrownEvent>)
                .with_system(helper::run_scheduled_events::<NextWaveEvent>)
                .with_system(
                    spawner::handle_spawners::<DynamiteThrownEvent>
                        .before(waves::detect_wave_finish),
                )
                .with_system(
                    spawner::handle_spawners::<BombThrownEvent>.before(waves::detect_wave_finish),
                )
                .with_system(
                    spawner::handle_spawners::<CoffeeThrownEvent>.before(waves::detect_wave_finish),
                )
                .with_system(spawner::throw_bomb)
                .with_system(spawner::throw_dynamite)
                .with_system(spawner::throw_coffee)
                .with_system(waves::detect_wave_finish)
                .with_system(waves::change_background_per_wave.before(waves::on_next_wave))
                .with_system(waves::on_next_wave.after(waves::detect_wave_finish))
                .with_system(ingame::button_system),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(animation::detect_toggle_visibility_removal)
                .with_system(helper::delayed_insertion::<ToggleVisibility>)
                .with_system(helper::delayed_removal::<ToggleVisibility>),
        )
        .add_system_set(
            SystemSet::on_enter(AppState::InGame)
                .with_system(setup)
                .with_system(guy::setup)
                .with_system(bomb::setup)
                .with_system(dynamite::setup)
                .with_system(coffee::setup),
        )
        .run();
}

#[derive(Debug, Clone, Resource, Deref)]
pub struct DefaultFont(Handle<Font>);

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    default_font: Option<Res<DefaultFont>>,
) {
    // load common assets

    // text font
    let _font: Handle<Font> = default_font.map(|f| f.0.clone()).unwrap_or_else(|| {
        let f = asset_server.load("font/Pixelme.ttf");
        commands.insert_resource(DefaultFont(f.clone()));
        // install as default font, for others to use
        f
    });
}

#[derive(Component)]
pub struct DelayedStateChange {
    pub timer: Timer,
    pub state: AppState,
}

impl DelayedStateChange {
    pub fn new(to: AppState, after: Duration) -> Self {
        DelayedStateChange {
            timer: Timer::new(after, TimerMode::Once),
            state: to,
        }
    }
}

/// system: apply delayed state change when the time is right
pub fn handle_state_changes(
    mut commands: Commands,
    mut app_state: ResMut<State<AppState>>,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedStateChange)>,
) {
    for (entity, mut state_change) in &mut query {
        state_change.timer.tick(time.delta());
        if state_change.timer.just_finished() {
            let _ = app_state.set(state_change.state);

            // destroy itself
            commands.entity(entity).despawn();
        }
    }
}
