use bevy::prelude::*;

pub struct DynamiteDefusedEvent(pub Entity);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExplosiveKind {
    Dynamite,
    Bomb,
}

pub struct BombDisarmedEvent(pub Entity);

pub struct DisarmProgressEvent(pub f32);

pub struct DisarmCancelledEvent(pub Entity);

#[derive(Debug)]
pub struct ExplodedEvent {
    pub kind: ExplosiveKind,
    pub position: Vec3,
}

#[derive(Debug)]
pub struct GuyHurtEvent {
    pub from: ExplosiveKind,
}

#[derive(Debug, Copy, Clone)]
pub struct DynamiteThrownEvent;

#[derive(Debug, Copy, Clone)]
pub struct BombThrownEvent;

#[derive(Debug, Copy, Clone)]
pub struct CoffeeThrownEvent;

#[derive(Debug, Copy, Clone)]
pub struct WaveFinishedEvent;

#[derive(Debug, Copy, Clone)]
pub struct NextWaveEvent;

#[derive(Debug, Copy, Clone)]
pub struct CoffeePickedUpEvent(pub Entity);

#[derive(Debug, Copy, Clone)]
pub struct CoffeeWornOffEvent;
