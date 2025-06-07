use super::components::*;
use crate::{map::GridMap, screens::Screen};
use bevy::prelude::*;

/// System to set up the gameplay camera
pub fn setup_gameplay_camera(
    mut commands: Commands,
    grid_map: Option<Res<GridMap>>,
    existing_cameras: Query<Entity, With<Camera2d>>,
) {
    // Remove any existing cameras
    for camera_entity in &existing_cameras {
        commands.entity(camera_entity).despawn();
    }

    // Create camera bounds based on map size
    let camera_bounds = if let Some(map) = grid_map.as_ref() {
        CameraBounds::from_map_size(
            map.world_width(),
            map.world_height(),
            super::MULTI_PLAYER_PADDING,
        )
    } else {
        CameraBounds::new(-500.0, 500.0, -400.0, 400.0)
    };

    info!("Gameplay camera spawned with bounds: {:?}", camera_bounds);
    // Spawn the gameplay camera
    commands.spawn((
        Name::new("Gameplay Camera"),
        Camera2d,
        CameraController::default(),
        camera_bounds,
        StateScoped(Screen::Gameplay),
    ));
}

/// System to update camera targets and calculate target position
pub fn update_camera_targets(
    mut camera_query: Query<&mut CameraController>,
    target_query: Query<(&Transform, &CameraTarget)>,
    camera_settings: Res<CameraSettings>,
) {
    for mut camera_controller in &mut camera_query {
        if !camera_controller.is_following {
            continue;
        }

        let targets: Vec<_> = target_query.iter().collect();

        if targets.is_empty() {
            continue;
        }

        let target_position = calculate_target_position(&targets, &camera_settings.follow_mode);
        camera_controller.target_position = target_position;

        // Update target zoom for auto-zoom
        if camera_settings.auto_zoom {
            camera_controller.target_zoom = calculate_target_zoom(&targets, &camera_settings);
        }
    }
}

/// Calculate the target position based on follow mode
fn calculate_target_position(
    targets: &[(&Transform, &CameraTarget)],
    follow_mode: &CameraFollowMode,
) -> Vec2 {
    if targets.is_empty() {
        return Vec2::ZERO;
    }

    match follow_mode {
        CameraFollowMode::Weighted => {
            let mut weighted_sum = Vec2::ZERO;
            let mut total_weight = 0.0;

            for (transform, target) in targets {
                let position = transform.translation.xy();
                weighted_sum += position * target.weight;
                total_weight += target.weight;
            }

            if total_weight > 0.0 {
                weighted_sum / total_weight
            } else {
                targets[0].0.translation.xy()
            }
        }

        CameraFollowMode::Priority => targets
            .iter()
            .max_by_key(|(_, target)| target.priority)
            .map(|(transform, _)| transform.translation.xy())
            .unwrap_or(Vec2::ZERO),

        CameraFollowMode::IncludeAll | CameraFollowMode::Average => {
            let sum: Vec2 = targets
                .iter()
                .map(|(transform, _)| transform.translation.xy())
                .sum();
            sum / targets.len() as f32
        }
    }
}

/// Calculate target zoom to include all targets
fn calculate_target_zoom(
    targets: &[(&Transform, &CameraTarget)],
    settings: &CameraSettings,
) -> f32 {
    if targets.len() <= 1 || !settings.auto_zoom {
        return super::DEFAULT_CAMERA_ZOOM;
    }

    // Find the bounding box of all targets
    let positions: Vec<Vec2> = targets
        .iter()
        .map(|(transform, _)| transform.translation.xy())
        .collect();

    let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = positions
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);

    let width = max_x - min_x + settings.zoom_margin;
    let height = max_y - min_y + settings.zoom_margin;

    // Calculate zoom to fit the bounding box (simplified)
    let target_zoom = if width > height {
        (800.0 / width).min(super::MAX_CAMERA_ZOOM)
    } else {
        (600.0 / height).min(super::MAX_CAMERA_ZOOM)
    };

    target_zoom.max(super::MIN_CAMERA_ZOOM)
}

/// System to smoothly move camera to target position
pub fn update_camera_follow(
    time: Res<Time>,
    camera_settings: Res<CameraSettings>,
    mut camera_query: Query<(&mut Transform, &mut CameraController, &CameraBounds)>,
) {
    for (mut transform, mut controller, bounds) in &mut camera_query {
        if !controller.is_following {
            continue;
        }

        let current_pos = transform.translation.xy();
        let target_pos = controller.target_position;
        let distance_to_target = current_pos.distance(target_pos);

        // Only move if outside deadzone
        if distance_to_target > controller.deadzone_radius {
            let direction = (target_pos - current_pos).normalize_or_zero();
            let movement_distance = distance_to_target - controller.deadzone_radius;

            if camera_settings.smooth_follow {
                // Smooth following with velocity
                let target_velocity = direction * movement_distance * controller.follow_speed;
                controller.current_velocity = controller
                    .current_velocity
                    .lerp(target_velocity, time.delta_secs() * controller.follow_speed);

                let new_position = current_pos + controller.current_velocity * time.delta_secs();
                let clamped_position = bounds.clamp_position(new_position);

                transform.translation.x = clamped_position.x;
                transform.translation.y = clamped_position.y;
            } else {
                // Instant movement
                let new_position = current_pos + direction * movement_distance;
                let clamped_position = bounds.clamp_position(new_position);

                transform.translation.x = clamped_position.x;
                transform.translation.y = clamped_position.y;
            }
        } else {
            // Within deadzone, reduce velocity
            controller.current_velocity *= 0.9;
        }

        // Handle zoom
        let current_scale = transform.scale.x;
        if (current_scale - controller.target_zoom).abs() > 0.01 {
            let new_scale = current_scale.lerp(
                controller.target_zoom,
                time.delta_secs() * controller.zoom_speed,
            );
            transform.scale = Vec3::splat(new_scale);
        }
    }
}

/// System to update camera bounds when map changes
pub fn update_camera_bounds(
    grid_map: Res<GridMap>,
    mut camera_query: Query<&mut CameraBounds>,
    camera_settings: Res<CameraSettings>,
) {
    if !grid_map.is_changed() || !camera_settings.respect_bounds {
        return;
    }

    for mut bounds in &mut camera_query {
        let new_bounds = CameraBounds::from_map_size(
            grid_map.world_width(),
            grid_map.world_height(),
            camera_settings.zoom_margin,
        );

        *bounds = new_bounds;
        info!("Camera bounds updated for new map size");
    }
}
