use super::components::*;
use crate::screens::Screen;
use crate::settings::GameSettings;
use bevy::prelude::*;

/// System to set up the gameplay UI
pub fn setup_gameplay_ui(mut commands: Commands) {
    // Score and timer overlay at the top right
    commands.spawn((
        Name::new("Gameplay UI"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(15.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            align_items: AlignItems::End,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderRadius::all(Val::Px(8.0)),
        StateScoped(Screen::Gameplay),
        children![
            // Timer display
            (
                Name::new("Timer Display"),
                Text("05:00".to_string()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimerDisplay,
            ),
            // Score display
            (
                Name::new("Score Display"),
                Text("Score: 0".to_string()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ScoreDisplay,
            ),
            // Additional stats
            (
                Name::new("Stats Display"),
                Text("Streak: 0 | Accuracy: 0%".to_string()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node::default(),
            ),
        ],
    ));
}

/// System to reset game state when entering gameplay
pub fn reset_game_state(
    mut gameplay_score: ResMut<GameplayScore>,
    mut game_timer: ResMut<GameTimer>,
    game_settings: Res<GameSettings>,
    time: Res<Time>,
) {
    // Reset gameplay score
    gameplay_score.players.clear();
    gameplay_score.game_active = true;
    gameplay_score.game_start_time = time.elapsed_secs();

    // Initialize scores for all active players
    for player_settings in &game_settings.multiplayer.players {
        if player_settings.enabled {
            gameplay_score.add_player(
                Entity::PLACEHOLDER, // Will be updated when actual entities are created
                player_settings.name.clone(),
            );
        }
    }

    // Reset game timer
    *game_timer = GameTimer::default();

    info!(
        "Game state reset - new game started with {} players!",
        game_settings.multiplayer.player_count
    );
}

/// System to update the game timer
pub fn update_game_timer(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    mut timer_events: EventWriter<GameTimerEvent>,
) {
    game_timer.timer.tick(time.delta());

    // Update remaining time
    game_timer.time_remaining =
        (game_timer.game_duration - game_timer.timer.elapsed_secs()).max(0.0);

    // Check for overtime
    if game_timer.timer.finished() && !game_timer.is_overtime {
        game_timer.is_overtime = true;
        timer_events.write(GameTimerEvent::GameEnded);
        info!("Game time ended! Entering overtime...");
    }
}

/// System to handle score update events
pub fn handle_score_events(
    mut score_events: EventReader<ScoreUpdateEvent>,
    mut gameplay_score: ResMut<GameplayScore>,
) {
    for event in score_events.read() {
        // Ensure player exists in the score tracking
        if !gameplay_score.players.contains_key(&event.player_entity) {
            gameplay_score.add_player(event.player_entity, "Player 1".to_string());
        }

        // Update player score
        if let Some(player_score) = gameplay_score.get_player_score_mut(event.player_entity) {
            if event.is_correct {
                player_score.add_correct_answer();
                info!(
                    "Player scored {} points! Total: {} (Streak: {})",
                    event.points_awarded, player_score.total_score, player_score.current_streak
                );
            } else {
                player_score.add_wrong_answer();
                info!(
                    "Player lost points! Total: {} (Streak reset)",
                    player_score.total_score
                );
            }
        }
    }
}

/// System to update score display for multiplayer
pub fn update_score_display(
    gameplay_score: Res<GameplayScore>,
    game_settings: Res<GameSettings>,
    mut score_query: Query<&mut Text, With<ScoreDisplay>>,
    mut stats_query: Query<&mut Text, (Without<ScoreDisplay>, Without<TimerDisplay>)>,
) {
    // Calculate total score and best player stats
    let mut total_score = 0;
    let mut best_streak = 0;
    let mut total_correct = 0;
    let mut total_attempts = 0;

    for player_score in gameplay_score.players.values() {
        total_score += player_score.total_score;
        best_streak = best_streak.max(player_score.best_streak);
        total_correct += player_score.correct_answers;
        total_attempts += player_score.collection_count;
    }

    let accuracy = if total_attempts > 0 {
        (total_correct as f32 / total_attempts as f32) * 100.0
    } else {
        0.0
    };

    for mut text in &mut score_query {
        if game_settings.multiplayer.enabled {
            text.0 = format!("Team Score: {}", total_score);
        } else {
            text.0 = format!("Score: {}", total_score);
        }
    }

    // Update additional stats
    for mut text in stats_query.iter_mut().take(1) {
        text.0 = format!("Best Streak: {} | Accuracy: {:.0}%", best_streak, accuracy);
    }
}

/// System to update timer display
pub fn update_timer_display(
    game_timer: Res<GameTimer>,
    mut timer_query: Query<(&mut Text, &mut TextColor), With<TimerDisplay>>,
) {
    for (mut text, mut color) in &mut timer_query {
        text.0 = game_timer.time_remaining_formatted();

        // Change color based on time remaining
        if game_timer.is_overtime {
            color.0 = Color::srgb(1.0, 0.3, 0.3); // Red for overtime
        } else if game_timer.time_remaining <= 30.0 {
            color.0 = Color::srgb(1.0, 0.7, 0.3); // Orange for warning
        } else if game_timer.time_remaining <= 10.0 {
            color.0 = Color::srgb(1.0, 0.3, 0.3); // Red for urgent
        } else {
            color.0 = Color::WHITE; // White for normal
        }
    }
}

/// System to convert option collection events to score update events
pub fn handle_option_collection_events(
    mut collection_events: EventReader<crate::player::OptionCollectedEvent>,
    mut score_events: EventWriter<ScoreUpdateEvent>,
) {
    for event in collection_events.read() {
        let points = if event.is_correct {
            super::CORRECT_ANSWER_POINTS as i32
        } else {
            super::WRONG_ANSWER_PENALTY
        };

        score_events.write(ScoreUpdateEvent {
            player_entity: event.player_entity,
            is_correct: event.is_correct,
            points_awarded: points,
        });
    }
}

/// System to handle chain segment destruction events and update score
pub fn handle_chain_destruction_events(
    mut destruction_events: EventReader<crate::chain::ChainSegmentDestroyedEvent>,
    mut gameplay_score: ResMut<GameplayScore>,
) {
    for event in destruction_events.read() {
        // Ensure player exists in the score tracking
        if !gameplay_score.players.contains_key(&event.player_entity) {
            gameplay_score.add_player(event.player_entity, "Player 1".to_string());
        }

        // Deduct points from player score
        if let Some(player_score) = gameplay_score.get_player_score_mut(event.player_entity) {
            player_score.total_score = (player_score.total_score - event.points_lost).max(0);
            info!(
                "Chain destruction! Segment {} ('{}') destroyed. Player lost {} points. Total: {}",
                event.segment_index, event.option_text, event.points_lost, player_score.total_score
            );
        }
    }
}
