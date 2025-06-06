use bevy::prelude::*;
use std::collections::HashMap;

/// Resource that tracks overall game scoring state
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct GameplayScore {
    pub players: HashMap<Entity, PlayerScore>,
    pub game_active: bool,
    pub game_start_time: f32,
}

impl Default for GameplayScore {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            game_active: true,
            game_start_time: 0.0,
        }
    }
}

impl GameplayScore {
    pub fn add_player(&mut self, player_entity: Entity, player_name: String) {
        self.players
            .insert(player_entity, PlayerScore::new(player_name));
    }

    pub fn get_player_score_mut(&mut self, player_entity: Entity) -> Option<&mut PlayerScore> {
        self.players.get_mut(&player_entity)
    }
}

/// Component and data structure for individual player scores
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct PlayerScore {
    pub player_name: String,
    pub total_score: i32,
    pub correct_answers: u32,
    pub wrong_answers: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub collection_count: u32,
}

impl PlayerScore {
    pub fn new(player_name: String) -> Self {
        Self {
            player_name,
            total_score: 0,
            correct_answers: 0,
            wrong_answers: 0,
            current_streak: 0,
            best_streak: 0,
            collection_count: 0,
        }
    }

    pub fn add_correct_answer(&mut self) {
        self.correct_answers += 1;
        self.current_streak += 1;
        self.collection_count += 1;

        // Calculate score with streak bonus
        let base_points = super::CORRECT_ANSWER_POINTS;
        let streak_bonus = self.current_streak.saturating_sub(1) * super::STREAK_BONUS_MULTIPLIER;
        self.total_score += (base_points + streak_bonus) as i32;

        if self.current_streak > self.best_streak {
            self.best_streak = self.current_streak;
        }
    }

    pub fn add_wrong_answer(&mut self) {
        self.wrong_answers += 1;
        self.current_streak = 0;
        self.collection_count += 1;
        self.total_score = (self.total_score + super::WRONG_ANSWER_PENALTY).max(0);
    }

    pub fn accuracy(&self) -> f32 {
        if self.collection_count == 0 {
            return 0.0;
        }
        (self.correct_answers as f32 / self.collection_count as f32) * 100.0
    }
}

/// Resource for tracking game time
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct GameTimer {
    pub timer: Timer,
    pub game_duration: f32,
    pub time_remaining: f32,
    pub is_overtime: bool,
}

impl Default for GameTimer {
    fn default() -> Self {
        let duration = super::GAME_DURATION_MINUTES * 60.0; // Convert to seconds
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            game_duration: duration,
            time_remaining: duration,
            is_overtime: false,
        }
    }
}

impl GameTimer {
    pub fn time_remaining_formatted(&self) -> String {
        if self.is_overtime {
            let overtime = self.timer.elapsed_secs() - self.game_duration;
            format!(
                "+{:02}:{:02}",
                (overtime / 60.0) as u32,
                (overtime % 60.0) as u32
            )
        } else {
            let remaining = self.time_remaining;
            format!(
                "{:02}:{:02}",
                (remaining / 60.0) as u32,
                (remaining % 60.0) as u32
            )
        }
    }
}

/// Events for score updates
#[derive(Event)]
pub struct ScoreUpdateEvent {
    pub player_entity: Entity,
    pub is_correct: bool,
    pub points_awarded: i32,
}

/// Events for game timer - simplified to only what's used
#[derive(Event)]
pub enum GameTimerEvent {
    GameEnded,
}

/// Component for score display UI
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScoreDisplay;

/// Component for timer display UI
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TimerDisplay;

/// Component for player score display UI
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerScoreDisplay {
    pub player_entity: Entity,
}
