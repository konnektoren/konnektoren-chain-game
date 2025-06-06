use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GridMap>();
    app.register_type::<GridCell>();
    app.register_type::<GridPosition>();

    app.add_systems(
        Update,
        (
            setup_grid_map.run_if(not(resource_exists::<GridMap>)),
            update_grid_visualization,
        )
            .chain(),
    );
}

// Configuration constants
pub const GRID_SIZE: usize = 20;
pub const CELL_SIZE: f32 = 32.0;
pub const GRID_COLOR: Color = Color::srgba(0.3, 0.3, 0.4, 0.8);
pub const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.15);
