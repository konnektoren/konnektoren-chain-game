use bevy::prelude::*;

/// Component for chain explosion effects
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChainExplosionEffect {
    pub lifetime: Timer,
    pub intensity: f32,
}

impl ChainExplosionEffect {
    pub fn new(duration: f32, intensity: f32) -> Self {
        Self {
            lifetime: Timer::from_seconds(duration, TimerMode::Once),
            intensity,
        }
    }
}

/// Component for option collection effects
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CollectionEffect {
    pub lifetime: Timer,
}

impl CollectionEffect {
    pub fn new(duration: f32) -> Self {
        Self {
            lifetime: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Event to spawn explosion effects
#[derive(Event)]
pub struct SpawnExplosionEvent {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
}

/// Event to spawn collection effects
#[derive(Event)]
pub struct SpawnCollectionEvent {
    pub position: Vec3,
    #[allow(dead_code)] // Color is used when particles feature is enabled
    pub color: Color,
}

/// Resource containing pre-built particle effects
#[derive(Resource, Default)]
pub struct ParticleEffects {}
