use super::OPTION_FADE_DURATION;
use super::components::*;
use crate::{
    effects::SpawnCollectionEvent,
    map::{GridMap, GridPosition},
    question::QuestionSystem,
    screens::Screen,
};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Spawn a single option collectible with light effects
fn spawn_option_collectible(
    commands: &mut Commands,
    option_id: usize,
    option_text: String,
    is_correct: bool,
    grid_pos: GridPosition,
    grid_map: &GridMap,
    current_time: f32,
    lifetime: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let world_pos = grid_map.grid_to_world(grid_pos.x, grid_pos.y);

    // Choose color based on option type
    let base_colors = [
        Color::srgb(0.3, 0.5, 0.8), // Blue
        Color::srgb(0.8, 0.5, 0.3), // Orange
        Color::srgb(0.5, 0.8, 0.3), // Green
        Color::srgb(0.8, 0.3, 0.5), // Pink
        Color::srgb(0.5, 0.3, 0.8), // Purple
    ];

    let color_index = option_id % base_colors.len();
    let base_color = base_colors[color_index];

    // Make correct answers brighter
    let display_color = if is_correct {
        Color::srgb(
            (base_color.to_srgba().red * 1.3).min(1.0),
            (base_color.to_srgba().green * 1.3).min(1.0),
            (base_color.to_srgba().blue * 1.3).min(1.0),
        )
    } else {
        base_color
    };

    // Create meshes and materials for all visual layers
    let main_mesh = meshes.add(Circle::new(14.0));
    let main_material = materials.add(ColorMaterial::from(display_color));

    let glow_mesh = meshes.add(Circle::new(20.0));
    let glow_color = Color::srgba(
        display_color.to_srgba().red,
        display_color.to_srgba().green,
        display_color.to_srgba().blue,
        0.3,
    );
    let glow_material = materials.add(ColorMaterial::from(glow_color));

    let pulse_mesh = meshes.add(Circle::new(30.0));
    let pulse_color = Color::srgba(
        display_color.to_srgba().red,
        display_color.to_srgba().green,
        display_color.to_srgba().blue,
        0.1,
    );
    let pulse_material = materials.add(ColorMaterial::from(pulse_color));

    let mut collectible =
        OptionCollectible::new(option_id, option_text.clone(), is_correct, lifetime);
    collectible.spawn_time = current_time;

    // Spawn the main option entity with all light effects
    commands.spawn((
        Name::new(format!("Option: {}", option_text)),
        Mesh2d(main_mesh),
        MeshMaterial2d(main_material),
        Transform::from_translation(Vec3::new(world_pos.x, world_pos.y, 1.0)),
        grid_pos,
        collectible,
        OptionType::new(option_id),
        OptionVisual,
        OptionLightEffect::new(base_color, is_correct),
        OptionSparkles::new(is_correct), // Use different settings based on correctness
        StateScoped(Screen::Gameplay),
        children![
            // Text label
            (
                Name::new("Option Text"),
                Text2d::new(option_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.3)),
            ),
            // Inner glow effect
            (
                Name::new("Option Glow"),
                Mesh2d(glow_mesh),
                MeshMaterial2d(glow_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
                OptionGlow,
            ),
            // Outer pulse ring
            (
                Name::new("Option Pulse Ring"),
                Mesh2d(pulse_mesh),
                MeshMaterial2d(pulse_material),
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.2)),
                OptionPulseRing::new(40.0),
            ),
        ],
    ));
}

/// System to spawn option collectibles on the map
pub fn spawn_option_collectibles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<OptionSpawnTimer>,
    question_system: Option<Res<QuestionSystem>>,
    grid_map: Option<Res<GridMap>>,
    existing_options: Query<(&OptionType, &GridPosition), With<OptionCollectible>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_timer.timer.tick(time.delta());

    if !spawn_timer.timer.just_finished() {
        return;
    }

    let Some(question_system) = question_system else {
        return;
    };

    let Some(grid_map) = grid_map else {
        return;
    };

    let Some(current_question) = question_system.get_current_question() else {
        return;
    };

    let options = question_system.get_current_options();
    let current_time = time.elapsed_secs();

    // Count existing options by type and total
    let mut option_counts: HashMap<usize, usize> = HashMap::new();
    let mut occupied_positions: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    for (option_type, grid_pos) in &existing_options {
        *option_counts.entry(option_type.option_id).or_insert(0) += 1;
        occupied_positions.insert((grid_pos.x, grid_pos.y));
    }

    let total_existing = existing_options.iter().count();

    // Don't spawn if we already have enough options total
    if total_existing >= spawn_timer.total_target_options {
        return;
    }

    info!(
        "Spawning options: {}/{} exist, target: {} total, {} per type",
        total_existing,
        spawn_timer.total_target_options,
        spawn_timer.total_target_options,
        spawn_timer.options_per_type
    );

    // For each option type, ensure we have the right number spawned
    for option in options {
        let existing_count = option_counts.get(&option.id).copied().unwrap_or(0);
        let is_correct = option.id == current_question.option;

        // Check if we should spawn more of this type
        // Also check that we don't exceed the total target
        if existing_count < spawn_timer.options_per_type
            && total_existing < spawn_timer.total_target_options
        {
            let spawn_count = (spawn_timer.options_per_type - existing_count)
                .min(spawn_timer.total_target_options - total_existing);

            for _ in 0..spawn_count {
                if let Some(spawn_pos) = find_empty_spawn_position(&grid_map, &occupied_positions) {
                    spawn_option_collectible(
                        &mut commands,
                        option.id,
                        option.name.clone(),
                        is_correct,
                        spawn_pos.clone(),
                        &grid_map,
                        current_time,
                        spawn_timer.option_lifetime,
                        &mut meshes,
                        &mut materials,
                    );

                    // Mark this position as occupied for subsequent spawns
                    occupied_positions.insert((spawn_pos.x, spawn_pos.y));

                    info!(
                        "Spawned option '{}' at ({}, {})",
                        option.name, spawn_pos.x, spawn_pos.y
                    );
                }
            }
        }
    }
}

