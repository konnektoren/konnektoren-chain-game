use super::components::*;
use crate::{
    map::GridMap,
    player::{OptionCollectedEvent, Player},
    screens::Screen,
};
use bevy::prelude::*;

/// System to set up the player chain when entering gameplay
pub fn setup_player_chain(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    info!("Setting up player chain system...");

    let mut player_count = 0;
    for player_entity in &player_query {
        commands
            .entity(player_entity)
            .insert(PlayerChain::default());
        player_count += 1;
        info!(
            "Added PlayerChain component to player entity: {:?}",
            player_entity
        );
    }

    if player_count == 0 {
        warn!("No players found when setting up chain system!");
    }

    // Initialize movement trail resource
    commands.init_resource::<MovementTrail>();
    info!(
        "Player chain system initialized with {} players",
        player_count
    );
}

/// System to update flying objects and convert them to chain segments when they arrive
pub fn update_flying_objects(
    mut commands: Commands,
    time: Res<Time>,
    mut flying_query: Query<(Entity, &mut Transform, &mut FlyingToChain)>,
    mut player_chain_query: Query<&mut PlayerChain>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, mut flying) in &mut flying_query {
        flying.flight_timer.tick(time.delta());

        // Update position along the flight path
        let current_pos = flying.current_position();
        transform.translation.x = current_pos.x;
        transform.translation.y = current_pos.y;

        // Scale effect during flight
        let t = flying.flight_timer.fraction();
        let scale = 1.0 + (t * (1.0 - t) * 4.0) * 0.3; // Slight scale up in the middle
        transform.scale = Vec3::splat(scale);

        // Check if flight is complete
        if flying.flight_timer.finished() {
            // Convert to chain segment
            create_chain_segment(
                &mut commands,
                transform.translation.xy(),
                flying.option_text.clone(),
                flying.option_color,
                &mut player_chain_query,
                &mut meshes,
                &mut materials,
            );

            // Remove the flying object
            commands.entity(entity).despawn();
        }
    }
}

/// Helper function to create a chain segment
fn create_chain_segment(
    commands: &mut Commands,
    position: Vec2,
    option_text: String,
    color: Color,
    player_chain_query: &mut Query<&mut PlayerChain>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Find the player with a chain (should be only one)
    if let Some(mut player_chain) = player_chain_query.iter_mut().next() {
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
                Name::new(format!("Chain Segment: {}", option_text)),
                ChainSegment::new(segment_index, option_text.clone(), color),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_translation(Vec3::new(position.x, position.y, 1.5)),
                StateScoped(Screen::Gameplay),
                children![
                    // Text label for the segment - centered inside the ball
                    (
                        Name::new("Chain Segment Text"),
                        Text2d::new(option_text.clone()),
                        TextFont {
                            font_size: 10.0, // Slightly larger font
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)), // Centered at (0,0)
                    )
                ],
            ))
            .id();

        player_chain.segments.push(segment_entity);
        info!(
            "Created chain segment {} with text: {}",
            segment_index, option_text
        );
    }
}

