//! text that pops up from somewhere and disappears shortly after

use bevy::prelude::*;

use crate::{
    animation::FadeOut,
    helper::{Fixed, TimeToLive},
    movement::Velocity,
};
use bevy::utils::Duration;

#[derive(Debug, Component)]
pub struct PopupText;

#[derive(Bundle)]
pub struct PopupTextBundle {
    popup_text: PopupText,
    velocity: Velocity,
    fade_out: FadeOut,
    ttl: TimeToLive,
    #[bundle]
    text_bundle: TextBundle,
    fixed: Fixed,
}

pub fn spawn_popup_text(
    commands: &mut Commands,
    font: Handle<Font>,
    pos: Vec2,
    text: impl Into<String>,
    font_size: f32,
    color: Color,
) -> Entity {
    let text: String = text.into();
    let text_bundle = TextBundle::from_section(
        text,
        TextStyle {
            font,
            font_size,
            color,
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        position: UiRect {
            left: Val::Px(pos.x - 2.),
            bottom: Val::Px(pos.y),
            ..default()
        },
        ..default()
    });
    commands
        .spawn(PopupTextBundle {
            popup_text: PopupText,
            fixed: Fixed,
            velocity: Velocity([0., 14.].into()),
            fade_out: FadeOut::new(Duration::from_millis(750)),
            ttl: TimeToLive::new(Duration::from_millis(900)),
            text_bundle,
        })
        .id()
}
