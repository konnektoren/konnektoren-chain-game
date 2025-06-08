use bevy::prelude::*;

/// Main camera controller component
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraController {
    pub target_position: Vec2,
    pub current_velocity: Vec2,
    pub follow_speed: f32,
    pub zoom_speed: f32,
    pub target_zoom: f32,
    pub deadzone_radius: f32,
    pub is_following: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            target_position: Vec2::ZERO,
            current_velocity: Vec2::ZERO,
            follow_speed: super::DEFAULT_CAMERA_SPEED,
            zoom_speed: 2.0,
            target_zoom: super::DEFAULT_CAMERA_ZOOM,
            deadzone_radius: super::CAMERA_DEADZONE,
            is_following: true,
        }
    }
}

/// Marker component for entities that the camera should follow
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraTarget {
    pub weight: f32, // For weighted average when multiple targets
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self { weight: 1.0 }
    }
}

/// Resource for camera settings
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CameraSettings {
    pub smooth_follow: bool,
    pub auto_zoom: bool,
    pub respect_bounds: bool,
    pub max_follow_distance: f32,
    pub zoom_margin: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            smooth_follow: true,
            auto_zoom: true,
            respect_bounds: true,
            max_follow_distance: 1000.0,
            zoom_margin: super::DEFAULT_ZOOM_MARGIN,
        }
    }
}

/// Component to define camera movement bounds
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct CameraBounds {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub enabled: bool,
}

impl CameraBounds {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            enabled: true,
        }
    }

    pub fn from_map_size(map_width: f32, map_height: f32, padding: f32) -> Self {
        let half_width = map_width / 2.0;
        let half_height = map_height / 2.0;
        Self::new(
            -half_width - padding,
            half_width + padding,
            -half_height - padding,
            half_height + padding,
        )
    }

    pub fn clamp_position(&self, position: Vec2) -> Vec2 {
        if !self.enabled {
            return position;
        }

        Vec2::new(
            position.x.clamp(self.min_x, self.max_x),
            position.y.clamp(self.min_y, self.max_y),
        )
    }
}
