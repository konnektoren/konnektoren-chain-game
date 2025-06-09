use super::MIN_SEGMENTS_TO_MERGE;
use super::components::*;
use crate::{
    map::GridMap,
    player::{OptionCollectedEvent, Player},
    screens::Screen,
};
use bevy::prelude::*;

// Track which player a flying object belongs to
#[derive(Component)]
pub struct FlyingToPlayer(pub Entity);

/// System to set up the player chain when entering gameplay
pub fn setup_player_chain(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    info!("Setting up player chain system...");

    let mut player_count = 0;
    for player_entity in &player_query {
        commands.entity(player_entity).insert((
            PlayerChain::default(),
            MovementTrail::default(), // Add MovementTrail as component instead of resource
        ));
        player_count += 1;
        info!(
            "Added PlayerChain and MovementTrail components to player entity: {:?}",
            player_entity
        );
    }

    if player_count == 0 {
        warn!("No players found when setting up chain system!");
    }

    info!(
        "Player chain system initialized with {} players",
        player_count
    );
}

/// System to update flying objects and convert them to chain segments when they arrive
pub fn update_flying_objects(
    mut commands: Commands,
    time: Res<Time>,
    mut flying_query: Query<(Entity, &mut Transform, &mut FlyingToChain, &FlyingToPlayer)>,
    mut player_query: Query<&mut PlayerChain, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, mut flying, flying_to_player) in &mut flying_query {
        flying.flight_timer.tick(time.delta());

        let current_pos = flying.current_position();
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;

        // Scale effect during flight
        let t = flying.flight_timer.fraction();
        let scale = 1.0 + (t * (1.0 - t) * 4.0) * 0.3;
        transform.scale = Vec3::splat(scale);

        // Check if flight is complete
        if flying.flight_timer.finished() {
            // Convert to chain segment for the specific player
            if let Ok(mut player_chain) = player_query.get_mut(flying_to_player.0) {
                create_chain_segment_for_player(
                    &mut commands,
                    flying_to_player.0,
                    transform.translation.xy(),
                    flying.option_text.clone(),
                    flying.option_id,
                    flying.option_color,
                    &mut player_chain,
                    &mut meshes,
                    &mut materials,
                );
            }

            // Remove the flying object
            commands.entity(entity).despawn();
        }
    }
}

// Create chain segment for specific player
fn create_chain_segment_for_player(
    commands: &mut Commands,
    player_entity: Entity,
    position: Vec2,
    option_text: String,
    option_id: usize,
    color: Color,
    player_chain: &mut PlayerChain,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let segment_index = player_chain.segments.len();

    // Check if we've reached max segments
    if segment_index >= player_chain.max_segments {
        // Remove the oldest segment
        if let Some(oldest_segment) = player_chain.segments.first() {
            commands.entity(*oldest_segment).despawn();
            player_chain.segments.remove(0);
        }
    }

    let mesh = meshes.add(Circle::new(super::CHAIN_SEGMENT_SIZE));
    let material = materials.add(ColorMaterial::from(color));

    let segment_entity = commands
        .spawn((
            Name::new(format!(
                "Chain Segment: {} (Player {:?})",
                option_text, player_entity
            )),
            ChainSegment::new(segment_index, option_text.clone(), option_id, color),
            PlayerChainSegment(player_entity),
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(Vec3::new(position.x, position.y, 1.5)),
            StateScoped(Screen::Gameplay),
            children![(
                Name::new("Chain Segment Text"),
                Text2d::new(option_text.clone()),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
            )],
        ))
        .id();

    player_chain.segments.push(segment_entity);
    info!(
        "Created chain segment {} with text: {} (ID: {}) for player {:?}",
        segment_index, option_text, option_id, player_entity
    );
}