/// System to update chain segment positions based on the movement trail
pub fn update_chain_positions(
    movement_trail: Res<MovementTrail>,
    grid_map: Option<Res<GridMap>>,
    player_chain_query: Query<&PlayerChain>,
    mut segment_query: Query<(&ChainSegment, &mut Transform), Without<ChainReaction>>,
) {
    let Some(grid_map) = grid_map else {
        return;
    };

    let map_width = grid_map.width as f32 * grid_map.cell_size;
    let map_height = grid_map.height as f32 * grid_map.cell_size;

    for player_chain in &player_chain_query {
        for &segment_entity in &player_chain.segments {
            if let Ok((segment, mut transform)) = segment_query.get_mut(segment_entity) {
                let distance = (segment.segment_index + 1) as f32 * super::CHAIN_SEGMENT_SPACING;

                if let Some(target_position) = movement_trail
                    .get_position_at_distance_with_wraparound(distance, map_width, map_height)
                {
                    // Smooth movement to target position
                    let current_pos = transform.translation.xy();

                    // Calculate the shortest path considering wraparound
                    let new_pos = calculate_shortest_movement(
                        current_pos,
                        target_position,
                        map_width / 2.0,
                        map_height / 2.0,
                        0.15, // lerp factor
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
        // Update pulse phase
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
            continue; // Only correct answers join the chain
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
    player_chain_query: Query<&PlayerChain>,
    movement_trail: Res<MovementTrail>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in chain_events.read() {
        info!("Processing chain extend event for: {}", event.option_text);

        if let Ok(player_chain) = player_chain_query.get(event.player_entity) {
            // Calculate where the new segment should go
            let target_distance =
                (player_chain.segments.len() + 1) as f32 * super::CHAIN_SEGMENT_SPACING;
            let target_position = movement_trail
                .get_position_at_distance(target_distance)
                .unwrap_or(event.collect_position);

            info!(
                "Target position for chain segment: {:?}, distance: {}",
                target_position, target_distance
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
                    event.option_color,
                ),
                StateScoped(Screen::Gameplay),
                children![
                    // Text label for the flying object - centered inside
                    (
                        Name::new("Flying Object Text"),
                        Text2d::new(event.option_text.clone()),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)), // Centered
                    )
                ],
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
    mut movement_trail: ResMut<MovementTrail>,
    player_query: Query<&Transform, With<Player>>,
) {
    movement_trail.sample_timer.tick(time.delta());

    if movement_trail.sample_timer.just_finished() {
        for transform in &player_query {
            let position = transform.translation.xy();
            movement_trail.add_position(position);
        }
    }
}

/// System to detect when player collides with their own chain
pub fn detect_player_chain_collision(
    mut reaction_events: EventWriter<ChainReactionEvent>,
    player_query: Query<(Entity, &Transform, &PlayerChain), With<Player>>,
    segment_query: Query<(&ChainSegment, &Transform), (With<ChainSegment>, Without<Player>)>,
    reaction_state: Res<ChainReactionState>,
) {
    // Don't detect new collisions if a reaction is already active
    if reaction_state.is_active {
        return;
    }

    for (player_entity, player_transform, player_chain) in &player_query {
        let player_pos = player_transform.translation.xy();

        for &segment_entity in &player_chain.segments {
            if let Ok((segment, segment_transform)) = segment_query.get(segment_entity) {
                // Skip collision detection for the first chain element (index 0)
                if segment.segment_index == 0 {
                    continue;
                }

                let segment_pos = segment_transform.translation.xy();
                let distance = player_pos.distance(segment_pos);

                // Check collision (player radius + segment radius)
                let collision_distance = crate::player::PLAYER_SIZE + super::CHAIN_SEGMENT_SIZE;

                if distance <= collision_distance {
                    info!(
                        "Player hit chain segment {} at distance {}",
                        segment.segment_index, distance
                    );

                    reaction_events.write(ChainReactionEvent {
                        player_entity,
                        hit_segment_index: segment.segment_index,
                    });
                    break; // Only trigger one reaction at a time
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
        if !reaction_state.is_active {
            info!(
                "Starting chain reaction at segment {}",
                event.hit_segment_index
            );

            reaction_state.is_active = true;
            reaction_state.player_entity = Some(event.player_entity);
            reaction_state.hit_segment_index = Some(event.hit_segment_index);
            reaction_state.current_spread_distance = 0;
            reaction_state.reaction_spread_timer.reset();
        }
    }
}

/// System to update the chain reaction spread
pub fn update_chain_reaction(
    mut commands: Commands,
    time: Res<Time>,
    mut reaction_state: ResMut<ChainReactionState>,
    player_chain_query: Query<&PlayerChain>,
    segment_query: Query<(Entity, &ChainSegment), (With<ChainSegment>, Without<ChainReaction>)>,
) {
    if !reaction_state.is_active {
        return;
    }

    reaction_state.reaction_spread_timer.tick(time.delta());

    if reaction_state.reaction_spread_timer.just_finished() {
        let Some(hit_index) = reaction_state.hit_segment_index else {
            return;
        };

        let spread_distance = reaction_state.current_spread_distance;
        let mut segments_to_react = Vec::new();

        // Find segments at the current spread distance
        for player_chain in &player_chain_query {
            for &segment_entity in &player_chain.segments {
                if let Ok((entity, segment)) = segment_query.get(segment_entity) {
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
                "Starting reaction on segment at distance {} from hit",
                spread_distance
            );
            commands
                .entity(entity)
                .insert(ChainReaction::new(super::REACTION_BALL_DURATION));
        }

        // Increase spread distance for next iteration
        reaction_state.current_spread_distance += 1;

        // Check if reaction is complete
        if reaction_state.current_spread_distance > reaction_state.max_spread_distance {
            // Check if any segments are still reacting
            let reacting_segments: Vec<_> = segment_query.iter().collect();
            if reacting_segments.is_empty() {
                info!("Chain reaction complete");
                reaction_state.is_active = false;
                reaction_state.hit_segment_index = None;
                reaction_state.current_spread_distance = 0;
            }
        }
    }
}

/// System to animate reacting chain segments
pub fn animate_reacting_segments(
    mut commands: Commands,
    time: Res<Time>,
    mut reacting_query: Query<(Entity, &mut ChainReaction, &mut Transform, &ChainSegment)>,
    mut player_chain_query: Query<&mut PlayerChain>,
    mut destruction_events: EventWriter<ChainSegmentDestroyedEvent>,
    mut explosion_events: EventWriter<crate::effects::SpawnExplosionEvent>, // Add this
    reaction_state: Res<ChainReactionState>,
) {
    for (entity, mut reaction, mut transform, segment) in &mut reacting_query {
        reaction.reaction_timer.tick(time.delta());

        let progress = reaction.reaction_timer.fraction();

        match reaction.reaction_phase {
            ReactionPhase::Reacting => {
                // Pulsing and growing effect
                let pulse_intensity = 1.0 + progress * 2.0; // Grow up to 3x size
                let pulse_frequency = 10.0; // Fast pulsing
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
                let vanish_progress = (progress - 0.6) / 0.4; // Progress within vanishing phase
                let scale = (1.0 - vanish_progress).max(0.0);
                transform.scale = Vec3::splat(scale);
            }
        }

        // Remove segment when reaction is complete
        if reaction.reaction_timer.finished() {
            info!("Removing reacted segment {}", segment.segment_index);

            // Fire destruction event for scoring
            if let Some(player_entity) = reaction_state.player_entity {
                destruction_events.write(ChainSegmentDestroyedEvent {
                    player_entity,
                    segment_index: segment.segment_index,
                    option_text: segment.option_text.clone(),
                    points_lost: crate::chain::POINTS_LOST_PER_SEGMENT,
                });
            }

            // Remove from player chain
            for mut player_chain in &mut player_chain_query {
                player_chain
                    .segments
                    .retain(|&seg_entity| seg_entity != entity);
            }

            // Despawn the entity
            commands.entity(entity).despawn();
        }
    }
}
