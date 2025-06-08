use bevy::{audio::Volume, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};

use crate::{menus::Menu, screens::Screen, settings::*, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    // Register UI components
    app.register_type::<GlobalVolumeLabel>();
    app.register_type::<MultiplayerToggle>();
    app.register_type::<PlayerCountDisplay>();
    app.register_type::<AutoAssignToggle>();
    app.register_type::<AutoDetectToggle>();
    app.register_type::<DeviceStatusDisplay>();

    app.add_systems(
        Update,
        (
            update_global_volume_label,
            update_multiplayer_display,
            update_player_count_display,
            update_auto_assign_display,
            update_auto_detect_display,
            update_device_status_display,
        )
            .run_if(in_state(Menu::Settings)),
    );
}

fn spawn_settings_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Settings Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Settings),
        children![
            widget::header("Settings"),
            settings_grid(),
            widget::button("Back", go_back_on_click),
        ],
    ));
}

fn settings_grid() -> impl Bundle {
    (
        Name::new("Settings Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(15.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        children![
            // Audio Settings
            audio_section(),
            // Multiplayer Settings
            multiplayer_section(),
        ],
    )
}

fn audio_section() -> impl Bundle {
    (
        Name::new("Audio Section"),
        Node {
            grid_column: GridPlacement::span(2),
            flex_direction: FlexDirection::Column,
            row_gap: Px(10.0),
            ..default()
        },
        children![(
            Name::new("Audio Header"),
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(2, 400.0),
                column_gap: Px(30.0),
                ..default()
            },
            children![
                (
                    widget::label("Master Volume"),
                    Node {
                        justify_self: JustifySelf::End,
                        ..default()
                    }
                ),
                global_volume_widget(),
            ],
        ),],
    )
}

fn player_management_widget() -> impl Bundle {
    (
        Name::new("Player Management Widget"),
        Node {
            justify_self: JustifySelf::Start,
            align_items: AlignItems::Center,
            column_gap: Px(10.0),
            ..default()
        },
        children![
            widget::button_small("-", remove_player),
            (
                Name::new("Player Count Display"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    min_width: Px(40.0),
                    ..default()
                },
                children![(widget::label(""), PlayerCountDisplay)],
            ),
            widget::button_small("+", add_player),
        ],
    )
}

fn multiplayer_section() -> impl Bundle {
    (
        Name::new("Multiplayer Section"),
        Node {
            grid_column: GridPlacement::span(2),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(15.0),
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        },
        children![
            widget::header_small("Multiplayer"),
            (
                Name::new("Multiplayer Controls"),
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(2, 400.0),
                    column_gap: Val::Px(30.0),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                children![
                    (
                        widget::label("Enable Multiplayer"),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        }
                    ),
                    multiplayer_widget(),
                    (
                        widget::label("Add/Remove Players"),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        }
                    ),
                    player_management_widget(),
                    (
                        widget::label("Configure Players"),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        }
                    ),
                    configure_players_widget(),
                ],
            ),
        ],
    )
}

fn add_player(_: Trigger<Pointer<Click>>, mut game_settings: ResMut<GameSettings>) {
    let new_count = (game_settings.multiplayer.player_count + 1).min(MAX_PLAYERS);
    game_settings.multiplayer.set_player_count(new_count);
    info!("Added player, new count: {}", new_count);
}

fn remove_player(_: Trigger<Pointer<Click>>, mut game_settings: ResMut<GameSettings>) {
    let new_count = (game_settings.multiplayer.player_count.saturating_sub(1)).max(1);
    game_settings.multiplayer.set_player_count(new_count);
    info!("Removed player, new count: {}", new_count);
}

fn toggle_multiplayer(_: Trigger<Pointer<Click>>, mut game_settings: ResMut<GameSettings>) {
    let new_state = !game_settings.multiplayer.enabled;
    game_settings.multiplayer.enable_multiplayer(new_state);

    // Auto-enable multiplayer if more than 1 player
    if game_settings.multiplayer.player_count > 1 {
        game_settings.multiplayer.enabled = true;
    }

    info!("Multiplayer toggled: {}", game_settings.multiplayer.enabled);
}