/// System to update chain segment positions based on the movement trail
pub fn update_chain_positions(
    grid_map: Option<Res<GridMap>>,
    mut player_query: Query<(Entity, &PlayerChain, &MovementTrail), With<Player>>,
    mut segment_query: Query<(&ChainSegment, &mut Transform), Without<ChainReaction>>,
) {
    let Some(grid_map) = grid_map else {
        return;
    };

    for (_player_entity, player_chain, movement_trail) in &mut player_query {
        for &segment_entity in &player_chain.segments {
            if let Ok((segment, mut transform)) = segment_query.get_mut(segment_entity) {
                let distance = (segment.segment_index + 1) as f32 * super::CHAIN_SEGMENT_SPACING;

                if let Some(target_position) = movement_trail
                    .get_position_at_distance_with_wraparound(
                        distance,
                        grid_map.world_width(),
                        grid_map.world_height(),
                    )
                {
                    let current_pos = transform.translation.xy();
                    let new_pos = calculate_shortest_movement(
                        current_pos,
                        target_position,
                        grid_map.half_width(),
                        grid_map.half_height(),
                        0.15,
                    );

                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                }
            }
        }
    }
}

/// Calculate the shortest movement path considering wraparound
fn calculate_shortest_movement(
    current: Vec2,
    target: Vec2,
    half_width: f32,
    half_height: f32,
    lerp_factor: f32,
) -> Vec2 {
    let map_width = half_width * 2.0;
    let map_height = half_height * 2.0;

    // Calculate direct movement
    let direct_movement = current.lerp(target, lerp_factor);

    // Calculate wraparound movement for X
    let dx = target.x - current.x;
    let wrap_target_x = if dx > half_width {
        target.x - map_width
    } else if dx < -half_width {
        target.x + map_width
    } else {
        target.x
    };

    // Calculate wraparound movement for Y
    let dy = target.y - current.y;
    let wrap_target_y = if dy > half_height {
        target.y - map_height
    } else if dy < -half_height {
        target.y + map_height
    } else {
        target.y
    };

    let wrap_target = Vec2::new(wrap_target_x, wrap_target_y);
    let wrap_movement = current.lerp(wrap_target, lerp_factor);

    // Choose the movement that results in shorter distance
    if current.distance(direct_movement) <= current.distance(wrap_movement) {
        direct_movement
    } else {
        // Apply wraparound if needed
        let mut result = wrap_movement;
        if result.x > half_width {
            result.x -= map_width;
        } else if result.x < -half_width {
            result.x += map_width;
        }
        if result.y > half_height {
            result.y -= map_height;
        } else if result.y < -half_height {
            result.y += map_height;
        }
        result
    }
}

/// System to animate chain segments (pulsing and gentle floating)
pub fn animate_chain_segments(
    time: Res<Time>,
    mut segment_query: Query<(&mut ChainSegment, &mut Transform), Without<ChainReaction>>, // Exclude reacting segments
) {
    let time_factor = time.elapsed_secs();

    for (mut segment, mut transform) in &mut segment_query {
        segment.pulse_phase += time.delta_secs() * 2.0;

        // Pulsing scale effect
        let pulse = 1.0 + (segment.pulse_phase.sin() * 0.15);
        transform.scale = Vec3::splat(pulse);

        // Gentle floating motion
        let float_offset_x = (time_factor * 0.8 + segment.segment_index as f32 * 0.5).sin() * 2.0;
        let float_offset_y = (time_factor * 1.2 + segment.segment_index as f32 * 0.7).cos() * 1.5;

        // Apply floating offset
        let base_translation = Vec3::new(
            transform.translation.x + float_offset_x * 0.3,
            transform.translation.y + float_offset_y * 0.3,
            transform.translation.z,
        );

        transform.translation = base_translation;
    }
}

/// System to handle option collection and start the fly-to-chain animation
pub fn handle_chain_extend_events(
    mut collection_events: EventReader<OptionCollectedEvent>,
    mut chain_events: EventWriter<ChainExtendEvent>,
    player_query: Query<&Transform, With<Player>>,
) {
    for event in collection_events.read() {
        info!(
            "Chain system received collection event: {} (correct: {})",
            event.option_text, event.is_correct
        );

        if !event.is_correct {
            info!("Skipping incorrect answer for chain");
            continue;
        }

        // Get player position for the collect position
        if let Ok(player_transform) = player_query.get(event.player_entity) {
            let collect_position = player_transform.translation.xy();

            // Choose color based on option ID (similar to options system)
            let base_colors = [
                Color::srgb(0.3, 0.5, 0.8), // Blue
                Color::srgb(0.8, 0.5, 0.3), // Orange
                Color::srgb(0.5, 0.8, 0.3), // Green
                Color::srgb(0.8, 0.3, 0.5), // Pink
                Color::srgb(0.5, 0.3, 0.8), // Purple
            ];
            let color = base_colors[event.option_id % base_colors.len()];

            info!("Creating chain extend event for: {}", event.option_text);

            chain_events.write(ChainExtendEvent {
                player_entity: event.player_entity,
                option_text: event.option_text.clone(),
                option_id: event.option_id,
                option_color: color,
                collect_position,
            });
        } else {
            warn!("Could not find player entity for chain extend event");
        }
    }
}

