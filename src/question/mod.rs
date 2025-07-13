use crate::game_state::GameState;
use bevy::prelude::*;

mod components;
pub mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    // Only register types that implement Reflect
    app.register_type::<QuestionTimer>();
    app.register_type::<QuestionDisplay>();
    app.register_type::<QuestionHelpDisplay>();

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_question_system.run_if(|game_state: Res<GameState>| game_state.is_ready()),
    );

    app.add_systems(
        Update,
        (
            update_question_timer.in_set(crate::AppSystems::TickTimers),
            update_question_display.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .run_if(resource_exists::<QuestionSystem>)
            .in_set(crate::PausableSystems),
    );
}

pub const QUESTION_DURATION: f32 = 10.0; // seconds
pub const QUESTION_FADE_DURATION: f32 = 0.5; // seconds for fade in/out
