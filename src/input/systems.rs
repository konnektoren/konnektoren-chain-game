use super::components::*;
use crate::screens::Screen;
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
            controller.movement_input = movement;
            controller.input_source = InputSource::Keyboard;
        }

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

        if movement.length() > gamepad_settings.deadzone {
            controller.movement_input = movement;
            controller.input_source = InputSource::Gamepad(gamepad_entity);
        }

        // Handle action input
        controller.action_input.pause = gamepad.just_pressed(mapping.pause);
        controller.action_input.interact = gamepad.just_pressed(mapping.interact);
    }
}

/// System to handle mouse input
pub fn handle_mouse_input(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_events: EventReader<CursorMoved>,
    mut joystick_state: ResMut<VirtualJoystickState>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Handle mouse button press/release
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Get cursor position and flip Y coordinate
        if let Some(cursor_pos) = cursor_events.read().last().map(|e| {
            Vec2::new(e.position.x, window.height() - e.position.y) // Flip Y
        }) {
            joystick_state.start_input(cursor_pos, None);
        }
    } else if mouse_buttons.just_released(MouseButton::Left) {
        joystick_state.end_input();
    }

    // Handle mouse movement while pressed
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Some(cursor_pos) = cursor_events.read().last().map(|e| {
            Vec2::new(e.position.x, window.height() - e.position.y) // Flip Y
        }) {
            joystick_state.update_input(cursor_pos);
        }
    }

    // Update controller with joystick state
    for (mut controller, input_mapping) in &mut controller_query {
        if !input_mapping.mouse_enabled {
            continue;
        }

        if joystick_state.is_active {
            controller.movement_input = joystick_state.movement_vector;
            controller.input_source = InputSource::VirtualJoystick;
        }
    }
}

/// System to handle touch input
pub fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,
    mut joystick_state: ResMut<VirtualJoystickState>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    for touch in touch_events.read() {
        match touch.phase {
            bevy::input::touch::TouchPhase::Started => {
                // Flip Y coordinate for touch as well
                let flipped_pos = Vec2::new(touch.position.x, window.height() - touch.position.y);
                joystick_state.start_input(flipped_pos, Some(touch.id));
            }
            bevy::input::touch::TouchPhase::Moved => {
                if joystick_state.touch_id == Some(touch.id) {
                    let flipped_pos =
                        Vec2::new(touch.position.x, window.height() - touch.position.y);
                    joystick_state.update_input(flipped_pos);
                }
            }
            bevy::input::touch::TouchPhase::Ended | bevy::input::touch::TouchPhase::Canceled => {
                if joystick_state.touch_id == Some(touch.id) {
                    joystick_state.end_input();
                }
            }
        }
    }

    // Update controller with joystick state
    for (mut controller, input_mapping) in &mut controller_query {
        if !input_mapping.touch_enabled {
            continue;
        }

        if joystick_state.is_active {
            controller.movement_input = joystick_state.movement_vector;
            controller.input_source = InputSource::Touch;
        }
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

/// System to set up virtual joystick UI
pub fn setup_virtual_joystick(mut commands: Commands) {
    let joystick_size = super::VIRTUAL_JOYSTICK_RADIUS * 2.0;
    let knob_size = super::VIRTUAL_JOYSTICK_KNOB_SIZE * 2.0;

    commands.spawn((
        Name::new("Virtual Joystick Container"),
        VirtualJoystick,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(50.0), // Changed from left to right
            width: Val::Px(joystick_size),
            height: Val::Px(joystick_size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Visibility::Visible, // Always visible initially for testing
        StateScoped(Screen::Gameplay),
        children![
            // Base circle (outer ring)
            (
                Name::new("Joystick Base"),
                VirtualJoystickBase,
                Node {
                    width: Val::Px(joystick_size),
                    height: Val::Px(joystick_size),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.2)),
                BorderRadius::all(Val::Px(joystick_size / 2.0)),
                BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
                children![
                    // Knob circle (inner circle)
                    (
                        Name::new("Joystick Knob"),
                        VirtualJoystickKnob,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(knob_size),
                            height: Val::Px(knob_size),
                            left: Val::Px((joystick_size - knob_size) / 2.0), // Center initially
                            top: Val::Px((joystick_size - knob_size) / 2.0),  // Center initially
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
                        BorderRadius::all(Val::Px(knob_size / 2.0)),
                    ),
                ],
            ),
        ],
    ));

    info!("Virtual joystick UI created on the right side");
}

/// System to update virtual joystick visual position
pub fn update_virtual_joystick_visual(
    joystick_state: Res<VirtualJoystickState>,
    mut knob_query: Query<&mut Node, With<VirtualJoystickKnob>>,
) {
    if !joystick_state.is_changed() {
        return;
    }

    let joystick_size = super::VIRTUAL_JOYSTICK_RADIUS * 2.0;
    let knob_size = super::VIRTUAL_JOYSTICK_KNOB_SIZE * 2.0;

    for mut node in &mut knob_query {
        if joystick_state.is_active {
            // Calculate knob position based on movement vector
            let offset = joystick_state.movement_vector
                * (super::VIRTUAL_JOYSTICK_RADIUS - super::VIRTUAL_JOYSTICK_KNOB_SIZE);

            // Convert to UI coordinates (center is at joystick_size/2)
            let center_offset = (joystick_size - knob_size) / 2.0;
            node.left = Val::Px(center_offset + offset.x);
            node.top = Val::Px(center_offset - offset.y); // Flip Y for UI coordinates
        } else {
            // Reset to center when not active
            let center_offset = (joystick_size - knob_size) / 2.0;
            node.left = Val::Px(center_offset);
            node.top = Val::Px(center_offset);
        }
    }
}

/// System to toggle virtual joystick visibility based on input source
pub fn toggle_virtual_joystick_visibility(
    controller_query: Query<&InputController, With<crate::player::Player>>,
    gamepad_settings: Res<CustomGamepadSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut joystick_query: Query<&mut Visibility, With<VirtualJoystick>>,
    joystick_state: Res<VirtualJoystickState>,
) {
    let mut should_show_joystick = true;

    // Check if keyboard is being used (any key pressed)
    if keyboard.get_pressed().count() > 0 {
        should_show_joystick = false;
    }

    // Check if gamepad is being used
    if !gamepad_settings.connected_gamepads.is_empty() {
        for controller in &controller_query {
            if matches!(controller.input_source, InputSource::Gamepad(_))
                && controller.movement_input.length() > 0.1
            {
                should_show_joystick = false;
                break;
            }
        }
    }

    // Always show if touch/mouse is active
    if joystick_state.is_active {
        should_show_joystick = true;
    }

    // Update joystick visibility
    for mut visibility in &mut joystick_query {
        *visibility = if should_show_joystick {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