/// System to handle chain extend events and create flying objects
pub fn create_flying_to_chain_objects(
    mut commands: Commands,
    mut chain_events: EventReader<ChainExtendEvent>,
    player_query: Query<(&PlayerChain, &MovementTrail), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in chain_events.read() {
        info!("Processing chain extend event for: {}", event.option_text);

        if let Ok((player_chain, movement_trail)) = player_query.get(event.player_entity) {
            // Calculate where the new segment should go for THIS player
            let target_distance =
                (player_chain.segments.len() + 1) as f32 * super::CHAIN_SEGMENT_SPACING;
            let target_position = movement_trail
                .get_position_at_distance(target_distance)
                .unwrap_or(event.collect_position);

            info!(
                "Target position for chain segment on player {:?}: {:?}, distance: {}",
                event.player_entity, target_position, target_distance
            );

            // Create the flying object
            let mesh = meshes.add(Circle::new(super::CHAIN_SEGMENT_SIZE));
            let material = materials.add(ColorMaterial::from(event.option_color));

            commands.spawn((
                Name::new(format!("Flying to Chain: {}", event.option_text)),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_translation(Vec3::new(
                    event.collect_position.x,
                    event.collect_position.y,
                    3.0,
                )),
                FlyingToChain::new(
                    event.collect_position,
                    target_position,
                    event.option_text.clone(),
                    event.option_id,
                    event.option_color,
                ),
                FlyingToPlayer(event.player_entity),
                StateScoped(Screen::Gameplay),
                children![(
                    Name::new("Flying Object Text"),
                    Text2d::new(event.option_text.clone()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                )],
            ));

            info!("Started fly-to-chain animation for: {}", event.option_text);
        } else {
            warn!(
                "Could not find player chain for entity: {:?}",
                event.player_entity
            );
        }
    }
}

/// System to track player movement and build the trail
pub fn track_player_movement(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut MovementTrail), With<Player>>,
) {
    for (transform, mut movement_trail) in &mut player_query {
        movement_trail.sample_timer.tick(time.delta());

        if movement_trail.sample_timer.just_finished() {
            let position = transform.translation.xy();
            movement_trail.add_position(position);
        }
    }
}

pub fn detect_player_chain_collision(
    mut reaction_events: EventWriter<ChainReactionEvent>,
    player_query: Query<(Entity, &Transform, &PlayerChain), With<Player>>,
    segment_query: Query<
        (&ChainSegment, &Transform, &PlayerChainSegment),
        (With<ChainSegment>, Without<Player>),
    >,
    reaction_state: Res<ChainReactionState>,
) {
    for (player_entity, player_transform, player_chain) in &player_query {
        // Check if this player already has an active reaction
        if reaction_state
            .active_reactions
            .iter()
            .any(|r| r.player_entity == player_entity)
        {
            continue;
        }

        let player_pos = player_transform.translation.xy();

        for &segment_entity in &player_chain.segments {
            if let Ok((segment, segment_transform, segment_owner)) =
                segment_query.get(segment_entity)
            {
                // Only check collision with this player's own segments
                if segment_owner.0 != player_entity {
                    continue;
                }

                // Skip collision detection for the first chain element
                if segment.segment_index == 0 {
                    continue;
                }

                let segment_pos = segment_transform.translation.xy();
                let distance = player_pos.distance(segment_pos);
                let collision_distance = crate::player::PLAYER_SIZE + super::CHAIN_SEGMENT_SIZE;

                if distance <= collision_distance {
                    info!(
                        "Player {:?} hit their own chain segment {} at distance {}",
                        player_entity, segment.segment_index, distance
                    );

                    reaction_events.write(ChainReactionEvent {
                        player_entity,
                        hit_segment_index: segment.segment_index,
                    });
                    break;
                }
            }
        }
    }
}

