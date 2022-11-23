//! module for setting up sound effects/music and configuring it

use bevy::prelude::*;

pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let bomb_explosion: Handle<AudioSource> = asset_server.load("snd/explosion.ogg");

    let click: Handle<AudioSource> = asset_server.load("snd/zipclick.wav");

    // good sound for dynamite explosion
    let thwack1: Handle<AudioSource> = asset_server.load("snd/thwack-01.oga");

    // item touching ground
    let thwack3: Handle<AudioSource> = asset_server.load("snd/thwack-03.oga");

    // heavy item touching ground
    let thwack10: Handle<AudioSource> = asset_server.load("snd/thwack-10.oga");

    // throwing sound
    let woosh: Handle<AudioSource> = asset_server.load("snd/woosh.ogg");

    // drink sound
    let drink: Handle<AudioSource> = asset_server.load("snd/drink.ogg");

    // bomb disarm sound
    let disarm: Handle<AudioSource> = asset_server.load("snd/disarm.ogg");

    commands.insert_resource(GameSoundSources {
        bomb_explosion,
        click,
        thwack1,
        thwack3,
        thwack10,
        woosh,
        drink,
        disarm,
    });
}

#[derive(Resource)]
pub struct GameSoundSources {
    pub bomb_explosion: Handle<AudioSource>,
    pub click: Handle<AudioSource>,
    pub thwack1: Handle<AudioSource>,
    pub thwack3: Handle<AudioSource>,
    pub thwack10: Handle<AudioSource>,
    pub woosh: Handle<AudioSource>,
    pub drink: Handle<AudioSource>,
    pub disarm: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct GameMusic(pub Handle<AudioSource>);

/// A sound that something should make when it bounces off the ground.
#[derive(Component)]
pub struct BounceAudio(pub Handle<AudioSource>);
