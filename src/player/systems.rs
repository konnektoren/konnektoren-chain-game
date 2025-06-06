use super::components::*;
use crate::{
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

/// System to handle player input
pub fn handle_player_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut PlayerController, With<Player>>,
) {
    for mut controller in &mut player_query {
        // Only accept input if player can move
        if !controller.can_move {
            continue;
        }

        // Collect directional input
        let mut movement = Vec2::ZERO;

        if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::KeyW) {
            movement.y = 1.0;
        }
        if input.just_pressed(KeyCode::ArrowDown) || input.just_pressed(KeyCode::KeyS) {
            movement.y = -1.0;
        }
        if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
            movement.x = -1.0;
        }
        if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
            movement.x = 1.0;
        }

        // Only allow one direction at a time (snake-like movement)
        if movement != Vec2::ZERO {
            if movement.x != 0.0 {
                movement.y = 0.0; // Prioritize horizontal movement
            }
            controller.movement_input = movement;
        }
    }
}

/// System to move the player on the grid
pub fn move_player(
    time: Res<Time>,
    grid_map: Option<Res<GridMap>>,
    mut player_query: Query<
        (&mut PlayerController, &mut GridPosition, &mut Transform),
        With<Player>,
    >,
) {
    let Some(grid_map) = grid_map else {
        return;
    };

    for (mut controller, mut grid_pos, mut transform) in &mut player_query {
        controller.move_timer.tick(time.delta());

        // Only move when timer finishes and there's input
        if controller.move_timer.just_finished() && controller.movement_input != Vec2::ZERO {
            let new_x = (grid_pos.x as i32 + controller.movement_input.x as i32).max(0) as usize;
            let new_y = (grid_pos.y as i32 + controller.movement_input.y as i32).max(0) as usize;

            // Clamp to grid bounds
            let new_x = new_x.min(grid_map.width - 1);
            let new_y = new_y.min(grid_map.height - 1);

            // Update position if it changed
            if new_x != grid_pos.x || new_y != grid_pos.y {
                grid_pos.x = new_x;
                grid_pos.y = new_y;

                let world_pos = grid_map.grid_to_world(new_x, new_y);
                transform.translation.x = world_pos.x;
                transform.translation.y = world_pos.y;

                info!("Player moved to ({}, {})", new_x, new_y);
            }

            // Clear input after processing
            controller.movement_input = Vec2::ZERO;
        }
    }
}

/// System to handle option collection
pub fn collect_options(
    mut commands: Commands,
    mut event_writer: EventWriter<OptionCollectedEvent>,
    mut player_query: Query<(Entity, &GridPosition), With<Player>>,
    option_query: Query<(Entity, &GridPosition, &OptionCollectible, &OptionType), Without<Player>>,
) {
    for (player_entity, player_pos) in &mut player_query {
        for (option_entity, option_pos, collectible, option_type) in &option_query {
            // Check if player is on the same grid position as an option
            if player_pos.x == option_pos.x && player_pos.y == option_pos.y {
                // Send collection event
                event_writer.send(OptionCollectedEvent {
                    player_entity, // Add this field
                    option_id: option_type.option_id,
                    is_correct: collectible.is_correct,
                    option_text: collectible.option_text.clone(),
                });

                // Remove the collected option
                commands.entity(option_entity).despawn_recursive();
            }
        }
    }
}

/// System to animate player (pulsing effect)
pub fn animate_player(
    time: Res<Time>,
    mut player_query: Query<&mut Transform, (With<Player>, With<PlayerVisual>)>,
) {
    let time_factor = time.elapsed_secs() * 4.0; // Faster animation than options

    for mut transform in &mut player_query {
        let pulse = 1.0 + (time_factor.sin() * 0.1);
        transform.scale = Vec3::splat(pulse);
    }
}
/// System to handle option collection events and provide feedback
pub fn handle_collection_events(
    mut event_reader: EventReader<OptionCollectedEvent>,
    mut commands: Commands,
) {
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
