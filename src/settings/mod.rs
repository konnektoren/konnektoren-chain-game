use bevy::prelude::*;
use konnektoren_bevy::input::device::AvailableInputDevices;

mod components;
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
        .register_type::<DeviceSelectionState>();

    // Initialize resources
    app.init_resource::<GameSettings>()
        .init_resource::<AvailableInputDevices>()
        .init_resource::<DeviceSelectionState>()
        .init_resource::<DeviceWarningTracker>();

    // Only input device systems
    app.add_systems(
        Update,
        (
            detect_input_devices,
            auto_assign_input_devices,
            track_manual_assignments,
        ),
    );
}

pub const MAX_PLAYERS: usize = 4;
