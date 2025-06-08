use super::components::*;
use crate::{
    input::{InputController, PlayerInputMapping},
    map::{GridMap, GridPosition},
    options::{OptionCollectible, OptionType},
    screens::Screen,
    settings::GameSettings, // Add this import
};
use bevy::prelude::*;

/// System to spawn the player at the center of the grid with enhanced visuals
pub fn spawn_player(
    mut commands: Commands,
    grid_map: Option<Res<GridMap>>,
    game_settings: Res<GameSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(grid_map) = grid_map else {
        error!("GridMap not available when trying to spawn player!");
        return;
    };

    let player_count = game_settings.multiplayer.player_count;

    for (player_index, player_settings) in game_settings.multiplayer.players.iter().enumerate() {
        if !player_settings.enabled {
            continue;
        }

        // Calculate spawn position based on player count
        let spawn_pos = calculate_player_spawn_position(player_index, player_count, &grid_map);
        let world_pos = grid_map.grid_to_world(spawn_pos.x, spawn_pos.y);

        let player_effects = PlayerEffects {
            base_color: player_settings.color,
            ..Default::default()
        };

        // Create main player visual
        let main_mesh = meshes.add(Circle::new(super::PLAYER_SIZE));
        let main_material = materials.add(ColorMaterial::from(player_settings.color));

        // Create visual effect entities
        let core_mesh = meshes.add(Circle::new(super::PLAYER_SIZE * 0.6));
        let core_color = Color::srgb(1.0, 1.0, 1.0);
        let core_material = materials.add(ColorMaterial::from(core_color));

        let glow_mesh = meshes.add(Circle::new(super::PLAYER_SIZE * 1.4));
        let glow_color = Color::srgba(
            player_settings.color.to_srgba().red,
            player_settings.color.to_srgba().green,
            player_settings.color.to_srgba().blue,
            0.4,
        );
        let glow_material = materials.add(ColorMaterial::from(glow_color));

        let aura_mesh = meshes.add(Circle::new(super::PLAYER_SIZE * 2.0));
        let aura_color = Color::srgba(
            player_settings.color.to_srgba().red,
            player_settings.color.to_srgba().green,
            player_settings.color.to_srgba().blue,
            0.15,
        );
        let aura_material = materials.add(ColorMaterial::from(aura_color));

        // Spawn the player entity with core components first
        let player_entity = commands
            .spawn((
                Name::new(format!("Player {}", player_index + 1)),
                Player,
                PlayerController::default(),
                PlayerStats::default(),
                PlayerVisual,
                Transform::from_translation(Vec3::new(world_pos.x, world_pos.y, 2.0)),
                spawn_pos,
                StateScoped(Screen::Gameplay),
                PlayerIndex(player_index), // ADD THIS LINE - this is the crucial missing component!
            ))
            .id();

        // Add additional components in separate calls to avoid tuple size limits
        commands.entity(player_entity).insert((
            player_effects,
            PlayerEnergyParticles::default(),
            PlayerTrail::default(),
            InputController {
                player_id: player_index as u32,
                ..Default::default()
            },
            PlayerInputMapping {
                player_id: player_index as u32,
                ..Default::default()
            },
        ));

        commands.entity(player_entity).insert((
            crate::camera::CameraTarget::default(),
            Mesh2d(main_mesh),
            MeshMaterial2d(main_material),
        ));

        // Add child entities for visual effects
        let core_entity = commands
            .spawn((
                Name::new("Player Core"),
                Mesh2d(core_mesh),
                MeshMaterial2d(core_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
            ))
            .id();

        let glow_entity = commands
            .spawn((
                Name::new("Player Glow"),
                Mesh2d(glow_mesh),
                MeshMaterial2d(glow_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
                PlayerGlow,
            ))
            .id();

        let aura_entity = commands
            .spawn((
                Name::new("Player Aura"),
                Mesh2d(aura_mesh),
                MeshMaterial2d(aura_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.2)),
                PlayerAura::new(super::PLAYER_SIZE * 2.5),
            ))
            .id();

        // Set up parent-child relationships
        commands
            .entity(player_entity)
            .add_children(&[core_entity, glow_entity, aura_entity]);

        // Fix: Access spawn_pos values before it was moved, or recreate the position info
        let spawn_x = (world_pos.x / grid_map.cell_size + grid_map.width as f32 / 2.0) as usize;
        let spawn_y = (world_pos.y / grid_map.cell_size + grid_map.height as f32 / 2.0) as usize;

        info!(
            "Spawned {} at position ({}, {}) with color {:?} and PlayerIndex({})",
            player_settings.name, spawn_x, spawn_y, player_settings.color, player_index
        );
    }
}

fn calculate_player_spawn_position(
    player_index: usize,
    total_players: usize,
    grid_map: &GridMap,
) -> GridPosition {
    let center_x = grid_map.width / 2;
    let center_y = grid_map.height / 2;

    if total_players == 1 {
        GridPosition::new(center_x, center_y)
    } else {
        // Spread players around the center
        let angle = (player_index as f32 / total_players as f32) * std::f32::consts::TAU;
        let radius = (grid_map.width.min(grid_map.height) / 6) as f32;

        let offset_x = (angle.cos() * radius) as i32;
        let offset_y = (angle.sin() * radius) as i32;

        let spawn_x = (center_x as i32 + offset_x).clamp(1, grid_map.width as i32 - 2) as usize;
        let spawn_y = (center_y as i32 + offset_y).clamp(1, grid_map.height as i32 - 2) as usize;

        GridPosition::new(spawn_x, spawn_y)
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

        // Handle wraparound using grid map dimensions
        let wrapped_world_pos =
            handle_map_wraparound(new_world_pos, grid_map.half_width(), grid_map.half_height());

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

/// System to animate player with enhanced visual effects (OPTIMIZED)
pub fn animate_player(
    time: Res<Time>,
    mut player_query: Query<
        (
            &PlayerController,
            &mut Transform,
            &mut PlayerEffects,
            &Children,
        ),
        (With<Player>, With<PlayerVisual>),
    >,
    mut glow_query: Query<
        (&mut Transform, &mut MeshMaterial2d<ColorMaterial>),
        (With<PlayerGlow>, Without<Player>, Without<PlayerAura>),
    >,
    mut aura_query: Query<
        (
            &mut Transform,
            &mut PlayerAura,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        (Without<Player>, Without<PlayerGlow>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let time_factor = time.elapsed_secs();

    for (controller, mut transform, mut effects, children) in &mut player_query {
        // Update boost timer
        if effects.is_boosted {
            effects.boost_timer.tick(time.delta());
            if effects.boost_timer.finished() {
                effects.is_boosted = false;
                effects.glow_intensity = 0.8;
                effects.pulse_speed = 3.0;
                effects.energy_level = 1.0;
                // Reset base color when boost ends
                effects.base_color = Color::srgb(1.0, 0.8, 0.2);
            }
        }

        // Base pulsing effect (more intense when moving)
        let movement_intensity = if controller.movement_input.length() > 0.1 {
            1.2
        } else {
            1.0
        };
        let pulse = 1.0 + (time_factor * effects.pulse_speed).sin() * 0.1 * movement_intensity;
        transform.scale = Vec3::splat(pulse);

        // Rotation based on movement
        if controller.movement_input.length() > 0.1 {
            let target_angle = controller
                .movement_input
                .y
                .atan2(controller.movement_input.x);
            let current_rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
            let angle_diff = (target_angle - current_rotation + std::f32::consts::PI)
                % (2.0 * std::f32::consts::PI)
                - std::f32::consts::PI;
            let new_rotation = current_rotation + angle_diff * time.delta_secs() * 8.0;
            transform.rotation = Quat::from_rotation_z(new_rotation);
        }

        // Update glow effects (less frequent updates)
        for child in children.iter() {
            if let Ok((mut glow_transform, material_handle)) = glow_query.get_mut(child) {
                // Glow pulsing (offset from main pulse)
                let glow_pulse = 1.0 + (time_factor * effects.pulse_speed * 1.3).sin() * 0.2;
                glow_transform.scale = Vec3::splat(glow_pulse);

                // Only update material color occasionally to reduce performance impact
                if (time_factor * 10.0) as i32 % 2 == 0 {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        let current_color = effects.get_current_color(time_factor);
                        let alpha =
                            effects.glow_intensity * (0.3 + (time_factor * 2.5).sin() * 0.1);
                        material.color = Color::srgba(
                            current_color.to_srgba().red,
                            current_color.to_srgba().green,
                            current_color.to_srgba().blue,
                            alpha,
                        );
                    }
                }
            }

            if let Ok((mut aura_transform, mut aura, material_handle)) = aura_query.get_mut(child) {
                // Aura rotation and pulsing
                aura.aura_phase += time.delta_secs() * 1.5;
                if aura.aura_phase > std::f32::consts::TAU {
                    aura.aura_phase = 0.0;
                }

                // Rotating aura effect
                aura_transform.rotation = Quat::from_rotation_z(aura.aura_phase);

                // Breathing aura effect
                let aura_scale = 1.0 + (time_factor * 1.0).sin() * 0.15;
                aura_transform.scale = Vec3::splat(aura_scale);

                // Only update material color occasionally
                if (time_factor * 8.0) as i32 % 3 == 0 {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        let current_color = effects.get_current_color(time_factor);
                        let alpha =
                            if effects.is_boosted { 0.3 } else { 0.1 } * effects.energy_level;
                        material.color = Color::srgba(
                            current_color.to_srgba().red,
                            current_color.to_srgba().green,
                            current_color.to_srgba().blue,
                            alpha,
                        );
                    }
                }
            }
        }
    }
}

/// System to create energy particles around the player (OPTIMIZED)
pub fn update_player_energy_particles(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut PlayerEnergyParticles, &PlayerEffects), With<Player>>,
    mut particle_events: EventWriter<crate::effects::SpawnCollectionEvent>,
) {
    for (transform, mut particles, effects) in &mut player_query {
        particles.particle_timer.tick(time.delta());

        // Reduce particle frequency and only spawn when energy is high
        if particles.particle_timer.just_finished()
            && effects.energy_level > 0.7
            && !effects.is_boosted
        // Disable regular particles during boost to reduce spam
        {
            let base_pos = transform.translation;
            let time_factor = time.elapsed_secs();

            // Reduce particle count
            let particle_count = if effects.energy_level > 0.9 { 1 } else { 0 };

            for i in 0..particle_count {
                // Create orbital particle positions
                let angle = time_factor * 2.0
                    + i as f32 * std::f32::consts::TAU / particles.particle_count as f32;
                let radius = super::PLAYER_SIZE * 1.8;

                let particle_pos = Vec3::new(
                    base_pos.x + angle.cos() * radius,
                    base_pos.y + angle.sin() * radius,
                    base_pos.z + 0.2,
                );

                particle_events.write(crate::effects::SpawnCollectionEvent {
                    position: particle_pos,
                    color: effects.get_current_color(time_factor),
                });
            }
        }
    }
}

/// System to create movement trail (OPTIMIZED)
pub fn update_player_trail(
    time: Res<Time>,
    mut player_query: Query<
        (
            &Transform,
            &mut PlayerTrail,
            &PlayerController,
            &PlayerEffects,
        ),
        With<Player>,
    >,
    mut trail_events: EventWriter<crate::effects::SpawnCollectionEvent>,
) {
    for (transform, mut trail, controller, effects) in &mut player_query {
        trail.trail_timer.tick(time.delta());

        // Only create trail when moving and reduce frequency
        if controller.movement_input.length() > 0.1
            && trail.trail_timer.just_finished()
            && effects.trail_enabled
            && !effects.is_boosted
        // Disable trail during boost to reduce particle spam
        {
            let current_pos = transform.translation;
            let current_time = time.elapsed_secs();

            // Add current position to trail
            trail.trail_positions.push((current_pos, current_time));

            // Remove old trail positions
            trail
                .trail_positions
                .retain(|(_, age)| current_time - age < 0.5); // Shorter trail duration

            // Limit trail length more aggressively
            while trail.trail_positions.len() > 10 {
                // Reduced from 20
                trail.trail_positions.remove(0);
            }

            // Create fewer trail particles
            for (i, (pos, age)) in trail.trail_positions.iter().enumerate().rev() {
                if i % 5 == 0 {
                    // Only every 5th position instead of every 3rd
                    let trail_alpha = (1.0 - (current_time - age) * 2.0) * 0.3; // Faster fade
                    if trail_alpha > 0.05 {
                        let mut trail_color = effects.base_color; // Use base color instead of animated color
                        trail_color.set_alpha(trail_alpha);

                        trail_events.write(crate::effects::SpawnCollectionEvent {
                            position: *pos,
                            color: trail_color,
                        });
                    }
                }
            }
        }
    }
}

/// System to handle player visual events (OPTIMIZED)
pub fn handle_player_visual_events(
    mut visual_events: EventReader<PlayerVisualEvent>,
    mut player_query: Query<&mut PlayerEffects, With<Player>>,
) {
    for event in visual_events.read() {
        if let Ok(mut effects) = player_query.get_mut(event.player_entity) {
            match &event.event_type {
                PlayerVisualEventType::CorrectAnswer => {
                    effects.boost(1.0, 1.0);
                    effects.base_color = Color::srgb(0.2, 1.0, 0.2); // Green boost
                }
                PlayerVisualEventType::WrongAnswer => {
                    effects.energy_level = (effects.energy_level - 0.2).max(0.2);
                    effects.base_color = Color::srgb(1.0, 0.3, 0.3); // Red indication
                    // Reset color after a short time to prevent permanent color change
                    effects.boost_timer = Timer::from_seconds(0.5, TimerMode::Once);
                }
                PlayerVisualEventType::Streak(count) => {
                    let intensity = (*count as f32 * 0.1).min(1.0);
                    effects.boost(1.5, intensity); // Reduced duration
                    // Only apply rainbow for very high streaks
                    if *count > 10 {
                        effects.base_color = Color::hsl((*count as f32 * 30.0) % 360.0, 0.8, 0.6);
                    }
                }
                PlayerVisualEventType::Boost {
                    duration,
                    intensity,
                } => {
                    effects.boost(*duration, *intensity);
                }
            }
        }
    }
}

/// System to handle option collection events and provide enhanced feedback (OPTIMIZED)
pub fn handle_collection_events(
    mut collection_events: EventReader<OptionCollectedEvent>,
    mut visual_events: EventWriter<PlayerVisualEvent>,
    mut player_query: Query<&mut PlayerStats, With<Player>>,
) {
    for event in collection_events.read() {
        if let Ok(mut stats) = player_query.get_mut(event.player_entity) {
            if event.is_correct {
                stats.correct_answers += 1;
                stats.current_streak += 1;

                // Update best streak
                if stats.current_streak > stats.best_streak {
                    stats.best_streak = stats.current_streak;
                }

                info!(
                    "✅ Correct! Collected '{}' (ID: {}) - Streak: {}",
                    event.option_text, event.option_id, stats.current_streak
                );

                // Send visual feedback
                visual_events.write(PlayerVisualEvent {
                    player_entity: event.player_entity,
                    event_type: PlayerVisualEventType::CorrectAnswer,
                });

                // Only send streak events for significant milestones to reduce spam
                if stats.current_streak % 3 == 0 && stats.current_streak > 3 {
                    visual_events.write(PlayerVisualEvent {
                        player_entity: event.player_entity,
                        event_type: PlayerVisualEventType::Streak(stats.current_streak),
                    });
                }

                // Extra boost only for higher milestones to reduce effects
                if stats.current_streak % 10 == 0 && stats.current_streak > 0 {
                    visual_events.write(PlayerVisualEvent {
                        player_entity: event.player_entity,
                        event_type: PlayerVisualEventType::Boost {
                            duration: 2.0,  // Reduced duration
                            intensity: 1.0, // Reduced intensity
                        },
                    });
                    info!("🚀 Milestone streak reached: {}!", stats.current_streak);
                }
            } else {
                stats.wrong_answers += 1;
                stats.current_streak = 0;

                info!(
                    "❌ Wrong! Collected '{}' (ID: {})",
                    event.option_text, event.option_id
                );

                // Send visual feedback
                visual_events.write(PlayerVisualEvent {
                    player_entity: event.player_entity,
                    event_type: PlayerVisualEventType::WrongAnswer,
                });

                // Remove energy drain to reduce effect spam
                // visual_events.write(PlayerVisualEvent {
                //     player_entity: event.player_entity,
                //     event_type: PlayerVisualEventType::EnergyDrain,
                // });
            }
        }
    }
}
