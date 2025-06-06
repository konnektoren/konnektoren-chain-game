use bevy::prelude::*;

/// Component for collectible option items on the map
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct OptionCollectible {
    pub option_id: usize,
    pub option_text: String,
    pub is_correct: bool,
    pub spawn_time: f32,
    pub lifetime: f32,
}

impl OptionCollectible {
    pub fn new(option_id: usize, option_text: String, is_correct: bool, lifetime: f32) -> Self {
        Self {
            option_id,
            option_text,
            is_correct,
            spawn_time: 0.0, // Will be set when spawned
            lifetime,
        }
    }

    pub fn is_expired(&self, current_time: f32) -> bool {
        current_time - self.spawn_time > self.lifetime
    }

    pub fn time_remaining(&self, current_time: f32) -> f32 {
        self.lifetime - (current_time - self.spawn_time)
    }
}

/// Timer for spawning option collectibles
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct OptionSpawnTimer {
    pub timer: Timer,
    pub options_per_type: usize,
    pub option_lifetime: f32,
}

impl Default for OptionSpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(super::OPTION_SPAWN_INTERVAL, TimerMode::Repeating),
            options_per_type: super::OPTIONS_PER_TYPE,
            option_lifetime: super::OPTION_LIFETIME,
        }
    }
}

/// Marker component for option visual elements
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OptionVisual;

/// Component to track which option type this collectible represents
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct OptionType {
    pub option_id: usize,
}

impl OptionType {
    pub fn new(option_id: usize) -> Self {
        Self { option_id }
    }
}