/// System to handle chain reaction events
pub fn handle_chain_reaction_events(
    mut reaction_events: EventReader<ChainReactionEvent>,
    mut reaction_state: ResMut<ChainReactionState>,
) {
    for event in reaction_events.read() {
        if !reaction_state.is_active()
            || reaction_state
                .active_reactions
                .iter()
                .all(|r| r.player_entity != event.player_entity)
        {
            info!(
                "Starting chain reaction at segment {} for player {:?}",
                event.hit_segment_index, event.player_entity
            );

            reaction_state.start_reaction(event.player_entity, event.hit_segment_index);
        }
    }
}

/// System to update the chain reaction spread
pub fn update_chain_reaction(
    mut commands: Commands,
    time: Res<Time>,
    mut reaction_state: ResMut<ChainReactionState>,
    player_chain_query: Query<(Entity, &PlayerChain), With<Player>>,
    segment_query: Query<
        (Entity, &ChainSegment, &PlayerChainSegment),
        (With<ChainSegment>, Without<ChainReaction>),
    >,
) {
    if !reaction_state.is_active() {
        return;
    }

    reaction_state.reaction_spread_timer.tick(time.delta());

    if reaction_state.reaction_spread_timer.just_finished() {
        let mut reactions_to_remove = Vec::new();

        // Extract max_spread_distance before the mutable borrow
        let max_spread_distance = reaction_state.max_spread_distance;

        // Process each active reaction
        for reaction in &mut reaction_state.active_reactions {
            let hit_index = reaction.hit_segment_index;
            let spread_distance = reaction.current_spread_distance;
            let player_entity = reaction.player_entity;
            let mut segments_to_react = Vec::new();

            // Find this player's chain
            if let Some((_, player_chain)) = player_chain_query
                .iter()
                .find(|(entity, _)| *entity == player_entity)
            {
                // Find segments at the current spread distance for this specific player
                for &segment_entity in &player_chain.segments {
                    if let Ok((entity, segment, segment_owner)) = segment_query.get(segment_entity)
                    {
                        // Only affect this player's segments
                        if segment_owner.0 != player_entity {
                            continue;
                        }

                        let segment_distance_from_hit =
                            (segment.segment_index as i32 - hit_index as i32).abs();

                        if segment_distance_from_hit == spread_distance {
                            segments_to_react.push(entity);
                        }
                    }
                }
            }

            // Add ChainReaction component to segments that should start reacting
            for entity in segments_to_react {
                info!(
                    "Starting reaction on segment at distance {} from hit for player {:?}",
                    spread_distance, player_entity
                );
                commands
                    .entity(entity)
                    .insert(ChainReaction::new(super::REACTION_BALL_DURATION));
            }

            // Increase spread distance for next iteration
            reaction.current_spread_distance += 1;

            // Check if this reaction is complete - use the extracted value
            if reaction.current_spread_distance > max_spread_distance {
                // Check if any segments are still reacting for this player
                let player_reacting_segments: Vec<_> = segment_query
                    .iter()
                    .filter(|(_, _, segment_owner)| segment_owner.0 == player_entity)
                    .collect();

                if player_reacting_segments.is_empty() {
                    info!("Chain reaction complete for player {:?}", player_entity);
                    reactions_to_remove.push(player_entity);
                }
            }
        }

        // Remove completed reactions
        for player_entity in reactions_to_remove {
            reaction_state.remove_completed_reaction(player_entity);
        }
    }
}

