use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PlayerChain>();
    app.register_type::<ChainSegment>();
    app.register_type::<MovementTrail>();
    app.register_type::<FlyingToChain>();

    app.add_event::<ChainExtendEvent>();

    // Run setup system after player spawns
    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_player_chain.after(crate::player::spawn_player),
    );

    app.add_systems(
        Update,
        (
            track_player_movement.in_set(crate::AppSystems::Update),
            handle_chain_extend_events.in_set(crate::AppSystems::Update),
            create_flying_to_chain_objects.in_set(crate::AppSystems::Update),
            update_flying_objects.in_set(crate::AppSystems::Update),
            update_chain_positions.in_set(crate::AppSystems::Update),
            animate_chain_segments.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const CHAIN_SEGMENT_SIZE: f32 = 12.0;
pub const CHAIN_SEGMENT_SPACING: f32 = 25.0;
pub const MOVEMENT_SAMPLE_RATE: f32 = 0.1; // Record position every 0.1 seconds
pub const FLY_TO_CHAIN_DURATION: f32 = 0.8; // Duration of fly animation
