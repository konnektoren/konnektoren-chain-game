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
    pub option_id: usize,
    pub base_color: Color,
    pub pulse_phase: f32,
    pub level: u32,
    pub merge_value: u32,
}

impl ChainSegment {
    pub fn new(
        segment_index: usize,
        option_text: String,
        option_id: usize,
        base_color: Color,
    ) -> Self {
        Self {
            segment_index,
            option_text,
            option_id,
            base_color,
            pulse_phase: segment_index as f32 * 0.3,
            level: 1,
            merge_value: 1,
        }
    }

    pub fn get_radius(&self) -> f32 {
        super::CHAIN_SEGMENT_SIZE * (1.0 + (self.level - 1) as f32 * 0.5)
    }
}

/// Component to track the player's movement trail
#[derive(Component, Reflect)]
#[reflect(Component)]
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
            max_trail_length: 1000,
        }
    }
}

impl MovementTrail {
    /// Get position at a specific distance behind the player with wraparound support
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

    /// Get position at a specific distance with wraparound awareness
    pub fn get_position_at_distance_with_wraparound(
        &self,
        distance: f32,
        map_width: f32,
        map_height: f32,
    ) -> Option<Vec2> {
        if self.positions.is_empty() {
            return None;
        }

        let mut accumulated_distance = 0.0;
        let half_width = map_width / 2.0;
        let half_height = map_height / 2.0;

        for i in 0..self.positions.len().saturating_sub(1) {
            let current_pos = self.positions[i];
            let next_pos = self.positions[i + 1];

            // Calculate distance considering wraparound
            let segment_distance =
                calculate_wraparound_distance(current_pos, next_pos, half_width, half_height);

            if accumulated_distance + segment_distance >= distance {
                // Interpolate between current and next position with wraparound
                let remaining_distance = distance - accumulated_distance;
                let t = remaining_distance / segment_distance;
                return Some(interpolate_with_wraparound(
                    current_pos,
                    next_pos,
                    t,
                    half_width,
                    half_height,
                ));
            }

            accumulated_distance += segment_distance;
        }

        // If we've run out of trail, return the oldest position
        self.positions.back().copied()
    }

