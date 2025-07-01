use crate::settings::*;
use crate::theme::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use konnektoren_bevy::input::device::{AvailableInputDevices, InputDevice};

/// System to spawn the device selection interface with proper Bevy scrolling
pub fn spawn_device_selection_ui(mut commands: Commands, game_settings: Res<GameSettings>) {
    commands.spawn((
        Name::new("Device Selection Root"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(10),
        DeviceSelectionUI,
        StateScoped(crate::menus::Menu::DeviceSelection),
        Pickable::IGNORE,
        children![
            // Fixed Header
            (
                Name::new("Fixed Header"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                children![
                    widget::header("Configure Players"),
                    create_instruction_panel(),
                ],
            ),
            // Scrollable Container
            (
                Name::new("Scrollable Container"),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0), // Take remaining space
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    overflow: Overflow::scroll_y(), // Enable scrolling
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.3)),
                ScrollPosition::default(), // Add ScrollPosition component
                ScrollableArea,
                children![create_player_grid(game_settings.multiplayer.player_count),],
            ),
            // Fixed Footer
            (
                Name::new("Fixed Footer"),
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    flex_shrink: 0.0,
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::top(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
                BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                children![widget::button("Back to Settings", go_back_to_settings),],
            ),
        ],
    ));
}

#[derive(Component)]
pub struct ScrollableArea;

/// System to handle scrolling in the device selection UI
pub fn handle_scroll_input(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition, With<ScrollableArea>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    const LINE_HEIGHT: f32 = 30.0; // Adjust scroll sensitivity

    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        // Check if we're hovering over a scrollable area
        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= dy;
                    // Clamp scroll position to prevent over-scrolling
                    scroll_position.offset_y = scroll_position.offset_y.max(0.0);
                }
            }
        }
    }

    // Handle keyboard scrolling
    for mut scroll_position in scrolled_node_query.iter_mut() {
        let mut scroll_delta = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            scroll_delta += LINE_HEIGHT * 0.5;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            scroll_delta -= LINE_HEIGHT * 0.5;
        }

        if scroll_delta != 0.0 {
            scroll_position.offset_y -= scroll_delta;
            scroll_position.offset_y = scroll_position.offset_y.max(0.0);
        }
    }
}

fn go_back_to_settings(
    _: Trigger<Pointer<Click>>,
    mut next_menu: ResMut<NextState<crate::menus::Menu>>,
) {
    next_menu.set(crate::menus::Menu::Settings);
}

fn create_player_grid(player_count: usize) -> impl Bundle {
    // Calculate grid layout - 2 players per row
    let cols = 2;
    let rows = player_count.div_ceil(cols);
    let panel_width = if player_count <= 2 { 400.0 } else { 380.0 }; // Slightly smaller for more players

    (
        Name::new("Player Grid"),
        Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(cols, panel_width),
            grid_template_rows: RepeatedGridTrack::auto(rows.try_into().unwrap()),
            column_gap: Val::Px(20.0),
            row_gap: Val::Px(20.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center, // Center the grid
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        PlayerGrid,
        Pickable {
            should_block_lower: false,
            ..default()
        },
    )
}

fn create_instruction_panel() -> impl Bundle {
    (
        Name::new("Instructions"),
        Node {
            padding: UiRect::all(Val::Px(10.0)),
            max_width: Val::Px(600.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8)),
        BorderRadius::all(Val::Px(8.0)),
        children![(
            Name::new("Instruction Text"),
            Text("Use mouse wheel to scroll. Click device buttons to assign.".to_string()),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
        )],
    )
}

/// System to manage player panels
pub fn update_player_panels(
    mut commands: Commands,
    game_settings: Res<GameSettings>,
    assignment: Res<InputDeviceAssignment>,
    grid_query: Query<Entity, With<PlayerGrid>>,
    existing_panels: Query<(Entity, &PlayerConfigPanel)>,
) {
    let should_recreate = (game_settings.is_changed()
        && game_settings.multiplayer.player_count != existing_panels.iter().count())
        || existing_panels.is_empty();

    if !should_recreate {
        return;
    }

    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    // Clean up existing panels
    for (panel_entity, _) in &existing_panels {
        commands.entity(panel_entity).despawn();
    }

    // Create new panels
    for player_id in 0..game_settings.multiplayer.player_count {
        let player_settings = &game_settings.multiplayer.players[player_id];
        let panel_entity = commands
            .spawn(create_player_panel(player_id, player_settings, &assignment))
            .id();
        commands.entity(grid_entity).add_child(panel_entity);
    }
}

fn create_player_panel(
    player_id: usize,
    player_settings: &PlayerSettings,
    assignment: &InputDeviceAssignment,
) -> impl Bundle {
    // Dynamic sizing based on player count - make panels slightly smaller for grid layout
    let panel_width = 380.0; // Consistent width for grid layout

    (
        Name::new(format!("Player {} Panel", player_id + 1)),
        PlayerConfigPanel {
            player_id,
            is_active: player_settings.enabled,
        },
        Node {
            width: Val::Px(panel_width),
            min_height: Val::Px(280.0),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            row_gap: Val::Px(10.0),
            border: UiRect::all(Val::Px(2.0)),
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.8)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        BorderRadius::all(Val::Px(12.0)),
        Pickable {
            should_block_lower: false,
            ..default()
        },
        children![
            // Player header
            (
                Name::new("Player Header"),
                Text(format!("Player {}", player_id + 1)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(player_settings.color),
            ),
            // Current device display
            create_current_device_section(player_id, assignment),
            // Device selection
            create_device_section(player_id),
        ],
    )
}

fn create_current_device_section(
    player_id: usize,
    assignment: &InputDeviceAssignment,
) -> impl Bundle {
    let current_device = assignment.get_device_for_player(player_id as u32);

    (
        Name::new(format!("Current Device Section P{}", player_id)),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(3.0),
            padding: UiRect::all(Val::Px(6.0)),
            border: UiRect::all(Val::Px(1.0)),
            width: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.8)),
        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
        BorderRadius::all(Val::Px(6.0)),
        children![
            (
                Name::new("Current Device Label"),
                Text("Current Device:".to_string()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ),
            (
                Name::new(format!("Current Device Name P{}", player_id)),
                Text(if let Some(device) = current_device {
                    device.name()
                } else {
                    "None selected".to_string()
                }),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(if current_device.is_some() {
                    Color::srgb(0.2, 1.0, 0.2)
                } else {
                    Color::srgb(1.0, 0.4, 0.4)
                }),
            ),
        ],
    )
}

fn create_device_section(player_id: usize) -> impl Bundle {
    (
        Name::new("Device Section"),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        },
        DeviceSectionContainer {
            player_id,
            enabled: true,
        },
    )
}

