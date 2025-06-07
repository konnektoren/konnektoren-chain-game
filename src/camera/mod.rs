use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraController>();
    app.register_type::<CameraTarget>();
    app.register_type::<CameraSettings>();
    app.register_type::<CameraBounds>();

    app.init_resource::<CameraSettings>();

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_gameplay_camera,
    );

    app.add_systems(
        Update,
        (
            update_camera_targets.in_set(crate::AppSystems::Update),
            update_camera_follow.in_set(crate::AppSystems::Update),
            update_camera_bounds.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Camera configuration constants
pub const DEFAULT_CAMERA_SPEED: f32 = 5.0;
pub const DEFAULT_CAMERA_ZOOM: f32 = 1.0;
pub const MIN_CAMERA_ZOOM: f32 = 0.5;
pub const MAX_CAMERA_ZOOM: f32 = 3.0;
pub const CAMERA_DEADZONE: f32 = 50.0;
pub const MULTI_PLAYER_PADDING: f32 = 100.0;
