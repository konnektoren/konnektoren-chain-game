use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<OptionCollectible>();
    app.register_type::<OptionSpawnTimer>();
    app.register_type::<OptionVisual>();
    app.register_type::<OptionLightEffect>();
    app.register_type::<OptionGlow>();
    app.register_type::<OptionPulseRing>();
    app.register_type::<OptionSparkles>();

    app.init_resource::<OptionSpawnTimer>();

    app.add_systems(
        Update,
        (
            update_option_spawn_settings,
            spawn_option_collectibles,
            cleanup_expired_options,
            clear_options_on_question_change,
            animate_option_collectibles,
            update_option_sparkles,
            enhance_correct_answer_effects,
            fade_expiring_options,
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}

// Configuration constants for options
pub const OPTIONS_PER_TYPE: usize = 3;
pub const OPTION_LIFETIME: f32 = 8.0; // Options last 8 seconds
pub const OPTION_SPAWN_INTERVAL: f32 = 1.0; // Spawn every second
pub const OPTION_FADE_DURATION: f32 = 2.0; // Start fading 2 seconds before expiration
