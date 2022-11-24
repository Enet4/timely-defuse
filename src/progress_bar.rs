//! progress bar

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    events::{BombDisarmedEvent, DisarmCancelledEvent, DisarmProgressEvent, ExplodedEvent},
    helper::Fixed,
};

#[derive(Bundle)]
pub struct ProgressBarBundle<P>
where
    P: Component,
    P: Default,
{
    part: P,
    fixed: Fixed,
    #[bundle]
    material_mesh_2d: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Default, Component)]
pub struct ProgressBarOuterMesh;

#[derive(Default, Component)]
pub struct ProgressBarInnerMesh;

const PROGRESS_BAR_W: f32 = 30.;
const PROGRESS_BAR_H: f32 = 6.;

pub fn spawn_progress_bar(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> Entity {
    let mesh = Mesh::from(shape::Quad::new(Vec2::from_array([1., 1.])));

    let outer = commands
        .spawn(ProgressBarBundle {
            part: ProgressBarOuterMesh,
            fixed: Fixed,
            material_mesh_2d: MaterialMesh2dBundle {
                transform: Transform::from_translation([0., 15., 0.5].into())
                    .with_scale([PROGRESS_BAR_W, PROGRESS_BAR_H, 1.].into()),
                mesh: meshes.add(mesh).into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                visibility: Visibility { is_visible: false },
                ..default()
            },
        })
        .id();

    let mesh = Mesh::from(shape::Quad::new(Vec2::from_array([1., 1.])));

    let inner = commands
        .spawn(ProgressBarBundle {
            part: ProgressBarInnerMesh,
            fixed: Fixed,
            material_mesh_2d: MaterialMesh2dBundle {
                transform: Transform::from_translation([0., 0., 0.25].into())
                    .with_scale([0., 1., 1.].into()),
                mesh: meshes.add(mesh).into(),
                material: materials.add(ColorMaterial::from(Color::GREEN)),
                ..default()
            },
        })
        .id();

    commands.entity(outer).add_child(inner);

    outer
}

pub fn update_progress_bar(
    mut disarm_progress_event_reader: EventReader<DisarmProgressEvent>,
    mut outer_query: Query<&mut Visibility, With<ProgressBarOuterMesh>>,
    mut inner_query: Query<&mut Transform, With<ProgressBarInnerMesh>>,
) {
    for DisarmProgressEvent(progress) in disarm_progress_event_reader.iter() {
        if let Ok(mut visibility) = outer_query.get_single_mut() {
            visibility.is_visible = true;
        }
        if let Ok(mut transform) = inner_query.get_single_mut() {
            transform.scale.x = *progress;
            transform.translation.x = (progress - 1.) * 0.5;
        }
    }

    disarm_progress_event_reader.clear();
}

/// system: hide progress bar when not disarming
pub fn clear_progress_bar(
    mut disarmed_event_reader: EventReader<BombDisarmedEvent>,
    mut cancelled_event_reader: EventReader<DisarmCancelledEvent>,
    mut exploded_event_reader: EventReader<ExplodedEvent>,
    mut query: Query<&mut Visibility, With<ProgressBarOuterMesh>>,
) {
    for BombDisarmedEvent(_) in disarmed_event_reader.iter() {
        if let Ok(mut visibility) = query.get_single_mut() {
            visibility.is_visible = false;
        }
    }

    for DisarmCancelledEvent(_) in cancelled_event_reader.iter() {
        if let Ok(mut visibility) = query.get_single_mut() {
            visibility.is_visible = false;
        }
    }

    for _ in exploded_event_reader.iter() {
        if let Ok(mut visibility) = query.get_single_mut() {
            visibility.is_visible = false;
        }
    }
}
