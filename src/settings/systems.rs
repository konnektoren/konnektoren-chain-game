use super::components::*;
use bevy::prelude::*;
use konnektoren_bevy::input::{
    InputDeviceAssignment,
    device::{AvailableInputDevices, InputDevice, KeyboardScheme},
};
use std::collections::HashSet;

/// Resource to track already warned about device assignments
#[derive(Resource, Default)]
pub struct DeviceWarningTracker {
    warned_combinations: HashSet<(String, String)>, // (player_name, device_name)
}

/// System to detect available input devices
pub fn detect_input_devices(
    mut available_devices: ResMut<AvailableInputDevices>,
    gamepads: Query<Entity, With<Gamepad>>,
    mut warning_tracker: ResMut<DeviceWarningTracker>,
) {
    let old_gamepad_count = available_devices.gamepads.len();
    available_devices.gamepads.clear();
    for gamepad_entity in gamepads.iter() {
        available_devices.gamepads.push(gamepad_entity);
    }

    let new_gamepad_count = available_devices.gamepads.len();

    if old_gamepad_count != new_gamepad_count {
        info!(
            "Gamepad count changed: {} -> {}",
            old_gamepad_count, new_gamepad_count
        );

        // Clear warning tracker when gamepad availability changes
        warning_tracker.warned_combinations.clear();
    }

    // Always assume keyboard, mouse available on PC platforms
    #[cfg(not(target_family = "wasm"))]
    {
        available_devices.keyboard = true;
        available_devices.mouse = true;
    }

    // On web, we might need to detect these differently
    #[cfg(target_family = "wasm")]
    {
        available_devices.keyboard = true;
        available_devices.mouse = true;
        available_devices.touch = true;
    }
}

/// System to automatically assign input devices based on availability
pub fn auto_assign_input_devices(
    mut game_settings: ResMut<GameSettings>,
    available_devices: Res<AvailableInputDevices>,
    mut assignment: ResMut<InputDeviceAssignment>,
) {
    // Don't auto-assign if auto-assign is disabled
    if !game_settings.multiplayer.auto_assign_inputs {
        return;
    }

    // Only run auto-assignment on startup or when devices change, not when game settings change
    // This prevents overriding manual selections
    if !available_devices.is_changed() {
        return;
    }

    // Don't auto-assign if there are already manual assignments
    if !assignment.assignments.is_empty() {
        return;
    }

    let gamepad_count = available_devices.gamepads.len();

    // Auto-adjust player count based on available devices
    if game_settings.multiplayer.auto_detect_players {
        let suggested_players = calculate_suggested_player_count(gamepad_count, &available_devices);

        if suggested_players != game_settings.multiplayer.player_count {
            info!(
                "Auto-adjusting player count from {} to {} based on available devices",
                game_settings.multiplayer.player_count, suggested_players
            );
            game_settings
                .multiplayer
                .set_player_count(suggested_players);
        }
    }

    // Auto-assign input devices only if no manual assignments exist
    assign_optimal_input_configuration(
        &mut game_settings.multiplayer,
        &available_devices,
        &mut assignment,
    );
}

fn calculate_suggested_player_count(
    gamepad_count: usize,
    available_devices: &AvailableInputDevices,
) -> usize {
    if gamepad_count == 0 {
        // No gamepads: keyboard only (WASD + Arrow keys for 2 players max)
        if available_devices.keyboard { 2 } else { 1 }
    } else if gamepad_count == 1 {
        // One gamepad: gamepad + keyboard
        2
    } else {
        // Multiple gamepads: all gamepads + keyboard player
        (gamepad_count + 1).min(super::MAX_PLAYERS)
    }
}

fn assign_optimal_input_configuration(
    multiplayer_settings: &mut MultiplayerSettings,
    available_devices: &AvailableInputDevices,
    assignment: &mut InputDeviceAssignment,
) {
    let player_count = multiplayer_settings.player_count;

    // Clear existing assignments
    assignment.assignments.clear();

    if player_count == 1 {
        // Single player: prioritize touch on mobile platforms
        let player = &mut multiplayer_settings.players[0];

        #[cfg(target_family = "wasm")]
        {
            if available_devices.touch {
                player.input.primary_input = InputDevice::Touch;
                player.input.secondary_input = Some(InputDevice::Keyboard(KeyboardScheme::WASD));
            } else {
                player.input.primary_input = InputDevice::Keyboard(KeyboardScheme::WASD);
                player.input.secondary_input = Some(InputDevice::Mouse);
            }
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let gamepad_count = available_devices.gamepads.len();
            player.input.primary_input = InputDevice::Keyboard(KeyboardScheme::WASD);
            player.input.secondary_input = if gamepad_count > 0 {
                Some(InputDevice::Gamepad(0))
            } else {
                Some(InputDevice::Mouse)
            };
        }

        player.input.allow_multiple_devices = true;

        assignment.assign_device(0, player.input.primary_input.clone());
        if let Some(ref secondary) = player.input.secondary_input {
            assignment.assign_device(0, secondary.clone());
        }
    } else {
        // Multiplayer: assign unique devices to each player
        assign_multiplayer_devices(multiplayer_settings, available_devices, assignment);
    }
}

fn assign_multiplayer_devices(
    multiplayer_settings: &mut MultiplayerSettings,
    available_devices: &AvailableInputDevices,
    assignment: &mut InputDeviceAssignment,
) {
    let player_count = multiplayer_settings.player_count;
    let gamepad_count = available_devices.gamepads.len();

    for (player_index, player) in multiplayer_settings.players.iter_mut().enumerate() {
        player.input.allow_multiple_devices = false;
        player.input.secondary_input = None;

        let device = if player_index < gamepad_count {
            // Assign gamepads first
            InputDevice::Gamepad(player_index as u32)
        } else {
            // Assign keyboard schemes for remaining players
            match player_index - gamepad_count {
                0 => InputDevice::Keyboard(KeyboardScheme::WASD),
                1 => InputDevice::Keyboard(KeyboardScheme::Arrows),
                2 => InputDevice::Keyboard(KeyboardScheme::IJKL),
                _ => InputDevice::Mouse, // Fallback
            }
        };

        player.input.primary_input = device.clone();
        assignment.assign_device(player.player_id, device);
    }

    info!(
        "Assigned input devices for {} players with {} gamepads",
        player_count, gamepad_count
    );
}

/// System to mark manual assignments and prevent auto-override
pub fn track_manual_assignments(
    assignment: Res<InputDeviceAssignment>,
    mut game_settings: ResMut<GameSettings>,
) {
    // If there are any assignments, disable auto-assign to prevent conflicts
    if !assignment.assignments.is_empty() && game_settings.multiplayer.auto_assign_inputs {
        info!("Disabling auto-assign due to manual device selection");
        game_settings.multiplayer.auto_assign_inputs = false;
    }
}
