use crate::settings::GameSettings;
use crate::{menus::Menu, screens::Screen};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use konnektoren_bevy::input::InputEvent;
use konnektoren_bevy::screens::settings::*;
use konnektoren_bevy::screens::settings::{SettingsScreenConfig, SettingsScreenEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_screen)
        .add_systems(OnExit(Menu::Settings), cleanup_settings_screen)
        .add_systems(
            Update,
            (
                handle_settings_events,
                handle_input_configuration_events,
                go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
            ),
        );
}

fn spawn_settings_screen(mut commands: Commands, game_settings: Res<GameSettings>) {
    info!("Spawning settings screen");

    let config = SettingsScreenConfig::new("Settings")
        .mobile_layout(false)
        .with_back_button_text("Back")
        .add_section(SettingsSection::audio_section())
        .add_section(create_multiplayer_section(&game_settings))
        .add_section(SettingsSection::input_section());

    commands.spawn((
        Name::new("Game Settings Screen"),
        config,
        StateScoped(Menu::Settings),
    ));
}

fn cleanup_settings_screen(
    mut commands: Commands,
    settings_query: Query<Entity, With<SettingsScreenConfig>>,
    active_query: Query<Entity, With<ActiveSettingsScreen>>,
) {
    for entity in settings_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in active_query.iter() {
        commands.entity(entity).despawn();
    }
    info!("Cleaned up settings screen");
}

fn create_multiplayer_section(game_settings: &GameSettings) -> SettingsSection {
    SettingsSection::new("Multiplayer")
        .add_setting(ScreenSettingsItem::toggle(
            "multiplayer_enabled",
            "Enable Multiplayer",
            game_settings.multiplayer.enabled,
        ))
        .add_setting(ScreenSettingsItem::int_slider(
            "player_count",
            "Number of Players",
            game_settings.multiplayer.player_count as i32,
            1,
            crate::settings::MAX_PLAYERS as i32,
            1,
        ))
        .add_setting(ScreenSettingsItem::toggle(
            "auto_assign_inputs",
            "Auto Assign Inputs",
            game_settings.multiplayer.auto_assign_inputs,
        ))
        .add_setting(ScreenSettingsItem::toggle(
            "auto_detect_players",
            "Auto Detect Players",
            game_settings.multiplayer.auto_detect_players,
        ))
}

fn handle_settings_events(
    mut events: EventReader<SettingsScreenEvent>,
    mut game_settings: ResMut<GameSettings>,
    mut global_volume: ResMut<GlobalVolume>,
    mut next_menu: ResMut<NextState<Menu>>,
    screen: Res<State<Screen>>,
) {
    for event in events.read() {
        match event {
            SettingsScreenEvent::ValueChanged {
                setting_id, value, ..
            } => {
                match setting_id.as_str() {
                    "master_volume" => {
                        if let Some(volume) = value.as_float() {
                            global_volume.volume = bevy::audio::Volume::Linear(volume);
                            info!(
                                "Updated master volume to: {:.1}% ({:.2})",
                                volume * 100.0,
                                volume
                            );
                        }
                    }
                    "multiplayer_enabled" => {
                        if let Some(enabled) = value.as_bool() {
                            game_settings.multiplayer.enable_multiplayer(enabled);

                            // Auto-enable multiplayer if more than 1 player (same logic as original)
                            if game_settings.multiplayer.player_count > 1 {
                                game_settings.multiplayer.enabled = true;
                            }

                            info!("Multiplayer toggled: {}", game_settings.multiplayer.enabled);
                        }
                    }
                    "player_count" => {
                        if let Some(count) = value.as_int() {
                            let new_count = (count as usize).clamp(1, crate::settings::MAX_PLAYERS);
                            game_settings.multiplayer.set_player_count(new_count);
                            info!("Updated player count to: {}", new_count);
                        }
                    }
                    "auto_assign_inputs" => {
                        if let Some(enabled) = value.as_bool() {
                            game_settings.multiplayer.auto_assign_inputs = enabled;
                            info!("Auto assign inputs: {}", enabled);
                        }
                    }
                    "auto_detect_players" => {
                        if let Some(enabled) = value.as_bool() {
                            game_settings.multiplayer.auto_detect_players = enabled;
                            info!("Auto detect players: {}", enabled);
                        }
                    }
                    "configure_players" => {
                        // Handle the "Configure Players" button click
                        info!("Opening device selection");
                        next_menu.set(Menu::DeviceSelection);
                        return; // Don't handle dismissed event after this
                    }
                    _ => warn!("Unhandled setting: {}", setting_id),
                }
            }
            SettingsScreenEvent::Dismissed { .. } => {
                info!("Settings screen dismissed via back button");
                let target_menu = if screen.get() == &Screen::Title {
                    Menu::Main
                } else {
                    Menu::Pause
                };
                next_menu.set(target_menu);
            }
            _ => {}
        }
    }
}