/// System to animate reacting chain segments
pub fn animate_reacting_segments(
    mut commands: Commands,
    time: Res<Time>,
    mut reacting_query: Query<(
        Entity,
        &mut ChainReaction,
        &mut Transform,
        &ChainSegment,
        &PlayerChainSegment,
    )>,
    mut player_chain_query: Query<(Entity, &mut PlayerChain), With<Player>>,
    mut destruction_events: EventWriter<ChainSegmentDestroyedEvent>,
    mut explosion_events: EventWriter<crate::effects::SpawnExplosionEvent>,
) {
    for (entity, mut reaction, mut transform, segment, segment_owner) in &mut reacting_query {
        reaction.reaction_timer.tick(time.delta());

        let progress = reaction.reaction_timer.fraction();

        match reaction.reaction_phase {
            ReactionPhase::Reacting => {
                // Pulsing and growing effect
                let pulse_intensity = 1.0 + progress * 2.0;
                let pulse_frequency = 10.0;
                let pulse =
                    pulse_intensity * (1.0 + (time.elapsed_secs() * pulse_frequency).sin() * 0.3);

                transform.scale = Vec3::splat(pulse);

                // Change to vanishing phase partway through
                if progress > 0.6 {
                    reaction.reaction_phase = ReactionPhase::Vanishing;
                    info!("Segment {} entering vanishing phase", segment.segment_index);

                    // Spawn explosion effect when entering vanishing phase
                    explosion_events.write(crate::effects::SpawnExplosionEvent {
                        position: transform.translation,
                        color: segment.base_color,
                        intensity: 1.0,
                    });
                }
            }
            ReactionPhase::Vanishing => {
                // Shrinking effect
                let vanish_progress = (progress - 0.6) / 0.4;
                let scale = (1.0 - vanish_progress).max(0.0);
                transform.scale = Vec3::splat(scale);
            }
        }

        // Remove segment when reaction is complete
        if reaction.reaction_timer.finished() {
            info!("Removing reacted segment {}", segment.segment_index);

            let player_entity = segment_owner.0;

            // Fire destruction event for scoring
            destruction_events.write(ChainSegmentDestroyedEvent {
                player_entity,
                segment_index: segment.segment_index,
                option_text: segment.option_text.clone(),
                points_lost: crate::chain::POINTS_LOST_PER_SEGMENT,
            });

            // Remove from the correct player's chain
            if let Ok((_, mut player_chain)) = player_chain_query.get_mut(player_entity) {
                player_chain
                    .segments
                    .retain(|&seg_entity| seg_entity != entity);
            }

            // Despawn the entity
            commands.entity(entity).despawn();
        }
    }
}

/// System to detect when 3 consecutive segments of the same type can be merged
pub fn detect_chain_merges(
    time: Res<Time>,
    mut merge_events: EventWriter<ChainMergeEvent>,
    merge_state: Res<ChainMergeState>,
    player_query: Query<(Entity, &PlayerChain), With<Player>>,
    segment_query: Query<
        (Entity, &ChainSegment, &PlayerChainSegment),
        (
            With<ChainSegment>,
            Without<ChainMerging>,
            Without<ChainReaction>,
        ),
    >,
) {
    let current_time = time.elapsed_secs();

    for (player_entity, player_chain) in &player_query {
        // Check if this player can merge (cooldown check)
        if !merge_state.can_merge(player_entity, current_time) {
            continue;
        }

        // Look for sequences of 3+ consecutive segments with same option_id
        let segments_data: Vec<_> = player_chain
            .segments
            .iter()
            .filter_map(|&segment_entity| {
                segment_query
                    .get(segment_entity)
                    .ok()
                    .map(|(entity, segment, owner)| (entity, segment.clone(), owner.0))
            })
            .filter(|(_, _, owner)| *owner == player_entity)
            .collect();

        // Check for mergeable sequences
        for window_start in 0..segments_data
            .len()
            .saturating_sub(MIN_SEGMENTS_TO_MERGE - 1)
        {
            let window = &segments_data[window_start..window_start + MIN_SEGMENTS_TO_MERGE];

            // Check if all segments in window have same option_id and are level 1
            let first_segment = &window[0].1;
            let can_merge = window.iter().all(|(_, segment, _)| {
                segment.option_id == first_segment.option_id
                    && segment.level == first_segment.level
                    && segment.level < 3 // Don't merge beyond level 3
            });

            if can_merge {
                let merge_segments: Vec<_> = window
                    .iter()
                    .map(|(entity, segment, _)| (*entity, segment.segment_index))
                    .collect();

                info!(
                    "Detected mergeable sequence for player {:?}: {} segments of type '{}'",
                    player_entity, MIN_SEGMENTS_TO_MERGE, first_segment.option_text
                );

                merge_events.write(ChainMergeEvent {
                    player_entity,
                    merge_segments,
                    option_color: first_segment.base_color,
                    new_level: first_segment.level + 1,
                });

                // Only trigger one merge per detection cycle per player
                break;
            }
        }
    }
}

