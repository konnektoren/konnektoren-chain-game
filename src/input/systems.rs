use super::components::*;
use bevy::prelude::*;

/// System to detect and track connected gamepads
pub fn detect_gamepads(
    mut gamepad_settings: ResMut<CustomGamepadSettings>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    let old_count = gamepad_settings.connected_gamepads.len();

    // Update connected gamepads list
    gamepad_settings.connected_gamepads.clear();
    for gamepad_entity in gamepads.iter() {
        gamepad_settings.connected_gamepads.push(gamepad_entity);
    }

    let new_count = gamepad_settings.connected_gamepads.len();

    if old_count != new_count {
        info!("Gamepad count changed: {} -> {}", old_count, new_count);
        if new_count > 0 {
            info!(
                "Connected gamepads: {:?}",
                gamepad_settings.connected_gamepads
            );
        }
    }
}

/// System to handle keyboard input
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
) {
    let mapping = KeyboardMapping::default();

    for (mut controller, input_mapping) in &mut controller_query {
        if !input_mapping.keyboard_enabled {
            continue;
        }

        // Handle continuous movement input
        let mut movement = Vec2::ZERO;
        if mapping.move_up.iter().any(|key| keyboard.pressed(*key)) {
            movement.y += 1.0;
        }
        if mapping.move_down.iter().any(|key| keyboard.pressed(*key)) {
            movement.y -= 1.0;
        }
        if mapping.move_left.iter().any(|key| keyboard.pressed(*key)) {
            movement.x -= 1.0;
        }
        if mapping.move_right.iter().any(|key| keyboard.pressed(*key)) {
            movement.x += 1.0;
        }

        // Normalize diagonal movement
        if movement != Vec2::ZERO {
            movement = movement.normalize();
        }

        controller.movement_input = movement;
        controller.input_source = InputSource::Keyboard;

        // Handle action input
        controller.action_input.pause = mapping.pause.iter().any(|key| keyboard.just_pressed(*key));
        controller.action_input.interact = mapping
            .interact
            .iter()
            .any(|key| keyboard.just_pressed(*key));
    }
}

/// System to handle gamepad input
pub fn handle_gamepad_input(
    gamepads: Query<(Entity, &Gamepad)>,
    gamepad_settings: Res<CustomGamepadSettings>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
) {
    let mapping = GamepadMapping::default();

    for (mut controller, input_mapping) in &mut controller_query {
        let Some(gamepad_entity) = input_mapping.gamepad_entity else {
            continue;
        };

        // Check if this gamepad is still connected
        if !gamepad_settings
            .connected_gamepads
            .contains(&gamepad_entity)
        {
            continue;
        }

        // Find the gamepad
        let Some((_, gamepad)) = gamepads
            .iter()
            .find(|(entity, _)| *entity == gamepad_entity)
        else {
            continue;
        };

        let mut movement = Vec2::ZERO;

        // D-Pad input
        if gamepad.pressed(GamepadButton::DPadUp) {
            movement.y += 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadDown) {
            movement.y -= 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadLeft) {
            movement.x -= 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadRight) {
            movement.x += 1.0;
        }

        // Analog stick input (with deadzone)
        let left_stick = gamepad.left_stick();
        if left_stick.length() > gamepad_settings.deadzone {
            movement += left_stick;
        }

        // Normalize and clamp movement
        if movement.length() > 1.0 {
            movement = movement.normalize();
        }

        controller.movement_input = movement;
        controller.input_source = InputSource::Gamepad(gamepad_entity);

        // Handle action input
        controller.action_input.pause = gamepad.just_pressed(mapping.pause);
        controller.action_input.interact = gamepad.just_pressed(mapping.interact);
    }
}

/// System to automatically assign gamepads to players
pub fn assign_gamepads_to_players(
    mut player_query: Query<&mut PlayerInputMapping, With<crate::player::Player>>,
    gamepad_settings: Res<CustomGamepadSettings>,
) {
    for mut input_mapping in &mut player_query {
        // If player doesn't have a gamepad assigned and there are gamepads available
        if input_mapping.gamepad_entity.is_none() && !gamepad_settings.connected_gamepads.is_empty()
        {
            // Assign the first available gamepad
            input_mapping.gamepad_entity = Some(gamepad_settings.connected_gamepads[0]);
            info!(
                "Assigned gamepad {:?} to player {}",
                gamepad_settings.connected_gamepads[0], input_mapping.player_id
            );
        }
    }
}
