use crate::settings::*;
use crate::theme::prelude::*;
use bevy::prelude::*;

/// System to spawn the device selection interface
pub fn spawn_device_selection_ui(mut commands: Commands, game_settings: Res<GameSettings>) {
    commands.spawn((
        Name::new("Device Selection Root"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(10),
        DeviceSelectionUI,
        children![
            widget::header("Configure Players"),
            create_player_grid(game_settings.multiplayer.player_count),
            create_instruction_panel(),
        ],
    ));
}

/// Component marker for device selection UI
#[derive(Component)]
pub struct DeviceSelectionUI;

fn create_player_grid(player_count: usize) -> impl Bundle {
    let cols = if player_count <= 2 { 2 } else { 2 };
    let rows = (player_count + 1) / 2;

    (
        Name::new("Player Grid"),
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(cols, 400.0),
            grid_template_rows: RepeatedGridTrack::px(rows, 300.0),
            column_gap: Val::Px(30.0),
            row_gap: Val::Px(30.0),
            ..default()
        },
        PlayerGrid, // Marker component instead of Children::new()
    )
}

/// Marker component for the player grid
#[derive(Component)]
pub struct PlayerGrid;

fn create_instruction_panel() -> impl Bundle {
    (
        Name::new("Instructions"),
        Node {
            padding: UiRect::all(Val::Px(20.0)),
            max_width: Val::Px(600.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
        BorderRadius::all(Val::Px(10.0)),
        children![
            (
                Name::new("Instruction Text"),
                Text("Click on a player panel, then press any key/button on your desired input device to assign it. Press ESC to cancel assignment.".to_string()),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ),
        ],
    )
}

/// System to create player panels dynamically
pub fn update_player_panels(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    available_devices: Res<AvailableInputDevices>,
    assignment: Res<InputDeviceAssignment>,
    selection_state: Res<DeviceSelectionState>,
    grid_query: Query<Entity, With<PlayerGrid>>, // Use the marker component
    existing_panels: Query<Entity, With<PlayerConfigPanel>>,
) {
    if !game_settings.is_changed() && !assignment.is_changed() && !selection_state.is_changed() {
        return;
    }

    // Remove existing panels
    for panel_entity in &existing_panels {
        commands.entity(panel_entity).despawn_recursive();
    }

    // Find the grid entity
    if let Ok(grid_entity) = grid_query.get_single() {
        // Create panels for each player
        for player_id in 0..game_settings.multiplayer.player_count {
            let player_settings = &game_settings.multiplayer.players[player_id];
            let is_selecting = selection_state.selecting_player == Some(player_id);

            let panel_entity = commands
                .spawn(create_player_panel(
                    player_id,
                    player_settings,
                    &assignment,
                    is_selecting,
                ))
                .id();

            commands.entity(grid_entity).add_child(panel_entity);
        }
    }
}

fn create_player_panel(
    player_id: usize,
    player_settings: &PlayerSettings,
    assignment: &InputDeviceAssignment,
    is_selecting: bool,
) -> impl Bundle {
    (
        Name::new(format!("Player {} Panel", player_id + 1)),
        PlayerConfigPanel {
            player_id,
            is_active: player_settings.enabled,
        },
        Button,
        Node {
            width: Val::Px(380.0),
            height: Val::Px(280.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            border: UiRect::all(Val::Px(if is_selecting { 4.0 } else { 2.0 })),
            ..default()
        },
        BackgroundColor(if is_selecting {
            Color::srgba(0.3, 0.6, 1.0, 0.3)
        } else if player_settings.enabled {
            Color::srgba(0.2, 0.4, 0.2, 0.8)
        } else {
            Color::srgba(0.3, 0.3, 0.3, 0.5)
        }),
        BorderColor(if is_selecting {
            Color::srgb(0.5, 0.8, 1.0)
        } else {
            Color::srgb(0.5, 0.5, 0.5)
        }),
        BorderRadius::all(Val::Px(15.0)),
        children![
            // Player header
            (
                Name::new("Player Header"),
                Text(format!("Player {}", player_id + 1)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(player_settings.color),
            ),
            // Status section
            (
                Name::new("Status Section"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                children![
                    // Enable/Disable status
                    (
                        Name::new("Player Status"),
                        Text(if player_settings.enabled {
                            "ACTIVE".to_string()
                        } else {
                            "DISABLED".to_string()
                        }),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(if player_settings.enabled {
                            Color::srgb(0.2, 1.0, 0.2)
                        } else {
                            Color::srgb(1.0, 0.4, 0.4)
                        }),
                    ),
                    // Current device assignment
                    (
                        Name::new("Device Assignment"),
                        Text(get_device_assignment_text(player_id, assignment)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            max_width: Val::Px(300.0),
                            ..default()
                        },
                    ),
                ],
            ),
            // Create action buttons section
            create_action_buttons_section(player_id, player_settings),
        ],
    )
}

// Create a separate function to handle the action buttons with consistent types
fn create_action_buttons_section(
    player_id: usize,
    player_settings: &PlayerSettings,
) -> impl Bundle {
    (
        Name::new("Action Section"),
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        // We'll add the buttons as children manually to avoid type issues
        PlayerActionButtons {
            player_id,
            is_enabled: player_settings.enabled,
        },
    )
}

/// Component to track which buttons should be in an action section
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerActionButtons {
    pub player_id: usize,
    pub is_enabled: bool,
}

/// System to add buttons to action sections after they're created
pub fn setup_action_buttons(
    mut commands: Commands,
    action_buttons_query: Query<(Entity, &PlayerActionButtons), Added<PlayerActionButtons>>,
) {
    for (entity, action_buttons) in &action_buttons_query {
        // Always add the enable/disable button
        let enable_button = commands
            .spawn((
                widget::button_small(
                    if action_buttons.is_enabled {
                        "Disable"
                    } else {
                        "Enable"
                    },
                    create_toggle_action(action_buttons.player_id),
                ),
                PlayerToggleButton {
                    player_id: action_buttons.player_id,
                },
            ))
            .id();

        commands.entity(entity).add_child(enable_button);

        // Add configure button only if enabled
        if action_buttons.is_enabled {
            let configure_button = commands
                .spawn((
                    widget::button_small(
                        "Configure",
                        create_configure_action(action_buttons.player_id),
                    ),
                    PlayerConfigureButton {
                        player_id: action_buttons.player_id,
                    },
                ))
                .id();

            commands.entity(entity).add_child(configure_button);
        }
    }
}

/// Component markers for different button types
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerToggleButton {
    pub player_id: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerConfigureButton {
    pub player_id: usize,
}

fn get_device_assignment_text(player_id: usize, assignment: &InputDeviceAssignment) -> String {
    if let Some(device) = assignment.get_device_for_player(player_id as u32) {
        format!("Device: {}", device.name())
    } else {
        "No device assigned\nClick 'Configure' to assign".to_string()
    }
}

// Create action functions that work properly with the observer system
fn create_toggle_action(
    player_id: usize,
) -> impl Fn(Trigger<Pointer<Click>>) + Send + Sync + 'static {
    move |_trigger: Trigger<Pointer<Click>>| {
        info!("Toggle requested for player {}", player_id + 1);
        // The actual toggle will be handled by a separate system
    }
}

fn create_configure_action(
    player_id: usize,
) -> impl Fn(Trigger<Pointer<Click>>) + Send + Sync + 'static {
    move |_trigger: Trigger<Pointer<Click>>| {
        info!("Configure requested for player {}", player_id + 1);
        // The actual device selection will be handled by a separate system
    }
}

/// System to handle button actions through button component markers
pub fn handle_button_actions(
    mut game_settings: ResMut<GameSettings>,
    mut selection_state: ResMut<DeviceSelectionState>,
    toggle_buttons: Query<(&PlayerToggleButton, &Interaction), Changed<Interaction>>,
    configure_buttons: Query<(&PlayerConfigureButton, &Interaction), Changed<Interaction>>,
) {
    // Handle toggle buttons
    for (toggle_button, interaction) in &toggle_buttons {
        if *interaction == Interaction::Pressed {
            let player_id = toggle_button.player_id;
            if let Some(player_settings) = game_settings.multiplayer.players.get_mut(player_id) {
                player_settings.enabled = !player_settings.enabled;
                info!(
                    "Toggled player {} to {}",
                    player_id + 1,
                    if player_settings.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
        }
    }

    // Handle configure buttons
    for (configure_button, interaction) in &configure_buttons {
        if *interaction == Interaction::Pressed {
            let player_id = configure_button.player_id;
            selection_state.start_selection(player_id);
            info!("Started device selection for player {}", player_id + 1);
        }
    }
}

/// System to handle button clicks on player panels
pub fn handle_panel_interactions(
    mut selection_state: ResMut<DeviceSelectionState>,
    panel_query: Query<(&PlayerConfigPanel, &Interaction), Changed<Interaction>>,
) {
    for (panel, interaction) in &panel_query {
        if *interaction == Interaction::Pressed {
            if selection_state.selecting_player.is_some() {
                // Cancel current selection and start new one
                selection_state.cancel_selection();
            }

            // Start selection for this player
            selection_state.start_selection(panel.player_id);
            info!(
                "Started device selection for player {}",
                panel.player_id + 1
            );
        }
    }
}

/// System to handle device input during selection
pub fn handle_device_selection_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<(Entity, &Gamepad)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut selection_state: ResMut<DeviceSelectionState>,
    mut game_settings: ResMut<GameSettings>,
    available_devices: Res<AvailableInputDevices>,
    mut assignment: ResMut<InputDeviceAssignment>,
) {
    let Some(selecting_player) = selection_state.selecting_player else {
        return;
    };

    // Check for cancellation (ESC key)
    if keyboard.just_pressed(KeyCode::Escape) {
        selection_state.cancel_selection();
        info!(
            "Device selection cancelled for player {}",
            selecting_player + 1
        );
        return;
    }

    // Check for keyboard input (excluding ESC)
    if keyboard
        .get_just_pressed()
        .any(|key| *key != KeyCode::Escape)
    {
        // Determine which keyboard scheme to use
        let scheme = determine_keyboard_scheme(&keyboard);
        let device = InputDevice::Keyboard(scheme.clone());
        selection_state.confirm_selection(device.clone());
        assign_device_to_player(
            selecting_player,
            device,
            &mut game_settings,
            &mut assignment,
        );
        info!(
            "Assigned keyboard ({:?}) to player {}",
            scheme,
            selecting_player + 1
        );
        return;
    }

    // Check for gamepad input
    for (gamepad_entity, gamepad) in &gamepads {
        if gamepad.get_just_pressed().next().is_some() {
            // Find gamepad index
            if let Some(gamepad_index) = available_devices
                .gamepads
                .iter()
                .position(|&e| e == gamepad_entity)
            {
                let device = InputDevice::Gamepad(gamepad_index as u32);
                selection_state.confirm_selection(device.clone());
                assign_device_to_player(
                    selecting_player,
                    device,
                    &mut game_settings,
                    &mut assignment,
                );
                info!(
                    "Assigned gamepad {} to player {}",
                    gamepad_index,
                    selecting_player + 1
                );
                return;
            }
        }
    }

    // Check for mouse input
    if mouse.get_just_pressed().next().is_some() {
        let device = InputDevice::Mouse;
        selection_state.confirm_selection(device.clone());
        assign_device_to_player(
            selecting_player,
            device,
            &mut game_settings,
            &mut assignment,
        );
        info!("Assigned mouse to player {}", selecting_player + 1);
        return;
    }
}

fn determine_keyboard_scheme(keyboard: &ButtonInput<KeyCode>) -> KeyboardScheme {
    // Simple logic to determine scheme based on which key was pressed
    if keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::KeyA)
        || keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::KeyD)
    {
        KeyboardScheme::WASD
    } else if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::ArrowRight)
    {
        KeyboardScheme::Arrows
    } else if keyboard.just_pressed(KeyCode::KeyI)
        || keyboard.just_pressed(KeyCode::KeyJ)
        || keyboard.just_pressed(KeyCode::KeyK)
        || keyboard.just_pressed(KeyCode::KeyL)
    {
        KeyboardScheme::IJKL
    } else {
        // Default to WASD for any other key
        KeyboardScheme::WASD
    }
}

fn assign_device_to_player(
    player_id: usize,
    device: InputDevice,
    game_settings: &mut GameSettings,
    assignment: &mut InputDeviceAssignment,
) {
    if let Some(player_settings) = game_settings.multiplayer.players.get_mut(player_id) {
        player_settings.input.primary_input = device.clone();
        player_settings.input.secondary_input = None; // Clear secondary for explicit assignment
        player_settings.enabled = true; // Ensure player is enabled when device is assigned

        // Update assignment tracking
        assignment.assign_device(player_id as u32, device);
    }
}

/// System to handle selection timeout
pub fn handle_selection_timeout(
    time: Res<Time>,
    mut selection_state: ResMut<DeviceSelectionState>,
) {
    if selection_state.selecting_player.is_some() {
        selection_state.selection_timeout.tick(time.delta());

        if selection_state.selection_timeout.finished() {
            let player_id = selection_state.selecting_player.unwrap();
            selection_state.cancel_selection();
            info!("Device selection timed out for player {}", player_id + 1);
        }
    }
}