fn global_volume_widget() -> impl Bundle {
    (
        Name::new("Global Volume Widget"),
        Node {
            justify_self: JustifySelf::Start,
            align_items: AlignItems::Center,
            column_gap: Px(5.0),
            ..default()
        },
        children![
            widget::button_small("-", lower_global_volume),
            (
                Name::new("Current Volume"),
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    min_width: Px(60.0),
                    ..default()
                },
                children![(widget::label(""), GlobalVolumeLabel)],
            ),
            widget::button_small("+", raise_global_volume),
        ],
    )
}

fn multiplayer_widget() -> impl Bundle {
    (
        Name::new("Multiplayer Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![(
            widget::button("Toggle", toggle_multiplayer),
            MultiplayerToggle,
        ),],
    )
}

// Audio controls
const MIN_VOLUME: f32 = 0.0;
const MAX_VOLUME: f32 = 3.0;

fn lower_global_volume(_: Trigger<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() - 0.1).max(MIN_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

fn raise_global_volume(_: Trigger<Pointer<Click>>, mut global_volume: ResMut<GlobalVolume>) {
    let linear = (global_volume.volume.to_linear() + 0.1).min(MAX_VOLUME);
    global_volume.volume = Volume::Linear(linear);
}

// UI Component markers
#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MultiplayerToggle;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct PlayerCountDisplay;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct AutoAssignToggle;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct AutoDetectToggle;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct DeviceStatusDisplay;

fn update_global_volume_label(
    global_volume: Res<GlobalVolume>,
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
) {
    let percent = 100.0 * global_volume.volume.to_linear();
    label.0 = format!("{percent:3.0}%");
}

fn update_multiplayer_display(
    game_settings: Res<GameSettings>,
    button_query: Query<&Children, With<MultiplayerToggle>>,
    mut text_query: Query<&mut Text>,
) {
    if !game_settings.is_changed() {
        return;
    }

    for children in &button_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if game_settings.multiplayer.enabled {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };
            }
        }
    }
}

fn update_player_count_display(
    game_settings: Res<GameSettings>,
    mut label: Single<&mut Text, With<PlayerCountDisplay>>,
) {
    if game_settings.is_changed() {
        label.0 = format!("{}", game_settings.multiplayer.player_count);
    }
}

fn update_auto_assign_display(
    game_settings: Res<GameSettings>,
    button_query: Query<&Children, With<AutoAssignToggle>>,
    mut text_query: Query<&mut Text>,
) {
    if !game_settings.is_changed() {
        return;
    }

    for children in &button_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if game_settings.multiplayer.auto_assign_inputs {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };
            }
        }
    }
}

fn update_auto_detect_display(
    game_settings: Res<GameSettings>,
    button_query: Query<&Children, With<AutoDetectToggle>>,
    mut text_query: Query<&mut Text>,
) {
    if !game_settings.is_changed() {
        return;
    }

    for children in &button_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if game_settings.multiplayer.auto_detect_players {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };
            }
        }
    }
}

fn update_device_status_display(
    available_devices: Res<AvailableInputDevices>,
    assignment: Res<InputDeviceAssignment>,
    mut label: Single<&mut Text, With<DeviceStatusDisplay>>,
) {
    if available_devices.is_changed() || assignment.is_changed() {
        let gamepad_count = available_devices.gamepads.len();
        let keyboard = if available_devices.has_keyboard {
            "✓"
        } else {
            "✗"
        };
        let mouse = if available_devices.has_mouse {
            "✓"
        } else {
            "✗"
        };

        let mut status_text = format!(
            "Gamepads: {} | Keyboard: {} | Mouse: {}",
            gamepad_count, keyboard, mouse
        );

        if !assignment.conflicts.is_empty() {
            status_text.push_str("\n⚠️ Input conflicts detected");
        }

        label.0 = status_text;
    }
}

fn go_back_on_click(
    _: Trigger<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

fn configure_players_widget() -> impl Bundle {
    (
        Name::new("Configure Players Widget"),
        Node {
            justify_self: JustifySelf::Start,
            ..default()
        },
        children![(widget::button("Configure", open_device_selection),),],
    )
}

fn open_device_selection(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::DeviceSelection);
}
