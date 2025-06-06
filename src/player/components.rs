use crate::map::GridPosition;
use bevy::prelude::*;

/// The main player character
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

/// Controller for player movement
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerController {
    pub move_timer: Timer,
    pub movement_input: Vec2,
    pub can_move: bool,
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            move_timer: Timer::from_seconds(super::PLAYER_MOVE_INTERVAL, TimerMode::Repeating),
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
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerStats {
    pub score: u32,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub current_streak: u32,
    pub best_streak: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            score: 0,
            correct_answers: 0,
            wrong_answers: 0,
            current_streak: 0,
            best_streak: 0,
        }
    }
}

/// Event fired when player collects an option
#[derive(Event)]
pub struct OptionCollectedEvent {
    pub option_id: usize,
    pub is_correct: bool,
    pub option_text: String,
}
