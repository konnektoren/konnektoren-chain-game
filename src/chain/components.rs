use bevy::prelude::*;
use std::collections::VecDeque;

/// Component for the player's chain system
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerChain {
    pub segments: Vec<Entity>,
    pub max_segments: usize,
}

impl Default for PlayerChain {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
            max_segments: 20, // Maximum chain length
        }
    }
}

/// Component for individual chain segments
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct ChainSegment {
    pub segment_index: usize,
    pub option_text: String,
    pub base_color: Color,
    pub pulse_phase: f32,
}

impl ChainSegment {
    pub fn new(segment_index: usize, option_text: String, base_color: Color) -> Self {
        Self {
            segment_index,
            option_text,
            base_color,
            pulse_phase: segment_index as f32 * 0.3, // Offset pulse phases
        }
    }
}

/// Component to track the player's movement trail
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MovementTrail {
    pub positions: VecDeque<Vec2>,
    pub sample_timer: Timer,
    pub max_trail_length: usize,
}

impl Default for MovementTrail {
    fn default() -> Self {
        Self {
            positions: VecDeque::new(),
            sample_timer: Timer::from_seconds(super::MOVEMENT_SAMPLE_RATE, TimerMode::Repeating),
            max_trail_length: 1000, // Keep plenty of history
        }
    }
}

impl MovementTrail {
    /// Get position at a specific distance behind the player
    pub fn get_position_at_distance(&self, distance: f32) -> Option<Vec2> {
        if self.positions.is_empty() {
            return None;
        }

        let mut accumulated_distance = 0.0;

        for i in 0..self.positions.len().saturating_sub(1) {
            let current_pos = self.positions[i];
            let next_pos = self.positions[i + 1];
            let segment_distance = current_pos.distance(next_pos);

            if accumulated_distance + segment_distance >= distance {
                // Interpolate between current and next position
                let remaining_distance = distance - accumulated_distance;
                let t = remaining_distance / segment_distance;
                return Some(current_pos.lerp(next_pos, t));
            }

            accumulated_distance += segment_distance;
        }

        // If we've run out of trail, return the oldest position
        self.positions.back().copied()
    }

    /// Add a new position to the trail
    pub fn add_position(&mut self, position: Vec2) {
        // Only add if it's significantly different from the last position
        if let Some(&last_pos) = self.positions.front() {
            if last_pos.distance(position) < 5.0 {
                return; // Too close, don't add
            }
        }

        self.positions.push_front(position);

        // Limit trail length
        while self.positions.len() > self.max_trail_length {
            self.positions.pop_back();
        }
    }
}

/// Component for objects flying to join the chain
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlyingToChain {
    pub start_position: Vec2,
    pub target_position: Vec2,
    pub flight_timer: Timer,
    pub option_text: String,
    pub option_color: Color,
    pub curve_height: f32, // For parabolic flight path
}

impl FlyingToChain {
    pub fn new(
        start_pos: Vec2,
        target_pos: Vec2,
        option_text: String,
        option_color: Color,
    ) -> Self {
        Self {
            start_position: start_pos,
            target_position: target_pos,
            flight_timer: Timer::from_seconds(super::FLY_TO_CHAIN_DURATION, TimerMode::Once),
            option_text,
            option_color,
            curve_height: 50.0, // Height of the parabolic arc
        }
    }

    /// Get current position along the flight path (parabolic arc)
    pub fn current_position(&self) -> Vec2 {
        let t = self.flight_timer.fraction();

        // Linear interpolation for x and y
        let linear_pos = self.start_position.lerp(self.target_position, t);

        // Add parabolic curve for y
        let curve_offset = self.curve_height * (4.0 * t * (1.0 - t)); // Parabola peaks at t=0.5

        Vec2::new(linear_pos.x, linear_pos.y + curve_offset)
    }
}

/// Event to extend the chain with a new segment
#[derive(Event)]
pub struct ChainExtendEvent {
    pub player_entity: Entity,
    pub option_text: String,
    pub option_color: Color,
    pub collect_position: Vec2,
}
