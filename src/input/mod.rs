use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InputController>();
    app.register_type::<PlayerInputMapping>();
    app.register_type::<CustomGamepadSettings>();

    // Initialize resources
    app.init_resource::<CustomGamepadSettings>();

    app.add_systems(
        Update,
        (
            detect_gamepads,
            assign_gamepads_to_players,
            (handle_keyboard_input, handle_gamepad_input).in_set(crate::AppSystems::RecordInput),
        )
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const GAMEPAD_DEADZONE: f32 = 0.2;
pub const GAMEPAD_MOVE_THRESHOLD: f32 = 0.5;
