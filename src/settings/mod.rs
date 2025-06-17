use bevy::prelude::*;

mod components;
pub mod device_selection_ui;
pub mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    // Register types
    app.register_type::<GameSettings>()
        .register_type::<PlayerSettings>()
        .register_type::<InputSettings>()
        .register_type::<MultiplayerSettings>()
        .register_type::<AvailableInputDevices>()
        .register_type::<InputDeviceAssignment>()
        .register_type::<DeviceSelectionState>();

    // Initialize resources
    app.init_resource::<GameSettings>()
        .init_resource::<AvailableInputDevices>()
        .init_resource::<InputDeviceAssignment>()
        .init_resource::<DeviceSelectionState>()
        .init_resource::<DeviceWarningTracker>();

    // Only input device systems
    app.add_systems(
        Update,
        (
            detect_input_devices,
            auto_assign_input_devices,
            track_manual_assignments,
            validate_player_configurations,
        ),
    );
}

pub const MAX_PLAYERS: usize = 4;
