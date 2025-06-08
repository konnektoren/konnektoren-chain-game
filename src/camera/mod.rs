use bevy::prelude::*;

mod components;
mod systems;
mod viewport;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraController>();
    app.register_type::<CameraTarget>();
    app.register_type::<CameraSettings>();
    app.register_type::<CameraBounds>();

    app.init_resource::<CameraSettings>();

    // Set up cameras for different screens
    app.add_systems(OnEnter(crate::screens::Screen::Title), setup_title_camera);

    app.add_systems(
        OnEnter(crate::screens::Screen::Loading),
        setup_loading_camera,
    );

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_gameplay_camera,
    );

    // Only run camera follow systems during gameplay
    app.add_systems(
        Update,
        (
            update_camera_targets,
            update_camera_follow,
            update_camera_bounds,
        )
            .in_set(crate::AppSystems::Update)
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Camera configuration constants - adjusted for Transform::scale behavior
pub const DEFAULT_CAMERA_SPEED: f32 = 3.0;
pub const DEFAULT_CAMERA_ZOOM: f32 = 1.0; // 1.0 = normal zoom for Transform::scale
pub const MIN_CAMERA_ZOOM: f32 = 0.5; // 0.5 = zoomed in (see less)
pub const MAX_CAMERA_ZOOM: f32 = 5.0; // 5.0 = zoomed out (see more)
pub const CAMERA_DEADZONE: f32 = 15.0;
pub const MULTI_PLAYER_PADDING: f32 = 200.0; // For map bounds padding

// Viewport constants for viewport calculator
pub const BASE_VIEWPORT_WIDTH: f32 = 800.0;
pub const BASE_VIEWPORT_HEIGHT: f32 = 600.0;
pub const DEFAULT_ZOOM_MARGIN: f32 = 150.0;
