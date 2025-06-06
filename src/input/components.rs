use bevy::prelude::*;

/// Main input controller component for entities that need input
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct InputController {
    pub movement_input: Vec2,
    pub action_input: ActionInput,
    pub player_id: u32,
    pub input_source: InputSource,
}

impl Default for InputController {
    fn default() -> Self {
        Self {
            movement_input: Vec2::ZERO,
            action_input: ActionInput::default(),
            player_id: 0,
            input_source: InputSource::Keyboard,
        }
    }
}

/// Action inputs (buttons)
#[derive(Reflect, Clone, Default)]
pub struct ActionInput {
    pub pause: bool,
    pub interact: bool,
}

/// Input source tracking
#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum InputSource {
    Keyboard,
    Gamepad(Entity),
}

/// Component to map specific inputs to a player
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct PlayerInputMapping {
    pub player_id: u32,
    pub keyboard_enabled: bool,
    pub gamepad_entity: Option<Entity>,
}

impl Default for PlayerInputMapping {
    fn default() -> Self {
        Self {
            player_id: 0,
            keyboard_enabled: true,
            gamepad_entity: None,
        }
    }
}

/// Resource for our custom gamepad settings (renamed to avoid conflict)
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct CustomGamepadSettings {
    pub deadzone: f32,
    pub move_threshold: f32,
    pub connected_gamepads: Vec<Entity>,
}

impl Default for CustomGamepadSettings {
    fn default() -> Self {
        Self {
            deadzone: super::GAMEPAD_DEADZONE,
            move_threshold: super::GAMEPAD_MOVE_THRESHOLD,
            connected_gamepads: Vec::new(),
        }
    }
}

/// Keyboard key mappings
pub struct KeyboardMapping {
    pub move_up: Vec<KeyCode>,
    pub move_down: Vec<KeyCode>,
    pub move_left: Vec<KeyCode>,
    pub move_right: Vec<KeyCode>,
    pub pause: Vec<KeyCode>,
    pub interact: Vec<KeyCode>,
}

impl Default for KeyboardMapping {
    fn default() -> Self {
        Self {
            move_up: vec![KeyCode::ArrowUp, KeyCode::KeyW],
            move_down: vec![KeyCode::ArrowDown, KeyCode::KeyS],
            move_left: vec![KeyCode::ArrowLeft, KeyCode::KeyA],
            move_right: vec![KeyCode::ArrowRight, KeyCode::KeyD],
            pause: vec![KeyCode::Escape, KeyCode::KeyP],
            interact: vec![KeyCode::KeyE, KeyCode::KeyF],
        }
    }
}

/// Gamepad button mappings
pub struct GamepadMapping {
    pub pause: GamepadButton,
    pub interact: GamepadButton,
}

impl Default for GamepadMapping {
    fn default() -> Self {
        Self {
            pause: GamepadButton::Start,    // Start/Options button
            interact: GamepadButton::South, // A/X button
        }
    }
}
