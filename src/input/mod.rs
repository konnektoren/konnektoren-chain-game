use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InputController>();
    app.register_type::<PlayerInputMapping>();
    app.register_type::<CustomGamepadSettings>();
    app.register_type::<VirtualJoystickState>();
    app.register_type::<VirtualJoystick>();
    app.register_type::<VirtualJoystickBase>();
    app.register_type::<VirtualJoystickKnob>();

    // Initialize resources
    app.init_resource::<CustomGamepadSettings>();
    app.init_resource::<VirtualJoystickState>();

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_virtual_joystick,
    );

    app.add_systems(
        Update,
        (
            detect_gamepads,
            assign_gamepads_to_players,
            (
                handle_keyboard_input,
                handle_gamepad_input,
                handle_mouse_input,
                handle_touch_input,
                update_virtual_joystick_visual,
                toggle_virtual_joystick_visibility,
            )
                .in_set(crate::AppSystems::RecordInput),
        )
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const GAMEPAD_DEADZONE: f32 = 0.2;
pub const GAMEPAD_MOVE_THRESHOLD: f32 = 0.5;
pub const VIRTUAL_JOYSTICK_RADIUS: f32 = 80.0;
pub const VIRTUAL_JOYSTICK_DEADZONE: f32 = 10.0;
pub const VIRTUAL_JOYSTICK_KNOB_SIZE: f32 = 30.0;
