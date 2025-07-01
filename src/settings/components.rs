use bevy::prelude::*;
use konnektoren_bevy::input::device::{InputDevice, KeyboardScheme};

/// Main game settings resource
#[derive(Resource, Reflect, Clone, Debug, Default)]
#[reflect(Resource)]
pub struct GameSettings {
    pub multiplayer: MultiplayerSettings,
    pub audio: AudioSettings,
    pub display: DisplaySettings,
}

/// Multiplayer configuration
#[derive(Reflect, Clone, Debug)]
pub struct MultiplayerSettings {
    pub enabled: bool,
    pub player_count: usize,
    pub auto_detect_players: bool,
    pub auto_assign_inputs: bool,
    pub players: Vec<PlayerSettings>,
}

impl Default for MultiplayerSettings {
    fn default() -> Self {
        let mut settings = Self {
            enabled: false,
            player_count: 1,
            auto_detect_players: false,
            auto_assign_inputs: false,
            players: vec![PlayerSettings::default()],
        };
        settings.setup_default_player_configs();
        settings
    }
}

impl MultiplayerSettings {
    pub fn set_player_count(&mut self, count: usize) {
        let count = count.clamp(1, super::MAX_PLAYERS);
        self.player_count = count;
        self.players.resize_with(count, PlayerSettings::default);
        self.setup_default_player_configs();
    }

    pub fn enable_multiplayer(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled && self.player_count == 1 {
            self.set_player_count(2);
        } else if !enabled {
            self.set_player_count(1);
        }
    }

    fn setup_default_player_configs(&mut self) {
        for (i, player) in self.players.iter_mut().enumerate() {
            player.player_id = i as u32;
            player.name = format!("Player {}", i + 1);
            player.input = InputSettings::default_for_player(i);
            player.color = Self::default_player_color(i);
            player.enabled = true;
        }
    }

    fn default_player_color(index: usize) -> Color {
        let colors = [
            Color::srgb(1.0, 0.8, 0.2), // Yellow
            Color::srgb(0.2, 0.8, 1.0), // Blue
            Color::srgb(1.0, 0.2, 0.4), // Red
            Color::srgb(0.2, 1.0, 0.4), // Green
        ];
        colors[index % colors.len()]
    }
}

/// Settings for individual players
#[derive(Reflect, Clone, Debug)]
pub struct PlayerSettings {
    pub player_id: u32,
    pub name: String,
    pub color: Color,
    pub input: InputSettings,
    pub enabled: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            player_id: 0,
            name: "Player 1".to_string(),
            color: Color::srgb(1.0, 0.8, 0.2),
            input: InputSettings::default(),
            enabled: true,
        }
    }
}

/// Input configuration for a player
#[derive(Reflect, Clone, Debug)]
pub struct InputSettings {
    pub primary_input: InputDevice,
    pub secondary_input: Option<InputDevice>,
    pub allow_multiple_devices: bool,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            primary_input: InputDevice::Keyboard(KeyboardScheme::WASD),
            secondary_input: None,
            allow_multiple_devices: true,
        }
    }
}

impl InputSettings {
    pub fn default_for_player(player_index: usize) -> Self {
        match player_index {
            0 => Self {
                primary_input: InputDevice::Keyboard(KeyboardScheme::WASD),
                secondary_input: Some(InputDevice::Mouse),
                allow_multiple_devices: true,
            },
            1 => Self {
                primary_input: InputDevice::Keyboard(KeyboardScheme::Arrows),
                secondary_input: None,
                allow_multiple_devices: false,
            },
            2 => Self {
                primary_input: InputDevice::Gamepad(0),
                secondary_input: None,
                allow_multiple_devices: false,
            },
            3 => Self {
                primary_input: InputDevice::Gamepad(1),
                secondary_input: None,
                allow_multiple_devices: false,
            },
            _ => Self::default(),
        }
    }
}

/// Audio settings
#[derive(Reflect, Clone, Debug)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.8,
            sfx_volume: 1.0,
        }
    }
}

/// Display settings
#[derive(Reflect, Clone, Debug)]
pub struct DisplaySettings {
    pub vsync: bool,
    pub show_fps: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            vsync: true,
            show_fps: false,
        }
    }
}

/// Resource for managing input device assignments
#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct InputDeviceAssignment {
    pub assignments: Vec<(u32, InputDevice)>,
    pub conflicts: Vec<String>,
}

impl InputDeviceAssignment {
    pub fn assign_device(&mut self, player_id: u32, device: InputDevice) {
        self.assignments.retain(|(id, _)| *id != player_id);
        self.assignments.push((player_id, device));
        self.validate_assignments();
    }

    pub fn get_device_for_player(&self, player_id: u32) -> Option<&InputDevice> {
        self.assignments
            .iter()
            .find(|(id, _)| *id == player_id)
            .map(|(_, device)| device)
    }

    fn validate_assignments(&mut self) {
        self.conflicts.clear();

        for i in 0..self.assignments.len() {
            for j in (i + 1)..self.assignments.len() {
                let (player1, device1) = &self.assignments[i];
                let (player2, device2) = &self.assignments[j];

                if devices_conflict(device1, device2) {
                    self.conflicts.push(format!(
                        "Player {} and Player {} both assigned to {}",
                        player1 + 1,
                        player2 + 1,
                        device1.name()
                    ));
                }
            }
        }
    }
}

fn devices_conflict(device1: &InputDevice, device2: &InputDevice) -> bool {
    match (device1, device2) {
        (InputDevice::Keyboard(scheme1), InputDevice::Keyboard(scheme2)) => scheme1 == scheme2,
        (InputDevice::Gamepad(id1), InputDevice::Gamepad(id2)) => id1 == id2,
        (InputDevice::Mouse, InputDevice::Mouse) => true,
        (InputDevice::Touch, InputDevice::Touch) => true,
        _ => false,
    }
}

/// Resource to track device selection state
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DeviceSelectionState {
    pub selecting_player: Option<usize>,
    pub selection_timeout: Timer,
    pub pending_assignments: Vec<(usize, InputDevice)>,
}

// UI Components for device selection
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlayerConfigPanel {
    pub player_id: usize,
    pub is_active: bool,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DeviceButton {
    pub device: InputDevice,
    pub player_id: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DeviceButtonsContainer {
    pub player_id: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DeviceSectionContainer {
    pub player_id: usize,
    pub enabled: bool,
}

#[derive(Component)]
pub struct PlayerGrid;

#[derive(Component)]
pub struct DeviceSelectionUI;
