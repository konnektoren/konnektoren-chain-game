//! A loading screen during which game assets are loaded if necessary.
//! This reduces stuttering, especially for audio on Wasm.

use crate::game_state::GameState;
use bevy::prelude::*;

use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), spawn_loading_screen);

    app.add_systems(
        Update,
        (
            update_loading_text,
            enter_gameplay_screen.run_if(in_state(Screen::Loading).and(all_assets_loaded)),
            handle_loading_timeout,
        ),
    );
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Loading Screen"),
        StateScoped(Screen::Loading),
        LoadingTimeout(Timer::from_seconds(10.0, TimerMode::Once)), // 10 second timeout
        children![
            widget::label("Loading Level A1..."),
            (
                Name::new("Loading Details"),
                Text("Preparing challenges...".to_string()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                LoadingDetails,
            )
        ],
    ));
}

#[derive(Component)]
struct LoadingDetails;

#[derive(Component)]
struct LoadingTimeout(Timer);

fn update_loading_text(
    game_state: Res<GameState>,
    mut loading_query: Query<&mut Text, With<LoadingDetails>>,
) {
    if !game_state.is_changed() {
        return;
    }

    for mut text in &mut loading_query {
        if game_state.level_loaded && game_state.challenge_loaded {
            text.0 = "Ready to play!".to_string();
        } else if game_state.level_loaded {
            text.0 = format!(
                "Loading challenge: {}...",
                game_state
                    .current_challenge_id
                    .as_deref()
                    .unwrap_or("unknown")
            );
        } else {
            text.0 = format!("Loading level: {}...", game_state.current_level_id);
        }
    }
}

fn handle_loading_timeout(
    time: Res<Time>,
    mut timeout_query: Query<&mut LoadingTimeout>,
    mut loading_query: Query<&mut Text, With<LoadingDetails>>,
    game_state: Res<GameState>,
) {
    for mut timeout in &mut timeout_query {
        timeout.0.tick(time.delta());

        if timeout.0.just_finished() && !game_state.is_ready() {
            for mut text in &mut loading_query {
                text.0 = "Failed to load assets. Please check that asset files exist.".to_string();
            }
            error!("Loading timeout - assets failed to load within 10 seconds");
        }
    }
}

fn enter_gameplay_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Gameplay);
}

fn all_assets_loaded(game_state: Res<GameState>) -> bool {
    game_state.is_ready()
}
