use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
pub use systems::spawn_player;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerController>();
    app.register_type::<PlayerVisual>();
    app.register_type::<PlayerStats>();
    app.register_type::<PlayerEffects>();
    app.register_type::<PlayerGlow>();
    app.register_type::<PlayerAura>();
    app.register_type::<PlayerEnergyParticles>();
    app.register_type::<PlayerTrail>();
    app.register_type::<PlayerIndex>();

    // Register the events
    app.add_event::<OptionCollectedEvent>();
    app.add_event::<PlayerVisualEvent>();

    // Ensure player spawns AFTER map setup
    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        spawn_player.after(crate::map::setup_grid_map),
    );

    app.add_systems(
        Update,
        (
            handle_player_input.in_set(crate::AppSystems::RecordInput),
            move_player.in_set(crate::AppSystems::Update),
            collect_options.in_set(crate::AppSystems::Update),
            animate_player.in_set(crate::AppSystems::Update),
            update_player_energy_particles.in_set(crate::AppSystems::Update),
            update_player_trail.in_set(crate::AppSystems::Update),
            handle_player_visual_events.in_set(crate::AppSystems::Update),
            handle_collection_events.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const PLAYER_MOVE_SPEED: f32 = 200.0; // pixels per second
pub const PLAYER_SIZE: f32 = 20.0;
