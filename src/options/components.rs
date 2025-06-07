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

/// Component for option light effects
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OptionLightEffect {
    pub base_color: Color,
    pub pulse_speed: f32,
    pub pulse_intensity: f32,
    pub particle_timer: Timer,
    pub glow_radius: f32,
    pub is_correct_answer: bool,
}

impl OptionLightEffect {
    pub fn new(base_color: Color, is_correct: bool) -> Self {
        Self {
            base_color,
            pulse_speed: if is_correct { 3.0 } else { 2.0 }, // Correct answers pulse faster
            pulse_intensity: if is_correct { 0.8 } else { 0.5 },
            particle_timer: Timer::from_seconds(0.3, TimerMode::Repeating),
            glow_radius: if is_correct { 25.0 } else { 20.0 },
            is_correct_answer: is_correct,
        }
    }
}

/// Component for the inner glow effect
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OptionGlow;

/// Component for the outer pulse ring
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OptionPulseRing {
    pub ring_phase: f32,
    pub max_radius: f32,
}

impl OptionPulseRing {
    pub fn new(max_radius: f32) -> Self {
        Self {
            ring_phase: 0.0,
            max_radius,
        }
    }
}

/// Component for sparkle particles around options
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OptionSparkles {
    pub sparkle_timer: Timer,
    pub sparkle_count: usize,
    pub sparkle_intensity: f32, // Controls how often sparkles appear
}

impl OptionSparkles {
    pub fn new(is_correct: bool) -> Self {
        Self {
            sparkle_timer: Timer::from_seconds(
                if is_correct { 0.15 } else { 0.25 }, // Correct answers sparkle more frequently
                TimerMode::Repeating,
            ),
            sparkle_count: if is_correct { 4 } else { 2 }, // More sparkles for correct answers
            sparkle_intensity: if is_correct { 1.0 } else { 0.6 },
        }
    }
}

impl Default for OptionSparkles {
    fn default() -> Self {
        Self::new(false)
    }
}