/// System to animate option collectibles with enhanced light effects
pub fn animate_option_collectibles(
    time: Res<Time>,
    mut options_query: Query<
        (&mut Transform, &OptionLightEffect),
        (
            With<OptionCollectible>,
            With<OptionVisual>,
            Without<Text2d>,
            Without<OptionGlow>,
            Without<OptionPulseRing>,
        ),
    >,
    mut glow_query: Query<
        (&mut Transform, &mut MeshMaterial2d<ColorMaterial>),
        (
            With<OptionGlow>,
            Without<OptionVisual>,
            Without<OptionPulseRing>,
        ),
    >,
    mut pulse_query: Query<
        (
            &mut Transform,
            &mut OptionPulseRing,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        (Without<OptionVisual>, Without<OptionGlow>),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let time_factor = time.elapsed_secs();

    // Animate main option bodies
    for (mut transform, light_effect) in &mut options_query {
        // Base pulsing scale effect
        let pulse = 1.0 + (time_factor * light_effect.pulse_speed).sin() * 0.15;
        transform.scale = Vec3::splat(pulse);

        // Gentle floating motion
        let float_offset_x = (time_factor * 0.8).sin() * 2.0;
        let float_offset_y = (time_factor * 1.2).cos() * 1.5;

        // Apply floating offset
        transform.translation.x += float_offset_x * 0.1;
        transform.translation.y += float_offset_y * 0.1;
    }

    // Animate glow effects
    for (mut transform, material_handle) in &mut glow_query {
        // Glow pulse (slower than main pulse)
        let glow_pulse = 1.0 + (time_factor * 1.5).sin() * 0.3;
        transform.scale = Vec3::splat(glow_pulse);

        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 0.2 + (time_factor * 2.0).sin() * 0.1;
            material.color.set_alpha(alpha.max(0.1));
        }
    }

    // Animate pulse rings
    for (mut transform, mut pulse_ring, material_handle) in &mut pulse_query {
        pulse_ring.ring_phase += time.delta_secs() * 2.0;
        if pulse_ring.ring_phase > std::f32::consts::TAU {
            pulse_ring.ring_phase = 0.0;
        }

        // Expanding ring effect
        let ring_progress = (pulse_ring.ring_phase / std::f32::consts::TAU).sin();
        let ring_scale = 0.5 + ring_progress * 1.5;
        transform.scale = Vec3::splat(ring_scale);

        // Fade out as ring expands
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = (1.0 - ring_progress) * 0.15;
            material.color.set_alpha(alpha.max(0.0));
        }
    }
}

/// System to spawn sparkle particle effects around options
pub fn update_option_sparkles(
    time: Res<Time>,
    mut sparkle_query: Query<
        (&Transform, &mut OptionSparkles, &OptionLightEffect),
        With<OptionCollectible>,
    >,
    mut collection_events: EventWriter<SpawnCollectionEvent>,
) {
    for (transform, mut sparkles, light_effect) in &mut sparkle_query {
        sparkles.sparkle_timer.tick(time.delta());

        if sparkles.sparkle_timer.just_finished() {
            // Simple intensity check using time-based randomness
            let time_factor = time.elapsed_secs();
            let pseudo_random = (time_factor * 13.7).fract(); // Simple pseudo-random

            if pseudo_random > sparkles.sparkle_intensity {
                continue;
            }

            let base_pos = transform.translation;

            for i in 0..sparkles.sparkle_count {
                // Use time and index for pseudo-random positioning
                let angle = (time_factor * 2.0 + i as f32 * 2.1).fract() * std::f32::consts::TAU;
                let distance = 15.0 + ((time_factor * 3.7 + i as f32).fract() * 10.0);

                let sparkle_pos = Vec3::new(
                    base_pos.x + angle.cos() * distance,
                    base_pos.y + angle.sin() * distance,
                    base_pos.z + 0.1,
                );

                let sparkle_color = if light_effect.is_correct_answer {
                    Color::srgb(
                        (light_effect.base_color.to_srgba().red + 0.3).min(1.0),
                        (light_effect.base_color.to_srgba().green + 0.2).min(1.0),
                        light_effect.base_color.to_srgba().blue,
                    )
                } else {
                    light_effect.base_color
                };

                collection_events.write(SpawnCollectionEvent {
                    position: sparkle_pos,
                    color: sparkle_color,
                });
            }
        }
    }
}

