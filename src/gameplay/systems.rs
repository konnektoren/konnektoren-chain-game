use super::components::*;
use crate::screens::Screen;
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
    time: Res<Time>,
) {
    // Reset gameplay score
    gameplay_score.players.clear();
    gameplay_score.game_active = true;
    gameplay_score.game_start_time = time.elapsed_secs();

    // Reset game timer
    *game_timer = GameTimer::default();

    info!("Game state reset - new game started!");
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

/// System to update score display
pub fn update_score_display(
    gameplay_score: Res<GameplayScore>,
    mut score_query: Query<&mut Text, With<ScoreDisplay>>,
    mut stats_query: Query<&mut Text, (Without<ScoreDisplay>, Without<TimerDisplay>)>,
) {
    // Get the first (and currently only) player's score
    let player_score = gameplay_score.players.values().next();

    for mut text in &mut score_query {
        if let Some(score) = player_score {
            text.0 = format!("Score: {}", score.total_score);
        } else {
            text.0 = "Score: 0".to_string();
        }
    }

    // Update additional stats
    for mut text in stats_query.iter_mut().take(1) {
        // Only update the first stats display
        if let Some(score) = player_score {
            text.0 = format!(
                "Streak: {} | Accuracy: {:.0}%",
                score.current_streak,
                score.accuracy()
            );
        } else {
            text.0 = "Streak: 0 | Accuracy: 0%".to_string();
        }
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
