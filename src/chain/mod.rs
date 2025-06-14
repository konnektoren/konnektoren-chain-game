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
    app.register_type::<ChainReaction>();
    app.register_type::<ChainReactionState>();
    app.register_type::<PlayerChainSegment>();
    app.register_type::<ChainMerging>();
    app.register_type::<ChainMergeState>();
    app.register_type::<SegmentReindexMarker>();

    app.add_event::<ChainExtendEvent>();
    app.add_event::<ChainReactionEvent>();
    app.add_event::<ChainSegmentDestroyedEvent>();
    app.add_event::<ChainMergeEvent>();

    app.init_resource::<ChainReactionState>();
    app.init_resource::<ChainMergeState>();

    // Run setup system after player spawns (which runs after map setup)
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
            detect_player_chain_collision.in_set(crate::AppSystems::Update),
            handle_chain_reaction_events.in_set(crate::AppSystems::Update),
            update_chain_reaction.in_set(crate::AppSystems::Update),
            animate_reacting_segments.in_set(crate::AppSystems::Update),
            detect_chain_merges.in_set(crate::AppSystems::Update),
            handle_chain_merge_events.in_set(crate::AppSystems::Update),
            animate_merging_segments.in_set(crate::AppSystems::Update),
            cleanup_merged_chains
                .in_set(crate::AppSystems::Update)
                .after(animate_merging_segments),
            handle_segment_reindexing
                .in_set(crate::AppSystems::Update)
                .after(cleanup_merged_chains),
            update_merge_cooldown.in_set(crate::AppSystems::Update),
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

// Chain reaction constants
pub const REACTION_SPREAD_INTERVAL: f32 = 0.1; // Time between each ball starting to react
pub const REACTION_BALL_DURATION: f32 = 0.5; // How long each ball takes to disappear
pub const POINTS_LOST_PER_SEGMENT: i32 = 5; // Points deducted per destroyed chain segment

pub const MERGE_ANIMATION_DURATION: f32 = 0.8; // Duration of merge animation
pub const MERGE_COOLDOWN_DURATION: f32 = 1.0; // Cooldown between merges
pub const MIN_SEGMENTS_TO_MERGE: usize = 3; // Number of same segments needed to merge