/// System to handle chain merge events
pub fn handle_chain_merge_events(
    mut commands: Commands,
    mut merge_events: EventReader<ChainMergeEvent>,
    mut merge_state: ResMut<ChainMergeState>,
    player_query: Query<&PlayerChain, With<Player>>,
    segment_query: Query<&Transform, With<ChainSegment>>,
    time: Res<Time>,
) {
    for event in merge_events.read() {
        let Ok(_player_chain) = player_query.get(event.player_entity) else {
            continue;
        };

        // Record the merge
        merge_state.record_merge(event.player_entity, time.elapsed_secs());

        // Find the middle segment to be the target (others will merge into it)
        let target_index = event.merge_segments.len() / 2;
        let (target_entity, _target_segment_index) = event.merge_segments[target_index];

        let target_transform = segment_query.get(target_entity).ok();

        // Start merge animation for all segments
        for (i, &(segment_entity, _)) in event.merge_segments.iter().enumerate() {
            let is_target = i == target_index;

            if let Ok(transform) = segment_query.get(segment_entity) {
                let target_pos = if is_target {
                    transform.translation
                } else if let Some(target_transform) = target_transform {
                    target_transform.translation
                } else {
                    transform.translation
                };

                commands.entity(segment_entity).insert(ChainMerging::new(
                    target_pos,
                    transform.translation,
                    is_target,
                ));
            }
        }

        // Create the new merged segment
        let new_level = event.new_level;
        let merge_value = event.merge_segments.len() as u32;

        // Enhanced visual for merged segments
        let _enhanced_color = enhance_color_for_level(event.option_color, new_level);

        info!(
            "Starting merge animation for player {:?}: {} segments -> Level {} (value: {})",
            event.player_entity, merge_value, new_level, merge_value
        );
    }
}

/// System to animate merging segments
pub fn animate_merging_segments(
    mut commands: Commands,
    time: Res<Time>,
    mut merging_query: Query<(
        Entity,
        &mut ChainMerging,
        &mut Transform,
        &ChainSegment,
        &PlayerChainSegment,
    )>,
    _player_query: Query<&PlayerChain, With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut completed_merges: Vec<(Entity, ChainSegment, Entity, Vec3)> = Vec::new();
    let mut entities_to_despawn: Vec<Entity> = Vec::new();

    for (entity, mut merging, mut transform, segment, segment_owner) in &mut merging_query {
        merging.merge_timer.tick(time.delta());

        let progress = merging.merge_timer.fraction();

        if merging.is_target_segment {
            // Target segment grows and gets enhanced visuals
            let scale = 1.0 + progress * 0.5; // Grow by 50%
            transform.scale = Vec3::splat(scale);

            // Enhanced pulsing effect
            let pulse = 1.0 + (time.elapsed_secs() * 6.0).sin() * 0.3 * progress;
            transform.scale *= pulse;
        } else {
            // Other segments shrink and move toward target
            let scale = 1.0 - progress * 0.8; // Shrink to 20%
            transform.scale = Vec3::splat(scale.max(0.1));

            // Move toward target
            transform.translation = merging
                .original_position
                .lerp(merging.target_position, progress);
        }

        // Check if animation is complete
        if merging.merge_timer.finished() {
            if merging.is_target_segment {
                // Convert target to merged segment
                let mut new_segment = segment.clone();
                new_segment.level += 1;
                new_segment.merge_value = 3; // For now, assume we're always merging 3

                completed_merges.push((
                    segment_owner.0,
                    new_segment,
                    entity,
                    transform.translation,
                ));
            } else {
                // Mark non-target segments for removal
                entities_to_despawn.push(entity);
            }
        }
    }

    // First, despawn the non-target entities
    for entity in entities_to_despawn {
        commands.entity(entity).despawn();
    }

    // Then process completed merges
    for (player_entity, new_segment_data, target_entity, merge_position) in completed_merges {
        // Update the target entity with new merged data and visuals
        let new_radius = new_segment_data.get_radius();
        let enhanced_color =
            enhance_color_for_level(new_segment_data.base_color, new_segment_data.level);

        let new_mesh = meshes.add(Circle::new(new_radius));
        let new_material = materials.add(ColorMaterial::from(enhanced_color));

        commands
            .entity(target_entity)
            .remove::<ChainMerging>()
            .insert((
                new_segment_data.clone(),
                Mesh2d(new_mesh),
                MeshMaterial2d(new_material),
            ));

        // Update player chain - we'll do this via a marker component to avoid command conflicts
        commands
            .entity(target_entity)
            .insert(ChainCleanupMarker { player_entity });

        info!(
            "Completed merge for player {:?}: Created level {} segment (radius: {:.1})",
            player_entity, new_segment_data.level, new_radius
        );

        // Spawn merge effect
        commands.spawn((
            Name::new("Merge Effect"),
            Transform::from_translation(Vec3::new(merge_position.x, merge_position.y, 5.0)),
            // Add particle effect here if desired
        ));
    }
}

