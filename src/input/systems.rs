use super::components::*;
use crate::screens::Screen;
use crate::settings::GameSettings;
use bevy::prelude::*;
use konnektoren_bevy::input::{
    InputDeviceAssignment,
    device::{AvailableInputDevices, InputDevice},
};

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

/// System to handle keyboard input for multiple players
pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
    mut joystick_state: ResMut<VirtualJoystickState>,
) {
    for (mut controller, input_mapping) in &mut controller_query {
        let Some(keyboard_scheme) = &input_mapping.keyboard_scheme else {
            continue;
        };

        // Get keys for this player's scheme
        let (up, down, left, right) = keyboard_scheme.get_keys();

        // Handle continuous movement input
        let mut movement = Vec2::ZERO;
        if keyboard.pressed(up) {
            movement.y += 1.0;
        }
        if keyboard.pressed(down) {
            movement.y -= 1.0;
        }
        if keyboard.pressed(left) {
            movement.x -= 1.0;
        }
        if keyboard.pressed(right) {
            movement.x += 1.0;
        }

        // Normalize diagonal movement
        if movement != Vec2::ZERO {
            movement = movement.normalize();
            controller.movement_input = movement;
            controller.input_source = InputSource::Keyboard(keyboard_scheme.clone());

            // Update virtual joystick for player 0 only (for visual feedback)
            if input_mapping.player_id == 0 {
                joystick_state.movement_vector = movement;
                joystick_state.is_active = true;
            }
        } else if matches!(controller.input_source, InputSource::Keyboard(_)) {
            controller.movement_input = Vec2::ZERO;
            if input_mapping.player_id == 0 {
                joystick_state.movement_vector = Vec2::ZERO;
                joystick_state.is_active = false;
            }
        }

        // Handle action input (use common keys for all players for now)
        controller.action_input.pause =
            keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyP);
        controller.action_input.interact =
            keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::Space);
    }
}

