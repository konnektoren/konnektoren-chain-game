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
            input_source: InputSource::Keyboard(crate::settings::KeyboardScheme::Arrows),
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
    Keyboard(crate::settings::KeyboardScheme),
    Gamepad(Entity),
    Mouse,
    Touch,
    VirtualJoystick,
}

/// Component to map specific inputs to a player
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct PlayerInputMapping {
    pub player_id: u32,
    pub keyboard_scheme: Option<crate::settings::KeyboardScheme>,
    pub gamepad_entity: Option<Entity>,
    pub mouse_enabled: bool,
    pub touch_enabled: bool,
}

impl Default for PlayerInputMapping {
    fn default() -> Self {
        Self {
            player_id: 0,
            keyboard_scheme: Some(crate::settings::KeyboardScheme::WASD),
            gamepad_entity: None,
            mouse_enabled: true,
            touch_enabled: true,
        }
    }
}

impl PlayerInputMapping {
    pub fn from_settings(settings: &crate::settings::PlayerSettings) -> Self {
        // Extract input information from the new InputDevice structure
        let (keyboard_scheme, mouse_enabled, touch_enabled) = match &settings.input.primary_input {
            crate::settings::InputDevice::Keyboard(scheme) => (Some(scheme.clone()), false, false),
            crate::settings::InputDevice::Mouse => (None, true, false),
            crate::settings::InputDevice::Touch => (None, false, true),
            _ => (None, false, false),
        };

        // Check secondary input for additional capabilities
        let (secondary_mouse, secondary_touch, secondary_keyboard) =
            if let Some(ref secondary) = settings.input.secondary_input {
                match secondary {
                    crate::settings::InputDevice::Mouse => (true, false, None),
                    crate::settings::InputDevice::Touch => (false, true, None),
                    crate::settings::InputDevice::Keyboard(scheme) => {
                        (false, false, Some(scheme.clone()))
                    }
                    _ => (false, false, None),
                }
            } else {
                (false, false, None)
            };

        Self {
            player_id: settings.player_id,
            keyboard_scheme: keyboard_scheme.or(secondary_keyboard),
            gamepad_entity: None, // Will be assigned by system
            mouse_enabled: mouse_enabled || secondary_mouse,
            touch_enabled: touch_enabled || secondary_touch,
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

/// Virtual joystick state for touch/mouse input
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct VirtualJoystickState {
    pub is_active: bool,
    pub center_position: Vec2,
    pub current_position: Vec2,
    pub movement_vector: Vec2,
    pub touch_id: Option<u64>,
    pub max_distance: f32,
}

impl VirtualJoystickState {
    pub fn start_input(&mut self, position: Vec2, touch_id: Option<u64>) {
        self.is_active = true;
        self.center_position = position;
        self.current_position = position;
        self.touch_id = touch_id;
        self.update_movement();
    }

    pub fn update_input(&mut self, position: Vec2) {
        if self.is_active {
            self.current_position = position;
            self.update_movement();
        }
    }

    pub fn end_input(&mut self) {
        self.is_active = false;
        self.movement_vector = Vec2::ZERO;
        self.touch_id = None;
    }

    fn update_movement(&mut self) {
        let offset = self.current_position - self.center_position;
        let distance = offset.length();

        if distance > super::VIRTUAL_JOYSTICK_DEADZONE {
            if distance > self.max_distance {
                // Clamp to max distance and normalize
                self.movement_vector = offset.normalize();
                // Update current position to stay within bounds
                self.current_position =
                    self.center_position + offset.normalize() * self.max_distance;
            } else {
                // Scale by max distance to get normalized movement
                self.movement_vector = offset / self.max_distance;
            }
        } else {
            self.movement_vector = Vec2::ZERO;
        }
    }
}

/// Component for virtual joystick UI elements
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct VirtualJoystick;

/// Component for virtual joystick base (outer circle)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct VirtualJoystickBase;

/// Component for virtual joystick knob (inner circle)
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct VirtualJoystickKnob;

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
