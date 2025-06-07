use super::OPTION_FADE_DURATION;
use super::components::*;
use crate::{
    map::{GridMap, GridPosition},
    question::QuestionSystem,
    screens::Screen,
};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;

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

    // Count existing options by type
    let mut option_counts: HashMap<usize, usize> = HashMap::new();
    let mut occupied_positions: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();

    for (option_type, grid_pos) in &existing_options {
        *option_counts.entry(option_type.option_id).or_insert(0) += 1;
        occupied_positions.insert((grid_pos.x, grid_pos.y));
    }

    // For each option type, ensure we have the right number spawned
    for option in options {
        let existing_count = option_counts.get(&option.id).copied().unwrap_or(0);
        let is_correct = option.id == current_question.option;

        // Spawn options up to the limit
        for _ in existing_count..spawn_timer.options_per_type {
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

    for _ in 0..max_attempts {
        let x = rng.gen_range(2..grid_map.width - 2);
        let y = rng.gen_range(2..grid_map.height - 2);

        if !occupied_positions.contains(&(x, y)) {
            return Some(GridPosition::new(x, y));
        }
    }

    None
}

/// Spawn a single option collectible
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

    // Choose color based on option type (better indexing)
    let base_colors = [
        Color::srgb(0.3, 0.5, 0.8), // Blue
        Color::srgb(0.8, 0.5, 0.3), // Orange
        Color::srgb(0.5, 0.8, 0.3), // Green
        Color::srgb(0.8, 0.3, 0.5), // Pink
        Color::srgb(0.5, 0.3, 0.8), // Purple
    ];

    // Use option_id directly for modulo to ensure different colors for different IDs
    let color_index = option_id % base_colors.len();
    let color = base_colors[color_index];

    // Create the visual mesh (a circle)
    let mesh = meshes.add(Circle::new(14.0));
    let material = materials.add(ColorMaterial::from(color));

    let mut collectible =
        OptionCollectible::new(option_id, option_text.clone(), is_correct, lifetime);
    collectible.spawn_time = current_time;

    commands.spawn((
        Name::new(format!("Option: {}", option_text)),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::new(world_pos.x, world_pos.y, 1.0)),
        grid_pos,
        collectible,
        OptionType::new(option_id),
        OptionVisual,
        StateScoped(Screen::Gameplay),
        children![
            // Text label for the option - centered inside the ball
            (
                Name::new("Option Text"),
                Text2d::new(option_text),
                TextFont {
                    font_size: 14.0, // Larger font for options
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)), // Centered at (0,0)
            )
        ],
    ));
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

/// System to animate option collectibles (pulsing effect)
pub fn animate_option_collectibles(
    time: Res<Time>,
    mut options_query: Query<
        &mut Transform,
        (With<OptionCollectible>, With<OptionVisual>, Without<Text2d>),
    >,
) {
    let time_factor = time.elapsed_secs() * 2.0;

    for mut transform in &mut options_query {
        let pulse = 1.0 + (time_factor.sin() * 0.15);
        transform.scale = Vec3::splat(pulse);
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
