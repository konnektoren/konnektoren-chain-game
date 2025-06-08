use super::{components::*, viewport::ViewportCalculator};
use crate::{map::GridMap, screens::Screen};
use bevy::prelude::*;

/// System to set up the title/UI camera
pub fn setup_title_camera(mut commands: Commands, existing_cameras: Query<Entity, With<Camera2d>>) {
    for camera_entity in &existing_cameras {
        commands.entity(camera_entity).despawn();
    }

    commands.spawn((
        Name::new("Title Camera"),
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        StateScoped(Screen::Title),
    ));

    info!("Title camera spawned");
}

/// System to set up the gameplay camera
pub fn setup_gameplay_camera(
    mut commands: Commands,
    grid_map: Option<Res<GridMap>>,
    existing_cameras: Query<Entity, With<Camera2d>>,
) {
    for camera_entity in &existing_cameras {
        commands.entity(camera_entity).despawn();
    }

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

    let mut camera_controller = CameraController::default();
    camera_controller.target_zoom = super::DEFAULT_CAMERA_ZOOM;
    camera_controller.follow_speed = super::DEFAULT_CAMERA_SPEED;
    camera_controller.zoom_speed = 2.0;
    camera_controller.deadzone_radius = super::CAMERA_DEADZONE;

    // Spawn camera with the correct modern Bevy components
    commands.spawn((
        Name::new("Gameplay Camera"),
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 999.0)),
        camera_controller,
        camera_bounds,
        StateScoped(Screen::Gameplay),
    ));
}

/// System to set up a loading screen camera
pub fn setup_loading_camera(
    mut commands: Commands,
    existing_cameras: Query<Entity, With<Camera2d>>,
) {
    for camera_entity in &existing_cameras {
        commands.entity(camera_entity).despawn();
    }

    commands.spawn((
        Name::new("Loading Camera"),
        Camera2d,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        StateScoped(Screen::Loading),
    ));

    info!("Loading camera spawned");
}

/// System to update camera targets using ViewportCalculator for multiple targets or simple follow for single target
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

        if targets.len() == 1 {
            // Single player - just follow them
            camera_controller.target_position = targets[0].0.translation.xy();
            camera_controller.target_zoom = super::DEFAULT_CAMERA_ZOOM;
        } else {
            // Multiple players - use ViewportCalculator to include all
            let viewport_calculator = ViewportCalculator::new(camera_settings.zoom_margin);
            let transforms: Vec<&Transform> = targets.iter().map(|(t, _)| *t).collect();

            let base_viewport = Vec2::new(super::BASE_VIEWPORT_WIDTH, super::BASE_VIEWPORT_HEIGHT);

            if let Some((_center_pos, calculated_scale)) =
                viewport_calculator.calculate_from_transforms(&transforms, base_viewport)
            {
                // Calculate weighted average position for smooth following
                let (total_weight, weighted_position) = targets.iter().fold(
                    (0.0, Vec2::ZERO),
                    |(total_weight, weighted_pos), (transform, target)| {
                        (
                            total_weight + target.weight,
                            weighted_pos + transform.translation.xy() * target.weight,
                        )
                    },
                );

                if total_weight > 0.0 {
                    camera_controller.target_position = weighted_position / total_weight;
                }

                if camera_settings.auto_zoom {
                    // For Bevy's Transform::scale on cameras:
                    // - scale > 1.0 = zoomed out (see more)
                    // - scale < 1.0 = zoomed in (see less)

                    // When calculated_scale < 1.0, we need to zoom out
                    // So we need to INVERT the scale for Transform
                    let target_zoom = if calculated_scale < 1.0 {
                        // Players are spreading out, zoom out (increase transform scale)
                        (1.0 / calculated_scale)
                            .max(1.0) // Don't go below 1.0 for zoom out
                            .min(1.0 / super::MIN_CAMERA_ZOOM) // Respect max zoom out
                    } else {
                        // Players are together, can zoom in (decrease transform scale)
                        calculated_scale
                            .max(super::MIN_CAMERA_ZOOM)
                            .min(super::MAX_CAMERA_ZOOM)
                    };

                    camera_controller.target_zoom = target_zoom;
                }
            }
        }
    }
}

/// System to smoothly move camera to target position and zoom using Transform::scale
pub fn update_camera_follow(
    time: Res<Time>,
    camera_settings: Res<CameraSettings>,
    mut camera_query: Query<(&mut Transform, &mut CameraController, &CameraBounds), With<Camera>>,
) {
    for (mut transform, mut controller, bounds) in camera_query.iter_mut() {
        if !controller.is_following {
            continue;
        }

        // Handle position following
        let current_pos = transform.translation.xy();
        let target_pos = controller.target_position;
        let distance_to_target = current_pos.distance(target_pos);

        if distance_to_target > controller.deadzone_radius {
            let direction = (target_pos - current_pos).normalize_or_zero();
            let movement_distance = distance_to_target - controller.deadzone_radius;

            if camera_settings.smooth_follow {
                let target_velocity = direction * movement_distance * controller.follow_speed;
                controller.current_velocity = controller
                    .current_velocity
                    .lerp(target_velocity, time.delta_secs() * controller.follow_speed);

                let new_position = current_pos + controller.current_velocity * time.delta_secs();
                let clamped_position = bounds.clamp_position(new_position);

                transform.translation.x = clamped_position.x;
                transform.translation.y = clamped_position.y;
            } else {
                let new_position = current_pos + direction * movement_distance;
                let clamped_position = bounds.clamp_position(new_position);

                transform.translation.x = clamped_position.x;
                transform.translation.y = clamped_position.y;
            }
        } else {
            controller.current_velocity *= 0.9;
        }

        // Handle zoom following using Transform::scale
        let current_zoom = transform.scale.x; // Assuming uniform scale
        let target_zoom = controller
            .target_zoom
            .clamp(super::MIN_CAMERA_ZOOM, super::MAX_CAMERA_ZOOM);
        let zoom_difference = (current_zoom - target_zoom).abs();

        if zoom_difference > 0.01 {
            let zoom_speed = controller.zoom_speed;
            let new_zoom = current_zoom.lerp(target_zoom, time.delta_secs() * zoom_speed);
            let clamped_zoom = new_zoom.clamp(super::MIN_CAMERA_ZOOM, super::MAX_CAMERA_ZOOM);

            transform.scale = Vec3::splat(clamped_zoom);

            // Debug: show what's happening
            if zoom_difference > 0.1 {
                info!(
                    "Camera zoom: current={:.3}, target={:.3}, new={:.3}",
                    current_zoom, target_zoom, clamped_zoom
                );
            }
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

    for mut bounds in camera_query.iter_mut() {
        let new_bounds = CameraBounds::from_map_size(
            grid_map.world_width(),
            grid_map.world_height(),
            camera_settings.zoom_margin,
        );

        *bounds = new_bounds;
        info!("Camera bounds updated for new map size");
    }
}