/// System to handle gamepad input for multiple players
pub fn handle_gamepad_input(
    gamepads: Query<(Entity, &Gamepad)>,
    mut controller_query: Query<(&mut InputController, &PlayerInputMapping)>,
    mut joystick_state: ResMut<VirtualJoystickState>,
) {
    for (mut controller, input_mapping) in &mut controller_query {
        let Some(gamepad_entity) = input_mapping.gamepad_entity else {
            continue;
        };

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
        if left_stick.length() > super::GAMEPAD_DEADZONE {
            movement += left_stick;
        }

        // Normalize and clamp movement
        if movement.length() > 1.0 {
            movement = movement.normalize();
        }

        if movement.length() > super::GAMEPAD_DEADZONE {
            controller.movement_input = movement;
            controller.input_source = InputSource::Gamepad(gamepad_entity);

            // Update virtual joystick for player 0 only
            if input_mapping.player_id == 0 {
                joystick_state.movement_vector = movement;
                joystick_state.is_active = true;
            }
        } else if matches!(controller.input_source, InputSource::Gamepad(_)) {
            controller.movement_input = Vec2::ZERO;
            if input_mapping.player_id == 0 {
                joystick_state.movement_vector = Vec2::ZERO;
                joystick_state.is_active = false;
            }
        }

        // Handle action input
        controller.action_input.pause = gamepad.just_pressed(GamepadButton::Start);
        controller.action_input.interact = gamepad.just_pressed(GamepadButton::South);
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
    joystick_query: Query<&Node, With<VirtualJoystick>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Get camera for coordinate conversion
    let Ok((camera, camera_transform)) = cameras.single() else {
        return;
    };

    for touch in touch_events.read() {
        // Convert touch position to world coordinates
        let touch_world_pos = camera
            .viewport_to_world_2d(camera_transform, touch.position)
            .unwrap_or(touch.position);

        match touch.phase {
            bevy::input::touch::TouchPhase::Started => {
                // Check if touch is within joystick area or start new joystick interaction
                let should_start_joystick = if let Ok(joystick_node) = joystick_query.single() {
                    // Calculate joystick world position (this is simplified - you might need more complex calculations)
                    is_touch_in_joystick_area(touch.position, window, joystick_node)
                } else {
                    true // If no specific joystick area, allow anywhere
                };

                if should_start_joystick {
                    joystick_state.start_input(touch_world_pos, Some(touch.id));
                    joystick_state.max_distance = super::VIRTUAL_JOYSTICK_RADIUS;
                }
            }
            bevy::input::touch::TouchPhase::Moved => {
                if joystick_state.touch_id == Some(touch.id) {
                    joystick_state.update_input(touch_world_pos);
                }
            }
            bevy::input::touch::TouchPhase::Ended | bevy::input::touch::TouchPhase::Canceled => {
                if joystick_state.touch_id == Some(touch.id) {
                    joystick_state.end_input();
                }
            }
        }
    }

    // Update controller with joystick state for touch-enabled players
    for (mut controller, input_mapping) in &mut controller_query {
        if !input_mapping.touch_enabled {
            continue;
        }

        if joystick_state.is_active {
            controller.movement_input = joystick_state.movement_vector;
            controller.input_source = InputSource::Touch;
        } else if matches!(controller.input_source, InputSource::Touch) {
            controller.movement_input = Vec2::ZERO;
        }
    }
}

// Update the assign_gamepads_to_players system:
pub fn assign_gamepads_to_players(
    mut player_query: Query<&mut PlayerInputMapping, With<crate::player::Player>>,
    game_settings: Res<GameSettings>,
    available_devices: Res<AvailableInputDevices>,
    assignment: Res<InputDeviceAssignment>,
) {
    if !game_settings.is_changed() && !available_devices.is_changed() && !assignment.is_changed() {
        return;
    }

    for mut input_mapping in &mut player_query {
        let player_id = input_mapping.player_id as usize;

        if let Some(player_settings) = game_settings.multiplayer.players.get(player_id) {
            // Clear previous assignments
            input_mapping.keyboard_scheme = None;
            input_mapping.gamepad_entity = None;
            input_mapping.mouse_enabled = false;
            input_mapping.touch_enabled = false;

            // Assign primary input device
            match &player_settings.input.primary_input {
                InputDevice::Keyboard(scheme) => {
                    input_mapping.keyboard_scheme = Some(scheme.clone());
                }
                InputDevice::Gamepad(gamepad_id) => {
                    if let Some(gamepad_entity) =
                        available_devices.gamepads.get(*gamepad_id as usize)
                    {
                        input_mapping.gamepad_entity = Some(*gamepad_entity);
                    }
                }
                InputDevice::Mouse => {
                    input_mapping.mouse_enabled = true;
                }
                InputDevice::Touch => {
                    input_mapping.touch_enabled = true;
                }
            }

            // Assign secondary input device (for single player)
            if let Some(ref secondary_device) = player_settings.input.secondary_input {
                match secondary_device {
                    InputDevice::Keyboard(scheme) => {
                        if input_mapping.keyboard_scheme.is_none() {
                            input_mapping.keyboard_scheme = Some(scheme.clone());
                        }
                    }
                    InputDevice::Gamepad(gamepad_id) => {
                        if input_mapping.gamepad_entity.is_none() {
                            if let Some(gamepad_entity) =
                                available_devices.gamepads.get(*gamepad_id as usize)
                            {
                                input_mapping.gamepad_entity = Some(*gamepad_entity);
                            }
                        }
                    }
                    InputDevice::Mouse => {
                        input_mapping.mouse_enabled = true;
                    }
                    InputDevice::Touch => {
                        input_mapping.touch_enabled = true;
                    }
                }
            }
        }
    }
}

/// System to set up virtual joystick UI
pub fn setup_virtual_joystick(mut commands: Commands) {
    let joystick_size = super::VIRTUAL_JOYSTICK_RADIUS * 2.0;
    let knob_size = super::VIRTUAL_JOYSTICK_KNOB_SIZE * 2.0;

    // Make joystick more prominent on mobile/touch devices
    #[cfg(target_family = "wasm")]
    let (bottom_offset, right_offset, joystick_opacity) = (Val::Px(80.0), Val::Px(80.0), 0.8);

    #[cfg(not(target_family = "wasm"))]
    let (bottom_offset, right_offset, joystick_opacity) = (Val::Px(50.0), Val::Px(50.0), 0.6);

    commands.spawn((
        Name::new("Virtual Joystick Container"),
        VirtualJoystick,
        Node {
            position_type: PositionType::Absolute,
            bottom: bottom_offset,
            right: right_offset,
            width: Val::Px(joystick_size),
            height: Val::Px(joystick_size),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        Visibility::Visible,
        StateScoped(Screen::Gameplay),
        Interaction::default(), // Add interaction for touch detection
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
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, joystick_opacity * 0.3)),
                BorderRadius::all(Val::Px(joystick_size / 2.0)),
                BorderColor(Color::srgba(1.0, 1.0, 1.0, joystick_opacity * 0.6)),
                Interaction::default(), // Add interaction for touch detection
                children![
                    // Knob circle (inner circle)
                    (
                        Name::new("Joystick Knob"),
                        VirtualJoystickKnob,
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(knob_size),
                            height: Val::Px(knob_size),
                            left: Val::Px((joystick_size - knob_size) / 2.0),
                            top: Val::Px((joystick_size - knob_size) / 2.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, joystick_opacity)),
                        BorderRadius::all(Val::Px(knob_size / 2.0)),
                        Interaction::default(), // Add interaction for touch detection
                    ),
                ],
            ),
        ],
    ));

    info!("Virtual joystick UI created for mobile/touch input");
}

