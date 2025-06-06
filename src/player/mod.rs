use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerController>();
    app.register_type::<PlayerVisual>();
    app.register_type::<PlayerStats>();

    // Register the event
    app.add_event::<OptionCollectedEvent>();

    app.add_systems(OnEnter(crate::screens::Screen::Gameplay), spawn_player);

    app.add_systems(
        Update,
        (
            handle_player_input.in_set(crate::AppSystems::RecordInput),
            move_player.in_set(crate::AppSystems::Update),
            collect_options.in_set(crate::AppSystems::Update),
            animate_player.in_set(crate::AppSystems::Update),
            handle_collection_events.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const PLAYER_MOVE_SPEED: f32 = 200.0; // pixels per second
pub const PLAYER_SIZE: f32 = 20.0;
