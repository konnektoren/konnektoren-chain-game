use super::components::*;
use crate::{
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
    player_chain_query: Query<&PlayerChain>,
    mut segment_query: Query<(&ChainSegment, &mut Transform)>,
) {
    for player_chain in &player_chain_query {
        for &segment_entity in &player_chain.segments {
            if let Ok((segment, mut transform)) = segment_query.get_mut(segment_entity) {
                let distance = (segment.segment_index + 1) as f32 * super::CHAIN_SEGMENT_SPACING;

                if let Some(target_position) = movement_trail.get_position_at_distance(distance) {
                    // Smooth movement to target position
                    let current_pos = transform.translation.xy();
                    let new_pos = current_pos.lerp(target_position, 0.15); // Smooth following
                    transform.translation.x = new_pos.x;
                    transform.translation.y = new_pos.y;
                }
            }
        }
    }
}

/// System to animate chain segments (pulsing and gentle floating)
pub fn animate_chain_segments(
    time: Res<Time>,
    mut segment_query: Query<(&mut ChainSegment, &mut Transform)>,
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

        // Apply floating offset (this gets overridden by position updates, so we store base position)
        // We'll modify this to work better with the position system
        let base_translation = Vec3::new(
            transform.translation.x + float_offset_x * 0.3,
            transform.translation.y + float_offset_y * 0.3,
            transform.translation.z,
        );

        // Only apply floating if the segment isn't being actively positioned
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
            // Debug log every few samples
            if movement_trail.positions.len() % 20 == 0 {
                info!(
                    "Movement trail has {} positions, latest: {:?}",
                    movement_trail.positions.len(),
                    position
                );
            }
        }
    }
}