    /// Add a new position to the trail
    pub fn add_position(&mut self, position: Vec2) {
        // Only add if it's significantly different from the last position
        // But consider wraparound when calculating distance
        if let Some(&last_pos) = self.positions.front() {
            let distance = if self.positions.len() > 1 {
                // Simple distance check for now, could be enhanced with wraparound
                last_pos.distance(position)
            } else {
                last_pos.distance(position)
            };

            if distance < 5.0 {
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

/// Calculate distance between two points considering map wraparound
fn calculate_wraparound_distance(pos1: Vec2, pos2: Vec2, half_width: f32, half_height: f32) -> f32 {
    // Calculate direct distance
    let direct_distance = pos1.distance(pos2);

    // Calculate wraparound distances
    let dx = (pos2.x - pos1.x).abs();
    let dy = (pos2.y - pos1.y).abs();

    let wrap_dx = (half_width * 2.0) - dx;
    let wrap_dy = (half_height * 2.0) - dy;

    // Use the shorter distance in each dimension
    let effective_dx = dx.min(wrap_dx);
    let effective_dy = dy.min(wrap_dy);

    // Return the shorter of direct distance or wraparound distance
    direct_distance.min((effective_dx * effective_dx + effective_dy * effective_dy).sqrt())
}

/// Interpolate between two positions considering wraparound
fn interpolate_with_wraparound(
    pos1: Vec2,
    pos2: Vec2,
    t: f32,
    half_width: f32,
    half_height: f32,
) -> Vec2 {
    let map_width = half_width * 2.0;
    let map_height = half_height * 2.0;

    // Calculate the shortest path for X
    let dx = pos2.x - pos1.x;
    let x = if dx.abs() <= map_width - dx.abs() {
        // Direct path is shorter
        pos1.x + dx * t
    } else {
        // Wraparound path is shorter
        let wrap_dx = if dx > 0.0 {
            dx - map_width
        } else {
            dx + map_width
        };
        let new_x = pos1.x + wrap_dx * t;
        // Handle wraparound
        if new_x > half_width {
            new_x - map_width
        } else if new_x < -half_width {
            new_x + map_width
        } else {
            new_x
        }
    };

    // Calculate the shortest path for Y
    let dy = pos2.y - pos1.y;
    let y = if dy.abs() <= map_height - dy.abs() {
        // Direct path is shorter
        pos1.y + dy * t
    } else {
        // Wraparound path is shorter
        let wrap_dy = if dy > 0.0 {
            dy - map_height
        } else {
            dy + map_height
        };
        let new_y = pos1.y + wrap_dy * t;
        // Handle wraparound
        if new_y > half_height {
            new_y - map_height
        } else if new_y < -half_height {
            new_y + map_height
        } else {
            new_y
        }
    };

    Vec2::new(x, y)
}

/// Component for objects flying to join the chain
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlyingToChain {
    pub start_position: Vec2,
    pub target_position: Vec2,
    pub flight_timer: Timer,
    pub option_text: String,
    pub option_id: usize,
    pub option_color: Color,
    pub curve_height: f32,
}

impl FlyingToChain {
    pub fn new(
        start_pos: Vec2,
        target_pos: Vec2,
        option_text: String,
        option_id: usize,
        option_color: Color,
    ) -> Self {
        Self {
            start_position: start_pos,
            target_position: target_pos,
            flight_timer: Timer::from_seconds(super::FLY_TO_CHAIN_DURATION, TimerMode::Once),
            option_text,
            option_id,
            option_color,
            curve_height: 50.0,
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
    pub option_id: usize,
    pub option_color: Color,
    pub collect_position: Vec2,
}

/// Component for chain segments undergoing reaction
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChainReaction {
    pub reaction_timer: Timer,
    pub reaction_phase: ReactionPhase,
    pub original_scale: f32,
}

impl ChainReaction {
    pub fn new(reaction_duration: f32) -> Self {
        Self {
            reaction_timer: Timer::from_seconds(reaction_duration, TimerMode::Once),
            reaction_phase: ReactionPhase::Reacting,
            original_scale: 1.0,
        }
    }
}

/// Phases of the chain reaction
#[derive(Reflect, Clone, PartialEq)]
pub enum ReactionPhase {
    Reacting,  // Ball is pulsing/glowing before disappearing
    Vanishing, // Ball is disappearing
}

#[derive(Reflect, Clone)]
pub struct PlayerReaction {
    pub player_entity: Entity,
    pub hit_segment_index: usize,
    pub current_spread_distance: i32,
}

/// Resource to manage the chain reaction state
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ChainReactionState {
    pub active_reactions: Vec<PlayerReaction>,
    pub reaction_spread_timer: Timer,
    pub max_spread_distance: i32,
}

impl Default for ChainReactionState {
    fn default() -> Self {
        Self {
            active_reactions: Vec::new(),
            reaction_spread_timer: Timer::from_seconds(
                super::REACTION_SPREAD_INTERVAL,
                TimerMode::Repeating,
            ),
            max_spread_distance: 20,
        }
    }
}

impl ChainReactionState {
    pub fn is_active(&self) -> bool {
        !self.active_reactions.is_empty()
    }

    pub fn start_reaction(&mut self, player_entity: Entity, hit_segment_index: usize) {
        // Remove any existing reaction for this player
        self.active_reactions
            .retain(|r| r.player_entity != player_entity);

        // Add new reaction
        self.active_reactions.push(PlayerReaction {
            player_entity,
            hit_segment_index,
            current_spread_distance: 0,
        });

        self.reaction_spread_timer.reset();
    }

    pub fn remove_completed_reaction(&mut self, player_entity: Entity) {
        self.active_reactions
            .retain(|r| r.player_entity != player_entity);
    }
}

/// Event for when a chain reaction starts
#[derive(Event)]
pub struct ChainReactionEvent {
    pub player_entity: Entity,
    pub hit_segment_index: usize,
}

/// Event for when chain segments are destroyed and points should be deducted
#[allow(dead_code)]
#[derive(Event)]
pub struct ChainSegmentDestroyedEvent {
    pub player_entity: Entity,
    pub segment_index: usize,
    pub option_text: String,
    pub points_lost: i32,
}

/// Component to track which player owns a chain segment
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerChainSegment(pub Entity);

/// Event for when segments should be merged
#[derive(Event)]
pub struct ChainMergeEvent {
    pub player_entity: Entity,
    pub merge_segments: Vec<(Entity, usize)>, // (entity, segment_index)
    pub option_color: Color,
    pub new_level: u32,
}

/// Component for segments undergoing merge animation
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ChainMerging {
    pub merge_timer: Timer,
    pub target_position: Vec3,
    pub original_position: Vec3,
    pub is_target_segment: bool, // The segment that others merge into
}

impl ChainMerging {
    pub fn new(target_pos: Vec3, original_pos: Vec3, is_target: bool) -> Self {
        Self {
            merge_timer: Timer::from_seconds(super::MERGE_ANIMATION_DURATION, TimerMode::Once),
            target_position: target_pos,
            original_position: original_pos,
            is_target_segment: is_target,
        }
    }
}

/// Resource to track merge cooldown to prevent rapid merging
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChainMergeState {
    pub merge_cooldown: Timer,
    pub recent_merges: Vec<(Entity, f32)>, // (player_entity, timestamp)
}

impl ChainMergeState {
    pub fn can_merge(&self, player_entity: Entity, current_time: f32) -> bool {
        if !self.merge_cooldown.finished() {
            return false;
        }

        // Check if this player has merged recently
        !self.recent_merges.iter().any(|(entity, timestamp)| {
            *entity == player_entity && (current_time - timestamp) < super::MERGE_COOLDOWN_DURATION
        })
    }

    pub fn record_merge(&mut self, player_entity: Entity, current_time: f32) {
        self.recent_merges.push((player_entity, current_time));
        self.merge_cooldown.reset();

        // Clean up old merge records
        self.recent_merges.retain(|(_, timestamp)| {
            current_time - timestamp < super::MERGE_COOLDOWN_DURATION * 2.0
        });
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SegmentReindexMarker {
    pub new_index: usize,
}

#[derive(Component)]
pub struct ChainCleanupMarker {
    pub player_entity: Entity,
}
