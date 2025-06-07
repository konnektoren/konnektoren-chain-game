use bevy::prelude::*;

/// The main player character
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Controller for player movement
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerController {
    pub move_speed: f32,
    pub movement_input: Vec2,
    pub can_move: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_speed: super::PLAYER_MOVE_SPEED,
            movement_input: Vec2::ZERO,
            can_move: true,
        }
    }
}

/// Visual representation of the player
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerVisual;

/// Component to track player statistics
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PlayerStats {
    pub score: u32,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub current_streak: u32,
    pub best_streak: u32,
}

/// Component for player visual effects
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerEffects {
    pub base_color: Color,
    pub glow_intensity: f32,
    pub trail_enabled: bool,
    pub pulse_speed: f32,
    pub energy_level: f32, // 0.0 to 1.0, affects visual intensity
    pub boost_timer: Timer,
    pub is_boosted: bool,
}

impl Default for PlayerEffects {
    fn default() -> Self {
        Self {
            base_color: Color::srgb(1.0, 0.8, 0.2), // Bright yellow
            glow_intensity: 0.8,
            trail_enabled: true,
            pulse_speed: 3.0,
            energy_level: 1.0,
            boost_timer: Timer::from_seconds(2.0, TimerMode::Once),
            is_boosted: false,
        }
    }
}

impl PlayerEffects {
    pub fn boost(&mut self, duration: f32, intensity: f32) {
        self.is_boosted = true;
        self.energy_level = intensity.min(1.0);
        self.boost_timer = Timer::from_seconds(duration, TimerMode::Once);
        self.glow_intensity = 1.2;
        self.pulse_speed = 5.0;
    }

    pub fn get_current_color(&self, time: f32) -> Color {
        if self.is_boosted {
            // Slower rainbow effect to reduce calculation frequency
            let hue = (time * 0.5) % 1.0; // Reduced from 2.0 to 0.5
            Color::hsl(hue * 360.0, 0.8, 0.6)
        } else {
            // Use base color more often to reduce calculations
            let intensity = 0.7 + self.energy_level * 0.3; // Reduced variation
            Color::srgb(
                self.base_color.to_srgba().red * intensity,
                self.base_color.to_srgba().green * intensity,
                self.base_color.to_srgba().blue * intensity,
            )
        }
    }
}

/// Component for player inner glow
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerGlow;

/// Component for player outer aura
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerAura {
    pub aura_phase: f32,
    pub max_radius: f32,
}

impl PlayerAura {
    pub fn new(max_radius: f32) -> Self {
        Self {
            aura_phase: 0.0,
            max_radius,
        }
    }
}

/// Component for player energy particles
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerEnergyParticles {
    pub particle_timer: Timer,
    pub particle_count: usize,
}

impl Default for PlayerEnergyParticles {
    fn default() -> Self {
        Self {
            particle_timer: Timer::from_seconds(0.3, TimerMode::Repeating), // Increased from 0.1
            particle_count: 1,                                              // Reduced from 2
        }
    }
}

/// Component for movement trail particles
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerTrail {
    pub trail_positions: Vec<(Vec3, f32)>, // position and age
    pub max_trail_length: usize,
    pub trail_timer: Timer,
}

impl Default for PlayerTrail {
    fn default() -> Self {
        Self {
            trail_positions: Vec::new(),
            max_trail_length: 10, // Reduced from 20
            trail_timer: Timer::from_seconds(0.1, TimerMode::Repeating), // Increased from 0.05
        }
    }
}

/// Event fired when player collects an option
#[derive(Event)]
pub struct OptionCollectedEvent {
    pub player_entity: Entity,
    pub option_id: usize,
    pub is_correct: bool,
    pub option_text: String,
}

/// Event for player visual feedback
#[derive(Event)]
pub struct PlayerVisualEvent {
    pub player_entity: Entity,
    pub event_type: PlayerVisualEventType,
}

#[derive(Clone, Debug)]
pub enum PlayerVisualEventType {
    CorrectAnswer,
    WrongAnswer,
    Streak(u32),
    Boost { duration: f32, intensity: f32 },
}