/// System to enhance correct answer visual effects
pub fn enhance_correct_answer_effects(
    time: Res<Time>,
    question_system: Option<Res<crate::question::QuestionSystem>>,
    mut correct_options_query: Query<
        (&OptionCollectible, &mut OptionLightEffect, &Children),
        With<OptionVisual>,
    >,
    mut glow_query: Query<&mut MeshMaterial2d<ColorMaterial>, With<OptionGlow>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Some(question_system) = question_system else {
        return;
    };

    let Some(current_question) = question_system.get_current_question() else {
        return;
    };

    let time_factor = time.elapsed_secs();

    for (option, mut light_effect, children) in &mut correct_options_query {
        // Check if this is the correct answer
        if option.option_id == current_question.option {
            // Enhanced effects for correct answer
            light_effect.pulse_intensity = 0.9;
            light_effect.pulse_speed = 4.0;

            // Make the glow more intense
            for child in children.iter() {
                if let Ok(material_handle) = glow_query.get_mut(child) {
                    if let Some(material) = materials.get_mut(&material_handle.0) {
                        // Golden glow for correct answers
                        let golden_tint = Color::srgb(1.0, 0.9, 0.3);
                        let base_color = light_effect.base_color;
                        let mixed_color = Color::srgb(
                            (base_color.to_srgba().red + golden_tint.to_srgba().red) / 2.0,
                            (base_color.to_srgba().green + golden_tint.to_srgba().green) / 2.0,
                            (base_color.to_srgba().blue + golden_tint.to_srgba().blue) / 2.0,
                        );

                        let alpha = 0.4 + (time_factor * 3.0).sin() * 0.2;
                        material.color = Color::srgba(
                            mixed_color.to_srgba().red,
                            mixed_color.to_srgba().green,
                            mixed_color.to_srgba().blue,
                            alpha.max(0.1),
                        );
                    }
                }
            }
        }
    }
}

/// Find an empty position to spawn an option
fn find_empty_spawn_position(
    grid_map: &GridMap,
    occupied_positions: &std::collections::HashSet<(usize, usize)>,
) -> Option<GridPosition> {
    let mut rng = rand::thread_rng();
    let max_attempts = 50;

    // Use buffer based on map size - larger maps get smaller buffers
    let buffer = if grid_map.width > 30 { 1 } else { 2 };

    for _ in 0..max_attempts {
        let x = rng.gen_range(buffer..grid_map.width.saturating_sub(buffer));
        let y = rng.gen_range(buffer..grid_map.height.saturating_sub(buffer));

        if !occupied_positions.contains(&(x, y)) {
            return Some(GridPosition::new(x, y));
        }
    }

    None
}

/// System to clean up expired option collectibles
pub fn cleanup_expired_options(
    mut commands: Commands,
    time: Res<Time>,
    options_query: Query<(Entity, &OptionCollectible)>,
) {
    let current_time = time.elapsed_secs();

    for (entity, option) in &options_query {
        if option.is_expired(current_time) {
            commands.entity(entity).despawn();
        }
    }
}

/// System to clear all options when question changes
pub fn clear_options_on_question_change(
    mut commands: Commands,
    question_system: Res<QuestionSystem>,
    options_query: Query<Entity, With<OptionCollectible>>,
) {
    if question_system.is_changed() {
        info!(
            "Question changed, clearing {} options",
            options_query.iter().count()
        );
        for entity in &options_query {
            commands.entity(entity).despawn();
        }
    }
}

/// System to make options fade out as they approach expiration
pub fn fade_expiring_options(
    time: Res<Time>,
    options_query: Query<(&OptionCollectible, &MeshMaterial2d<ColorMaterial>), With<OptionVisual>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let current_time = time.elapsed_secs();

    for (option, material_handle) in &options_query {
        let time_remaining = option.time_remaining(current_time);

        if time_remaining <= OPTION_FADE_DURATION && time_remaining > 0.0 {
            let alpha = (time_remaining / OPTION_FADE_DURATION).max(0.1);

            if let Some(material) = materials.get_mut(&material_handle.0) {
                let mut color = material.color;
                color.set_alpha(alpha);
                material.color = color;
            }
        }
    }
}

/// System to update option spawn settings based on map size
pub fn update_option_spawn_settings(
    mut spawn_timer: ResMut<OptionSpawnTimer>,
    grid_map: Option<Res<GridMap>>,
    question_system: Option<Res<QuestionSystem>>,
) {
    let Some(grid_map) = grid_map else {
        return;
    };

    let Some(question_system) = question_system else {
        return;
    };

    // Only update when map or question system changes
    if !grid_map.is_changed() && !question_system.is_changed() {
        return;
    }

    let option_types = question_system.get_current_options().len();
    spawn_timer.calculate_target_options(grid_map.width, grid_map.height, option_types);
}
