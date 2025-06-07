use super::components::*;
use crate::{
    input::{InputController, PlayerInputMapping},
    map::{GridMap, GridPosition},
    options::{OptionCollectible, OptionType},
    screens::Screen,
};
use bevy::prelude::*;

/// System to spawn the player at the center of the grid
pub fn spawn_player(
    mut commands: Commands,
    grid_map: Option<Res<GridMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(grid_map) = grid_map else {
        warn!("GridMap not available when trying to spawn player");
        return;
    };

    // Spawn player at center of grid
    let center_x = grid_map.width / 2;
    let center_y = grid_map.height / 2;
    let grid_pos = GridPosition::new(center_x, center_y);
    let world_pos = grid_map.grid_to_world(center_x, center_y);

    // Create player visual (a bright colored circle)
    let mesh = meshes.add(Circle::new(super::PLAYER_SIZE));
    let material = materials.add(ColorMaterial::from(Color::srgb(1.0, 0.8, 0.2))); // Bright yellow

    commands.spawn((
        Name::new("Player"),
        Player,
        PlayerController::default(),
        PlayerStats::default(),
        PlayerVisual,
        InputController::default(),    // Add input controller
        PlayerInputMapping::default(), // Add input mapping
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(world_pos.x, world_pos.y, 2.0)), // Higher Z than options
        grid_pos,
        StateScoped(Screen::Gameplay),
    ));

    info!(
        "Player spawned at grid position ({}, {})",
        center_x, center_y
    );
}

/// System to handle player input using the new input system
pub fn handle_player_input(
    mut player_query: Query<
        (&mut PlayerController, &InputController),
        (With<Player>, Changed<InputController>),
    >,
) {
    for (mut controller, input_controller) in &mut player_query {
        // Only accept input if player can move
        if !controller.can_move {
            controller.movement_input = Vec2::ZERO;
            continue;
        }

        // Use input from the InputController
        controller.movement_input = input_controller.movement_input;
    }
}

/// System to move the player smoothly with wraparound at borders
pub fn move_player(
    time: Res<Time>,
    grid_map: Option<Res<GridMap>>,
    mut player_query: Query<(&PlayerController, &mut GridPosition, &mut Transform), With<Player>>,
) {
    let Some(grid_map) = grid_map else {
        return;
    };

    for (controller, mut grid_pos, mut transform) in &mut player_query {
        if controller.movement_input == Vec2::ZERO {
            continue;
        }

        // Calculate movement delta
        let movement_delta = controller.movement_input * controller.move_speed * time.delta_secs();

        // Update world position
        let new_world_pos = Vec2::new(
            transform.translation.x + movement_delta.x,
            transform.translation.y + movement_delta.y,
        );

        // Calculate grid bounds in world coordinates
        let half_width = (grid_map.width as f32 * grid_map.cell_size) / 2.0;
        let half_height = (grid_map.height as f32 * grid_map.cell_size) / 2.0;

        // Handle wraparound - teleport to opposite side when crossing borders
        let wrapped_world_pos = handle_map_wraparound(new_world_pos, half_width, half_height);

        // Update transform
        transform.translation.x = wrapped_world_pos.x;
        transform.translation.y = wrapped_world_pos.y;

        // Update grid position based on current world position
        if let Some((grid_x, grid_y)) = grid_map.world_to_grid(wrapped_world_pos) {
            grid_pos.x = grid_x;
            grid_pos.y = grid_y;
        }
    }
}

/// Handle map wraparound when player crosses borders
fn handle_map_wraparound(position: Vec2, half_width: f32, half_height: f32) -> Vec2 {
    let mut wrapped_pos = position;

    // Handle horizontal wraparound
    if wrapped_pos.x > half_width {
        wrapped_pos.x = -half_width + (wrapped_pos.x - half_width);
    } else if wrapped_pos.x < -half_width {
        wrapped_pos.x = half_width + (wrapped_pos.x + half_width);
    }

    // Handle vertical wraparound
    if wrapped_pos.y > half_height {
        wrapped_pos.y = -half_height + (wrapped_pos.y - half_height);
    } else if wrapped_pos.y < -half_height {
        wrapped_pos.y = half_height + (wrapped_pos.y + half_height);
    }

    wrapped_pos
}

/// System to handle option collection with smooth movement
pub fn collect_options(
    mut commands: Commands,
    mut event_writer: EventWriter<OptionCollectedEvent>,
    mut collection_effects: EventWriter<crate::effects::SpawnCollectionEvent>, // Add this
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    option_query: Query<
        (Entity, &Transform, &OptionCollectible, &OptionType),
        (Without<Player>, With<crate::options::OptionVisual>),
    >,
) {
    for (player_entity, player_transform) in &mut player_query {
        for (option_entity, option_transform, collectible, option_type) in &option_query {
            // Calculate distance between player and option
            let distance = player_transform
                .translation
                .xy()
                .distance(option_transform.translation.xy());

            // Collection radius (player size + option size)
            let collection_radius = super::PLAYER_SIZE + 14.0; // Option size is 14.0

            if distance <= collection_radius {
                // Spawn collection effect
                collection_effects.write(crate::effects::SpawnCollectionEvent {
                    position: option_transform.translation,
                    color: Color::from(if collectible.is_correct {
                        // Use a bright green tint for correct answers
                        bevy::color::palettes::css::GREEN_YELLOW
                    } else {
                        // Use a bright red tint for incorrect answers
                        bevy::color::palettes::css::ORANGE_RED
                    }),
                });

                // Send collection event
                event_writer.write(OptionCollectedEvent {
                    player_entity,
                    option_id: option_type.option_id,
                    is_correct: collectible.is_correct,
                    option_text: collectible.option_text.clone(),
                });

                // Remove the collected option
                commands.entity(option_entity).despawn();

                info!("Player collected option: {}", collectible.option_text);
            }
        }
    }
}

/// System to animate player (pulsing effect and rotation based on movement)
pub fn animate_player(
    time: Res<Time>,
    mut player_query: Query<
        (&PlayerController, &mut Transform),
        (With<Player>, With<PlayerVisual>),
    >,
) {
    let time_factor = time.elapsed_secs() * 4.0; // Faster animation than options

    for (controller, mut transform) in &mut player_query {
        // Base pulsing effect
        let pulse = 1.0 + (time_factor.sin() * 0.1);

        // Scale up slightly when moving
        let movement_scale = if controller.movement_input.length() > 0.1 {
            1.1
        } else {
            1.0
        };

        transform.scale = Vec3::splat(pulse * movement_scale);

        // Rotate based on movement direction
        if controller.movement_input.length() > 0.1 {
            let angle = controller
                .movement_input
                .y
                .atan2(controller.movement_input.x);
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

/// System to handle option collection events and provide feedback
pub fn handle_collection_events(mut event_reader: EventReader<OptionCollectedEvent>) {
    for event in event_reader.read() {
        if event.is_correct {
            info!(
                "✅ Correct! Collected '{}' (ID: {})",
                event.option_text, event.option_id
            );
            // TODO: Add positive visual/audio feedback
        } else {
            info!(
                "❌ Wrong! Collected '{}' (ID: {})",
                event.option_text, event.option_id
            );
            // TODO: Add negative visual/audio feedback
        }
    }
}
