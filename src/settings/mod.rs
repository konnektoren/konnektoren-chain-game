use bevy::prelude::*;

mod components;
mod systems;
mod ui;

pub use components::*;
use systems::*;
use ui::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GameSettings>();
    app.register_type::<PlayerSettings>();
    app.register_type::<InputSettings>();
    app.register_type::<MultiplayerSettings>();
    app.register_type::<AvailableInputDevices>();
    app.register_type::<InputDeviceAssignment>();

    // Initialize resources
    app.init_resource::<GameSettings>();
    app.init_resource::<AvailableInputDevices>();
    app.init_resource::<InputDeviceAssignment>();

    app.add_systems(
        Update,
        (
            detect_input_devices,
            auto_assign_input_devices,
            validate_player_configurations,
        ),
    );

    // UI systems
    app.add_systems(
        Update,
        (update_settings_ui, handle_input_device_selection)
            .run_if(in_state(crate::menus::Menu::Settings)),
    );
}

// Configuration constants
pub const MAX_PLAYERS: usize = 4;
pub const DEFAULT_PLAYER_COUNT: usize = 1;