/// System to setup device section content
pub fn setup_device_section_content(
    mut commands: Commands,
    sections_query: Query<(Entity, &DeviceSectionContainer), Added<DeviceSectionContainer>>,
) {
    for (entity, section) in &sections_query {
        let label = commands
            .spawn((
                Name::new("Device Selection Label"),
                Text("Available Devices:".to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .id();

        let grid = commands
            .spawn((
                Name::new("Device Buttons Grid"),
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(2, 120.0),
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                DeviceButtonsContainer {
                    player_id: section.player_id,
                },
            ))
            .id();

        commands.entity(entity).add_child(label);
        commands.entity(entity).add_child(grid);
    }
}

/// System to create device buttons
pub fn setup_device_buttons(
    mut commands: Commands,
    containers_query: Query<(Entity, &DeviceButtonsContainer), Added<DeviceButtonsContainer>>,
    available_devices: Res<AvailableInputDevices>,
    assignment: Res<InputDeviceAssignment>,
) {
    for (container_entity, container) in &containers_query {
        create_device_buttons_for_container(
            &mut commands,
            container_entity,
            container,
            &available_devices,
            &assignment,
        );
    }
}

fn create_device_buttons_for_container(
    commands: &mut Commands,
    container_entity: Entity,
    container: &DeviceButtonsContainer,
    available_devices: &AvailableInputDevices,
    assignment: &InputDeviceAssignment,
) {
    let player_id = container.player_id;
    let current_device = assignment.get_device_for_player(player_id as u32);
    let devices = available_devices.get_available_devices();

    for device in devices {
        let is_selected = current_device == Some(&device);
        let is_available = device.is_available(available_devices);
        let is_used_by_other = is_device_used_by_other_player(player_id, &device, assignment);

        let (button_color, text_color) =
            get_button_colors(is_selected, is_available, is_used_by_other);

        let device_button = commands
            .spawn((
                Name::new(format!("Device Button: {}", device.name())),
                Button,
                Node {
                    width: Val::Px(110.0),
                    height: Val::Px(30.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 })),
                    ..default()
                },
                BackgroundColor(button_color),
                BorderColor(if is_selected {
                    Color::srgb(0.4, 1.0, 0.4)
                } else {
                    Color::srgb(0.6, 0.6, 0.6)
                }),
                BorderRadius::all(Val::Px(4.0)),
                crate::theme::interaction::InteractionPalette {
                    none: button_color,
                    hovered: lighten_color(button_color, 0.1),
                    pressed: darken_color(button_color, 0.1),
                },
                DeviceButton {
                    device: device.clone(),
                    player_id,
                },
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                children![(
                    Name::new("Device Button Text"),
                    Text(format_device_name(&device)),
                    TextFont {
                        font_size: 9.0,
                        ..default()
                    },
                    TextColor(text_color),
                    Pickable::IGNORE,
                )],
            ))
            .id();

        commands.entity(container_entity).add_child(device_button);
    }
}

/// System to handle device button clicks
pub fn handle_device_button_clicks(
    mut game_settings: ResMut<GameSettings>,
    mut assignment: ResMut<InputDeviceAssignment>,
    device_buttons: Query<(&DeviceButton, &Interaction), (Changed<Interaction>, With<Button>)>,
    available_devices: Res<AvailableInputDevices>,
) {
    for (device_button, interaction) in &device_buttons {
        if *interaction == Interaction::Pressed {
            let player_id = device_button.player_id;
            let device = &device_button.device;

            // Disable auto-assignment to prevent conflicts
            game_settings.multiplayer.auto_assign_inputs = false;

            if !device.is_available(&available_devices) {
                warn!("Device {} is not available", device.name());
                continue;
            }

            if is_device_used_by_other_player(player_id, device, &assignment) {
                warn!("Device {} is already used by another player", device.name());
                continue;
            }

            if let Some(player_settings) = game_settings.multiplayer.players.get_mut(player_id) {
                player_settings.input.primary_input = device.clone();
                player_settings.input.secondary_input = None;
                player_settings.enabled = true;
            }

            assignment.assign_device(player_id as u32, device.clone());
            info!("Assigned {} to player {}", device.name(), player_id + 1);
        }
    }
}

/// System to update device button appearance
pub fn update_device_button_appearance(
    assignment: Res<InputDeviceAssignment>,
    mut button_query: Query<(
        &DeviceButton,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut Node,
        &Children,
    )>,
    mut text_query: Query<&mut TextColor>,
    available_devices: Res<AvailableInputDevices>,
) {
    if !assignment.is_changed() {
        return;
    }

    for (device_button, mut bg_color, mut border_color, mut node, children) in &mut button_query {
        let player_id = device_button.player_id;
        let device = &device_button.device;
        let current_device = assignment.get_device_for_player(player_id as u32);

        let is_selected = current_device == Some(device);
        let is_available = device.is_available(&available_devices);
        let is_used_by_other = is_device_used_by_other_player(player_id, device, &assignment);

        let (button_color, text_color) =
            get_button_colors(is_selected, is_available, is_used_by_other);

        *bg_color = BackgroundColor(button_color);
        *border_color = BorderColor(if is_selected {
            Color::srgb(0.4, 1.0, 0.4)
        } else {
            Color::srgb(0.6, 0.6, 0.6)
        });
        node.border = UiRect::all(Val::Px(if is_selected { 2.0 } else { 1.0 }));

        for child in children.iter() {
            if let Ok(mut text_color_comp) = text_query.get_mut(child) {
                text_color_comp.0 = text_color;
            }
        }
    }
}

/// System to update current device display
pub fn update_current_device_display(
    assignment: Res<InputDeviceAssignment>,
    panels_query: Query<&PlayerConfigPanel>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
    name_query: Query<(Entity, &Name)>,
) {
    if !assignment.is_changed() {
        return;
    }

    for panel in &panels_query {
        let player_id = panel.player_id;
        let current_device = assignment.get_device_for_player(player_id as u32);

        let device_text = current_device.map_or("None selected".to_string(), |d| d.name());
        let text_color = if current_device.is_some() {
            Color::srgb(0.2, 1.0, 0.2)
        } else {
            Color::srgb(1.0, 0.4, 0.4)
        };

        let target_name = format!("Current Device Name P{}", player_id);

        for (text_entity, name) in name_query.iter() {
            if name.as_str() == target_name {
                if let Ok((mut text, mut color)) = text_query.get_mut(text_entity) {
                    text.0 = device_text.clone();
                    color.0 = text_color;
                }
                break;
            }
        }
    }
}

// Helper functions
fn get_button_colors(
    is_selected: bool,
    is_available: bool,
    is_used_by_other: bool,
) -> (Color, Color) {
    let button_color = if is_selected {
        Color::srgba(0.2, 0.8, 0.2, 0.9) // Green for selected
    } else if !is_available || is_used_by_other {
        Color::srgba(0.8, 0.2, 0.2, 0.7) // Red for unavailable
    } else {
        Color::srgba(0.4, 0.4, 0.6, 0.8) // Blue for available
    };

    let text_color = if is_selected || (!is_available && !is_used_by_other) {
        Color::WHITE
    } else {
        Color::srgb(0.9, 0.9, 0.9)
    };

    (button_color, text_color)
}

fn lighten_color(color: Color, amount: f32) -> Color {
    let rgba = color.to_srgba();
    Color::srgba(
        (rgba.red + amount).min(1.0),
        (rgba.green + amount).min(1.0),
        (rgba.blue + amount).min(1.0),
        rgba.alpha,
    )
}

fn darken_color(color: Color, amount: f32) -> Color {
    let rgba = color.to_srgba();
    Color::srgba(
        (rgba.red - amount).max(0.0),
        (rgba.green - amount).max(0.0),
        (rgba.blue - amount).max(0.0),
        rgba.alpha,
    )
}

fn format_device_name(device: &InputDevice) -> String {
    match device {
        InputDevice::Keyboard(scheme) => scheme.name().to_string(),
        InputDevice::Gamepad(id) => format!("Gamepad {}", id + 1),
        InputDevice::Mouse => "Mouse".to_string(),
        InputDevice::Touch => "Touch".to_string(),
    }
}

fn is_device_used_by_other_player(
    current_player_id: usize,
    device: &InputDevice,
    assignment: &InputDeviceAssignment,
) -> bool {
    assignment
        .assignments
        .iter()
        .any(|(player_id, assigned_device)| {
            *player_id != current_player_id as u32 && assigned_device == device
        })
}