fn handle_input_configuration_events(
    mut input_config_events: EventReader<InputConfigurationEvent>,
    mut input_events: EventReader<InputEvent>,
    mut commands: Commands,
    mut game_settings: ResMut<GameSettings>,
    input_config_query: Query<Entity, With<ActiveInputConfiguration>>,
) {
    for event in input_config_events.read() {
        match event {
            InputConfigurationEvent::Close => {
                info!("Input configuration closed, respawning settings screen");

                // Clean up input configuration
                for entity in input_config_query.iter() {
                    commands.entity(entity).despawn();
                }

                // Respawn the settings screen
                let config = SettingsScreenConfig::new("Settings")
                    .mobile_layout(false)
                    .with_back_button_text("Back")
                    .add_section(SettingsSection::audio_section())
                    .add_section(create_multiplayer_section(&game_settings))
                    .add_section(SettingsSection::input_section());

                commands.spawn((
                    Name::new("Game Settings Screen"),
                    config,
                    StateScoped(Menu::Settings),
                ));
            }
            InputConfigurationEvent::DeviceAssigned { player_id, device } => {
                info!(
                    "Device {} assigned to player {}",
                    device.name(),
                    player_id + 1
                );

                // Update game settings
                if let Some(player_settings) = game_settings
                    .multiplayer
                    .players
                    .get_mut(*player_id as usize)
                {
                    player_settings.input.primary_input = device.clone();
                    player_settings.enabled = true;
                }
                game_settings.multiplayer.auto_assign_inputs = false;
            }
            InputConfigurationEvent::DeviceUnassigned { player_id } => {
                info!("Device unassigned from player {}", player_id + 1);

                if let Some(player_settings) = game_settings
                    .multiplayer
                    .players
                    .get_mut(*player_id as usize)
                {
                    player_settings.enabled = false;
                }
            }
            _ => {}
        }
    }

    // Handle input system events for logging
    for event in input_events.read() {
        match event {
            InputEvent::DeviceAssigned { player_id, device } => {
                info!(
                    "Input system: Device {} assigned to player {}",
                    device.name(),
                    player_id + 1
                );
            }
            InputEvent::DeviceUnassigned { player_id } => {
                info!(
                    "Input system: Device unassigned from player {}",
                    player_id + 1
                );
            }
            _ => {}
        }
    }
}

fn go_back(
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut config_events: EventWriter<InputConfigurationEvent>,
    input_config_query: Query<&ActiveInputConfiguration>,
) {
    // If input configuration is open, close it instead of going back
    if !input_config_query.is_empty() {
        info!("Escape pressed in input configuration, closing it");
        config_events.write(InputConfigurationEvent::Close);
    } else {
        info!("Going back via escape key");
        next_menu.set(if screen.get() == &Screen::Title {
            Menu::Main
        } else {
            Menu::Pause
        });
    }
}
