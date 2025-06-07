use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
pub use systems::setup_grid_map;
use systems::{handle_map_config_changes, update_grid_visualization};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MapConfig>();
    app.register_type::<GridMap>();
    app.register_type::<GridCell>();
    app.register_type::<GridPosition>();

    // Initialize map configuration resource
    app.insert_resource(MapConfig::new(30, 25).with_cell_size(28.0).with_colors(
        Color::srgb(0.05, 0.05, 0.1),
        Color::srgba(0.2, 0.4, 0.6, 0.6),
    ));

    app.add_systems(OnEnter(crate::screens::Screen::Gameplay), setup_grid_map);

    app.add_systems(
        Update,
        (update_grid_visualization, handle_map_config_changes)
            .run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

// Default configuration constants
pub const DEFAULT_GRID_WIDTH: usize = 25;
pub const DEFAULT_GRID_HEIGHT: usize = 20;
pub const DEFAULT_CELL_SIZE: f32 = 32.0;
pub const GRID_COLOR: Color = Color::srgba(0.3, 0.3, 0.4, 0.8);
pub const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.15);