/// System to update virtual joystick visual position
pub fn update_virtual_joystick_visual(
    joystick_state: Res<VirtualJoystickState>,
    mut knob_query: Query<&mut Node, With<VirtualJoystickKnob>>,
) {
    let joystick_size = super::VIRTUAL_JOYSTICK_RADIUS * 2.0;
    let knob_size = super::VIRTUAL_JOYSTICK_KNOB_SIZE * 2.0;

    for mut node in &mut knob_query {
        if joystick_state.movement_vector.length() > 0.01 {
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

/// Helper function to check if touch is within joystick area
fn is_touch_in_joystick_area(touch_pos: Vec2, window: &Window, _joystick_node: &Node) -> bool {
    // For now, allow touch anywhere on the right side of the screen
    // You can make this more sophisticated by calculating the actual joystick position
    let right_side_threshold = window.width() * 0.7;
    touch_pos.x > right_side_threshold
}

/// System to handle direct interaction with virtual joystick UI elements
pub fn handle_virtual_joystick_interaction(
    mut interaction_query: Query<
        (&Interaction, &GlobalTransform, &Node),
        (With<VirtualJoystickBase>, Changed<Interaction>),
    >,
    mut joystick_state: ResMut<VirtualJoystickState>,
    mut touch_events: EventReader<TouchInput>,
    windows: Query<&Window>,
) {
    let Ok(_window) = windows.single() else {
        return;
    };

    // Handle UI-based interaction
    for (interaction, global_transform, _node) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => {
                // Find the current touch position and start joystick from center of UI element
                for touch in touch_events.read() {
                    if matches!(touch.phase, bevy::input::touch::TouchPhase::Started) {
                        let joystick_center = global_transform.translation().truncate();
                        joystick_state.start_input(joystick_center, Some(touch.id));
                        joystick_state.max_distance = super::VIRTUAL_JOYSTICK_RADIUS;
                        break;
                    }
                }
            }
            Interaction::Hovered => {
                // Visual feedback when hovering/touching the joystick
            }
            Interaction::None => {
                // Could handle release here if needed
            }
        }
    }
}
