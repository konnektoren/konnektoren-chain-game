use bevy::prelude::*;

mod components;
pub mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GameplayScore>();
    app.register_type::<PlayerScore>();
    app.register_type::<GameTimer>();
    app.register_type::<ScoreDisplay>();
    app.register_type::<TimerDisplay>();
    app.register_type::<PlayerScoreDisplay>();
    app.register_type::<PlayerStatsDisplay>();
    app.register_type::<TeamStatsDisplay>();
    app.register_type::<OptionsLegendDisplay>();
    app.register_type::<OptionsLegendContainer>();
    app.register_type::<OptionLegendItem>();

    // Register events
    app.add_event::<ScoreUpdateEvent>();
    app.add_event::<GameTimerEvent>();

    // Initialize resources
    app.init_resource::<GameplayScore>();
    app.init_resource::<GameTimer>();

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        (setup_gameplay_ui, reset_game_state),
    );

    app.add_systems(
        Update,
        (
            update_game_timer.in_set(crate::AppSystems::TickTimers),
            handle_option_collection_events.in_set(crate::AppSystems::Update),
            handle_score_events.in_set(crate::AppSystems::Update),
            handle_chain_destruction_events.in_set(crate::AppSystems::Update),
            update_individual_player_scores.in_set(crate::AppSystems::Update),
            update_team_stats_display.in_set(crate::AppSystems::Update),
            update_timer_display.in_set(crate::AppSystems::Update),
            update_options_legend_display.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants
pub const CORRECT_ANSWER_POINTS: u32 = 10;
pub const STREAK_BONUS_MULTIPLIER: u32 = 5;
pub const WRONG_ANSWER_PENALTY: i32 = -5;
pub const GAME_DURATION_MINUTES: f32 = 5.0;
