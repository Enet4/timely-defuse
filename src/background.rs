use bevy::prelude::*;
use bevy_ecs_tilemap::{
    prelude::{
        get_tilemap_center_transform, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize,
        TilemapType,
    },
    tiles::{TilePos, TileStorage, TileTextureIndex},
    TilemapBundle,
};

#[derive(Debug, Component)]
pub struct Background;

use crate::helper::Fixed;

const TILE_SCALE: f32 = 2.;
const TILE_SIZE: u32 = 48;

const TILE_DIVIDE: u32 = 8;

#[derive(Debug, Resource)]
pub struct BackgroundTilesheet(pub Handle<Image>);

pub fn setup(mut commands: Commands, windows: Res<Windows>, asset_server: Res<AssetServer>) {
    // load tilesheet image
    let tex_tilesheet: Handle<Image> = asset_server.load("img/tilesheet.png");

    // save tilesheet for later use
    commands.insert_resource(BackgroundTilesheet(tex_tilesheet.clone()));

    let window = windows.get_primary().unwrap();

    // spawn
    crate::background::spawn_background(&mut commands, window, tex_tilesheet, 0, 1);
}

pub fn spawn_background(
    commands: &mut Commands,
    window: &Window,
    tilesheet: Handle<Image>,
    upper_tile_index: u32,
    lower_tile_index: u32,
) -> Entity {
    // set up tiled background that covers the whole display
    let tilemap_size = TilemapSize {
        x: (window.width() / TILE_SCALE / TILE_SIZE as f32).ceil() as u32 + 2,
        //y: (window.height() / TILE_SIZE as f32).ceil() as u32,
        y: (window.height() / TILE_SCALE / TILE_SIZE as f32).ceil() as u32 + 4,
    };

    // create tilemap entity in advance
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(tilemap_size);

    // Spawn the elements of the tilemap.
    let divide = TILE_DIVIDE;

    bevy_ecs_tilemap::helpers::filling::fill_tilemap_rect(
        TileTextureIndex(lower_tile_index),
        TilePos { x: 0, y: 0 },
        TilemapSize {
            x: tilemap_size.x,
            y: divide,
        },
        TilemapId(tilemap_entity),
        &mut *commands,
        &mut tile_storage,
    );
    bevy_ecs_tilemap::helpers::filling::fill_tilemap_rect(
        TileTextureIndex(upper_tile_index),
        TilePos { x: 0, y: divide },
        TilemapSize {
            x: tilemap_size.x,
            y: tilemap_size.y - divide,
        },
        TilemapId(tilemap_entity),
        &mut *commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 48.0, y: 48.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(tilesheet),
            tile_size,
            transform: get_tilemap_center_transform(&tilemap_size, &grid_size, &map_type, -0.75)
                .with_scale(Vec3::new(TILE_SCALE, TILE_SCALE, 1.)),
            ..Default::default()
        },
        Background,
        Fixed,
    ));

    tilemap_entity
}

pub fn reset_background(
    mut query_background: Query<(Entity, &mut Transform), With<Background>>,
    mut query: Query<(&TilemapId, &TilePos, &mut TileTextureIndex)>,
    upper_tile_index: u32,
    lower_tile_index: u32,
) {
    // traverse the elements of the tilemap, reset texture index
    let divide = TILE_DIVIDE;

    let background_entity = query_background.get_single().unwrap().0;

    for (tile_pos, mut tile_tex_index) in query
        .iter_mut()
        .filter(|(id, ..)| id.0 == background_entity)
        .map(|(_, pos, tex_i)| (pos, tex_i))
    {
        tile_tex_index.0 = if tile_pos.y >= divide {
            upper_tile_index
        } else {
            lower_tile_index
        };
    }

    // reset background translation
    for (_e, mut transform) in &mut query_background {
        transform.translation.x = 0.;
    }
}