/// System to update merge cooldown timer
pub fn update_merge_cooldown(time: Res<Time>, mut merge_state: ResMut<ChainMergeState>) {
    merge_state.merge_cooldown.tick(time.delta());
}

/// Helper function to enhance colors for higher level segments
fn enhance_color_for_level(base_color: Color, level: u32) -> Color {
    match level {
        1 => base_color,
        2 => {
            // Add golden tint for level 2
            let rgba = base_color.to_srgba();
            Color::srgb(
                (rgba.red + 0.3).min(1.0),
                (rgba.green + 0.2).min(1.0),
                rgba.blue,
            )
        }
        3 => {
            // Add silver/platinum tint for level 3
            let rgba = base_color.to_srgba();
            Color::srgb(
                (rgba.red + 0.4).min(1.0),
                (rgba.green + 0.4).min(1.0),
                (rgba.blue + 0.2).min(1.0),
            )
        }
        _ => {
            // Rainbow effect for level 4+
            let hue = (level as f32 * 60.0) % 360.0;
            Color::hsl(hue, 0.8, 0.7)
        }
    }
}

/// System to clean up and reindex chains after merges
pub fn cleanup_merged_chains(
    mut commands: Commands,
    cleanup_query: Query<(Entity, &ChainCleanupMarker)>,
    mut player_query: Query<&mut PlayerChain, With<Player>>,
    segment_query: Query<Entity, With<ChainSegment>>,
) {
    for (marker_entity, cleanup_marker) in &cleanup_query {
        let player_entity = cleanup_marker.player_entity;

        if let Ok(mut player_chain) = player_query.get_mut(player_entity) {
            // Filter out entities that no longer exist
            let original_segments = player_chain.segments.clone();
            player_chain.segments.clear();

            for segment_entity in original_segments {
                // Check if the entity still exists by trying to get it
                if segment_query.get(segment_entity).is_ok() {
                    player_chain.segments.push(segment_entity);
                }
            }

            // Mark segments for reindexing
            for (new_index, &segment_entity) in player_chain.segments.iter().enumerate() {
                commands
                    .entity(segment_entity)
                    .insert(SegmentReindexMarker { new_index });
            }

            info!(
                "Cleaned up chain for player {:?}, {} segments remaining",
                player_entity,
                player_chain.segments.len()
            );
        }

        // Remove the cleanup marker
        commands
            .entity(marker_entity)
            .remove::<ChainCleanupMarker>();
    }
}

/// System to handle segment reindexing
pub fn handle_segment_reindexing(
    mut commands: Commands,
    mut reindex_query: Query<(Entity, &SegmentReindexMarker, &mut ChainSegment)>,
) {
    for (entity, marker, mut segment) in &mut reindex_query {
        segment.segment_index = marker.new_index;
        commands.entity(entity).remove::<SegmentReindexMarker>();
    }
}
