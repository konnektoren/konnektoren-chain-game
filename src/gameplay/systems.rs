use super::components::*;
use crate::screens::Screen;
use crate::settings::GameSettings;
use bevy::prelude::*;

/// System to set up the gameplay UI
pub fn setup_gameplay_ui(mut commands: Commands, game_settings: Res<GameSettings>) {
    let player_count = game_settings.multiplayer.player_count;

    // Score and timer overlay at the top right
    let ui_root = commands
        .spawn((
            Name::new("Gameplay UI"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::End,
                max_width: Val::Px(400.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderRadius::all(Val::Px(8.0)),
            StateScoped(Screen::Gameplay),
        ))
        .id();

    // Timer display
    let timer_entity = commands
        .spawn((
            Name::new("Timer Display"),
            Text("05:00".to_string()),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
            TimerDisplay,
        ))
        .id();

    // Player scores container
    let scores_container = commands
        .spawn((
            Name::new("Player Scores Container"),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                align_items: AlignItems::Stretch,
                width: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();

    // Create individual player score panels
    let mut player_panels = Vec::new();
    for i in 0..player_count {
        let player_settings = &game_settings.multiplayer.players[i];
        let player_data = PlayerScoreData {
            name: player_settings.name.clone(),
            color: player_settings.color,
        };

        let panel_entity = spawn_player_score_panel(&mut commands, i, &player_data, player_count);
        player_panels.push(panel_entity);
    }

    // Team stats display
    let team_stats = commands
        .spawn((
            Name::new("Team Stats Display"),
            Text("Team Stats: Loading...".to_string()),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            TeamStatsDisplay,
        ))
        .id();

    // Options display panel
    let options_panel = spawn_options_panel(&mut commands);

    // Set up parent-child relationships
    commands.entity(ui_root).add_children(&[
        timer_entity,
        scores_container,
        team_stats,
        options_panel,
    ]);
    commands
        .entity(scores_container)
        .add_children(&player_panels);
}

fn spawn_player_score_panel(
    commands: &mut Commands,
    player_index: usize,
    player_data: &PlayerScoreData,
    player_count: usize,
) -> Entity {
    // Create the panel entity
    let panel_entity = commands
        .spawn((
            Name::new(format!("Player {} Score Panel", player_index + 1)),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
            BorderColor(player_data.color),
            BorderRadius::all(Val::Px(5.0)),
        ))
        .id();

    // Create score text
    let score_text = commands
        .spawn((
            Name::new(format!("Player {} Score Text", player_index + 1)),
            Text(format!("{}: 0", player_data.name)),
            TextFont {
                font_size: if player_count > 2 { 14.0 } else { 16.0 },
                ..default()
            },
            TextColor(player_data.color),
            PlayerScoreDisplay { player_index },
        ))
        .id();

    // Create stats text
    let stats_text = commands
        .spawn((
            Name::new(format!("Player {} Stats Text", player_index + 1)),
            Text("Streak: 0 | Accuracy: 0%".to_string()),
            TextFont {
                font_size: if player_count > 2 { 10.0 } else { 12.0 },
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            PlayerStatsDisplay { player_index },
        ))
        .id();

    // Set up parent-child relationship
    commands
        .entity(panel_entity)
        .add_children(&[score_text, stats_text]);

    panel_entity
}

// Helper struct to hold player data
#[derive(Clone)]
struct PlayerScoreData {
    name: String,
    color: Color,
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
        info!(
            "Processing score event for player {:?}: correct={}, points={}",
            event.player_entity, event.is_correct, event.points_awarded
        );

        // Ensure player exists in the score tracking
        if !gameplay_score.players.contains_key(&event.player_entity) {
            gameplay_score.add_player(event.player_entity, "Player".to_string());
            info!(
                "Added new player {:?} to score tracking",
                event.player_entity
            );
        }

        // Update player score
        if let Some(player_score) = gameplay_score.get_player_score_mut(event.player_entity) {
            let old_score = player_score.total_score;
            let old_streak = player_score.current_streak;
            let old_collection_count = player_score.collection_count;

            if event.is_correct {
                player_score.add_correct_answer();
                info!(
                    "Player scored! Score: {} -> {}, Streak: {} -> {}, Collections: {} -> {}",
                    old_score,
                    player_score.total_score,
                    old_streak,
                    player_score.current_streak,
                    old_collection_count,
                    player_score.collection_count
                );
            } else {
                player_score.add_wrong_answer();
                info!(
                    "Player lost points! Score: {} -> {}, Streak reset: {} -> {}, Collections: {} -> {}",
                    old_score,
                    player_score.total_score,
                    old_streak,
                    player_score.current_streak,
                    old_collection_count,
                    player_score.collection_count
                );
            }
        } else {
            error!(
                "Failed to get mutable player score for entity {:?}",
                event.player_entity
            );
        }
    }
}

/// System to update individual player score displays
pub fn update_individual_player_scores(
    gameplay_score: Res<GameplayScore>,
    game_settings: Res<GameSettings>,
    mut player_score_query: Query<(&mut Text, &PlayerScoreDisplay)>,
    mut player_stats_query: Query<(&mut Text, &PlayerStatsDisplay), Without<PlayerScoreDisplay>>,
    player_query: Query<(Entity, &crate::player::PlayerIndex), With<crate::player::Player>>,
) {
    if !gameplay_score.is_changed() {
        return;
    }

    // Debug: Print all players and their indices
    info!("=== Player Index Debug ===");
    for (entity, player_index) in &player_query {
        info!("Entity {:?} has PlayerIndex({})", entity, player_index.0);
        if let Some(score) = gameplay_score.players.get(&entity) {
            info!(
                "  - Score: {}, Name: {}",
                score.total_score, score.player_name
            );
        } else {
            info!("  - No score data found");
        }
    }

    // Update individual player scores
    for (mut text, score_display) in &mut player_score_query {
        let player_index = score_display.player_index;

        // Find the player entity for this specific index
        let player_entity = player_query
            .iter()
            .find(|(_, idx)| idx.0 == player_index)
            .map(|(entity, _)| entity);

        info!(
            "Looking for PlayerIndex({}) -> Found entity: {:?}",
            player_index, player_entity
        );

        if let Some(entity) = player_entity {
            if let Some(player_score) = gameplay_score.players.get(&entity) {
                if let Some(player_settings) = game_settings.multiplayer.players.get(player_index) {
                    let new_text =
                        format!("{}: {}", player_settings.name, player_score.total_score);
                    info!(
                        "Updating score display for {}: {}",
                        player_settings.name, player_score.total_score
                    );
                    text.0 = new_text;
                } else {
                    text.0 = format!("Player {}: {}", player_index + 1, player_score.total_score);
                }
            } else {
                info!("No score data for entity {:?}", entity);
                if let Some(player_settings) = game_settings.multiplayer.players.get(player_index) {
                    text.0 = format!("{}: 0", player_settings.name);
                } else {
                    text.0 = format!("Player {}: 0", player_index + 1);
                }
            }
        } else {
            // Fallback if player entity not found
            info!("No entity found for PlayerIndex({})", player_index);
            if let Some(player_settings) = game_settings.multiplayer.players.get(player_index) {
                text.0 = format!("{}: 0", player_settings.name);
            } else {
                text.0 = format!("Player {}: 0", player_index + 1);
            }
        }
    }

    // Update individual player stats
    for (mut text, stats_display) in &mut player_stats_query {
        let player_index = stats_display.player_index;

        // Find the player entity for this specific index
        let player_entity = player_query
            .iter()
            .find(|(_, idx)| idx.0 == player_index)
            .map(|(entity, _)| entity);

        if let Some(entity) = player_entity {
            if let Some(player_score) = gameplay_score.players.get(&entity) {
                let accuracy = if player_score.collection_count > 0 {
                    (player_score.correct_answers as f32 / player_score.collection_count as f32)
                        * 100.0
                } else {
                    0.0
                };

                // Show current streak and best streak
                let new_text = format!(
                    "Current: {} | Best: {} | Accuracy: {:.0}%",
                    player_score.current_streak, player_score.best_streak, accuracy
                );

                info!(
                    "Player {} stats - Current: {}, Best: {}, Correct: {}, Total: {}, Accuracy: {:.1}%",
                    player_index + 1,
                    player_score.current_streak,
                    player_score.best_streak,
                    player_score.correct_answers,
                    player_score.collection_count,
                    accuracy
                );

                text.0 = new_text;
            } else {
                text.0 = "Current: 0 | Best: 0 | Accuracy: 0%".to_string();
            }
        } else {
            text.0 = "Current: 0 | Best: 0 | Accuracy: 0%".to_string();
        }
    }
}

/// System to update team stats display
pub fn update_team_stats_display(
    gameplay_score: Res<GameplayScore>,
    game_settings: Res<GameSettings>,
    mut team_stats_query: Query<&mut Text, With<TeamStatsDisplay>>,
) {
    if !gameplay_score.is_changed() {
        return;
    }

    for mut text in &mut team_stats_query {
        // Debug: Print all players in the score tracking
        info!(
            "Team stats update - Players in score tracking: {}",
            gameplay_score.players.len()
        );
        for (entity, player_score) in &gameplay_score.players {
            info!(
                "  Player {:?}: {} points, current streak: {}, best streak: {}, {} correct, {} total collections",
                entity,
                player_score.total_score,
                player_score.current_streak,
                player_score.best_streak,
                player_score.correct_answers,
                player_score.collection_count
            );
        }

        if game_settings.multiplayer.enabled && game_settings.multiplayer.player_count > 1 {
            // Show combined stats for multiplayer
            let best_streak_overall: u32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.best_streak)
                .max()
                .unwrap_or(0);

            let best_current_streak: u32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.current_streak)
                .max()
                .unwrap_or(0);

            let total_correct: u32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.correct_answers)
                .sum();

            let total_attempts: u32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.collection_count)
                .sum();

            let total_score: i32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.total_score)
                .sum();

            let team_accuracy = if total_attempts > 0 {
                (total_correct as f32 / total_attempts as f32) * 100.0
            } else {
                0.0
            };

            // Debug log the calculations
            info!(
                "Team calculations - Total Score: {}, Best Overall Streak: {}, Best Current Streak: {}, Total Correct: {}, Total Attempts: {}, Team Accuracy: {:.1}%",
                total_score,
                best_streak_overall,
                best_current_streak,
                total_correct,
                total_attempts,
                team_accuracy
            );

            // Show both current best streak and all-time best streak
            text.0 = format!(
                "Team: {} pts | Current Best: {} | All-Time Best: {} | Accuracy: {:.0}%",
                total_score, best_current_streak, best_streak_overall, team_accuracy
            );
        } else {
            // Single player stats
            let player_score = gameplay_score.players.values().next();
            if let Some(score) = player_score {
                let accuracy = if score.collection_count > 0 {
                    (score.correct_answers as f32 / score.collection_count as f32) * 100.0
                } else {
                    0.0
                };
                text.0 = format!(
                    "Current: {} | Best: {} | Accuracy: {:.0}%",
                    score.current_streak, score.best_streak, accuracy
                );
            } else {
                text.0 = "Current: 0 | Best: 0 | Accuracy: 0%".to_string();
            }
        }
    }
}

/// System to update score display for multiplayer (legacy support)
pub fn update_score_display(
    gameplay_score: Res<GameplayScore>,
    game_settings: Res<GameSettings>,
    mut score_query: Query<
        &mut Text,
        (
            With<ScoreDisplay>,
            Without<PlayerScoreDisplay>,
            Without<TeamStatsDisplay>,
        ),
    >,
) {
    if !gameplay_score.is_changed() {
        return;
    }

    // This system is kept for backward compatibility if there are any legacy ScoreDisplay components
    for mut text in &mut score_query {
        if game_settings.multiplayer.enabled && game_settings.multiplayer.player_count > 1 {
            let total_score: i32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.total_score)
                .sum();
            text.0 = format!("Team Total: {}", total_score);
        } else {
            let total_score: i32 = gameplay_score
                .players
                .values()
                .map(|ps| ps.total_score)
                .sum();
            text.0 = format!("Score: {}", total_score);
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
    mut gameplay_score: ResMut<GameplayScore>,
    game_settings: Res<GameSettings>,
    player_query: Query<&crate::player::PlayerIndex, With<crate::player::Player>>,
) {
    for event in collection_events.read() {
        info!(
            "Option collection event: player={:?}, correct={}",
            event.player_entity, event.is_correct
        );

        // Debug: Check if this player has a PlayerIndex
        match player_query.get(event.player_entity) {
            Ok(player_index) => {
                info!(
                    "Player {:?} has PlayerIndex({})",
                    event.player_entity, player_index.0
                );
            }
            Err(_) => {
                error!(
                    "Player {:?} not found in player_query or missing PlayerIndex component!",
                    event.player_entity
                );
            }
        }

        // Ensure player exists in the score tracking
        if !gameplay_score.players.contains_key(&event.player_entity) {
            // Get the player name from game settings using the PlayerIndex
            let player_name = if let Ok(player_index) = player_query.get(event.player_entity) {
                let name = game_settings
                    .multiplayer
                    .players
                    .get(player_index.0)
                    .map(|ps| ps.name.clone())
                    .unwrap_or_else(|| format!("Player {}", player_index.0 + 1));
                info!(
                    "Using player name '{}' from settings for PlayerIndex({})",
                    name, player_index.0
                );
                name
            } else {
                let fallback_name = format!("Player {}", gameplay_score.players.len() + 1);
                error!(
                    "Could not get PlayerIndex for {:?}, using fallback name '{}'",
                    event.player_entity, fallback_name
                );
                fallback_name
            };

            gameplay_score.add_player(event.player_entity, player_name);
            info!(
                "Added player {:?} with name '{}' to score tracking. Total players now: {}",
                event.player_entity,
                gameplay_score
                    .players
                    .get(&event.player_entity)
                    .unwrap()
                    .player_name,
                gameplay_score.players.len()
            );
        } else {
            info!(
                "Player {:?} already exists in score tracking",
                event.player_entity
            );
        }

        let points = if event.is_correct {
            super::CORRECT_ANSWER_POINTS as i32
        } else {
            super::WRONG_ANSWER_PENALTY
        };

        let score_event = ScoreUpdateEvent {
            player_entity: event.player_entity,
            is_correct: event.is_correct,
            points_awarded: points,
        };
        score_events.write(score_event);
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
            gameplay_score.add_player(event.player_entity, "Player".to_string());
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

/// System to update options display in the HUD
pub fn update_options_display(
    question_system: Option<Res<crate::question::QuestionSystem>>,
    options_panel_query: Query<&Children, With<OptionsDisplay>>,
    mut commands: Commands,
    container_query: Query<(Entity, &Children, &Name)>,
    existing_options: Query<(Entity, &OptionItemDisplay)>,
) {
    let Some(question_system) = question_system else {
        info!("No question system found");
        return;
    };

    let Some(current_question) = question_system.get_current_question() else {
        info!("No current question available");
        return;
    };

    let options = question_system.get_current_options();
    info!("Found {} options to display", options.len());

    // Find the options panel
    let Ok(panel_children) = options_panel_query.single() else {
        info!("Options panel not found");
        return;
    };

    info!("Options panel found with {} children", panel_children.len());

    // Find the options container
    let mut options_container = None;
    for child in panel_children.iter() {
        if let Ok((entity, _, name)) = container_query.get(child) {
            info!("Found child: {}", name.as_str());
            if name.as_str() == "Options Container" {
                options_container = Some(entity);
                break;
            }
        }
    }

    let Some(container_entity) = options_container else {
        info!("Options container not found in panel children");
        return;
    };

    info!("Found options container: {:?}", container_entity);

    // Clear existing option displays
    let existing_count = existing_options.iter().count();
    for (entity, _) in &existing_options {
        commands.entity(entity).despawn();
    }
    info!("Cleared {} existing option displays", existing_count);

    // Create new option displays
    for (index, option) in options.iter().enumerate() {
        info!(
            "Creating option display for: {} (id: {})",
            option.name, option.id
        );

        let is_correct = option.id == current_question.option;

        // Choose color based on option ID (same as the collectibles)
        let base_colors = [
            Color::srgb(0.3, 0.5, 0.8), // Blue
            Color::srgb(0.8, 0.5, 0.3), // Orange
            Color::srgb(0.5, 0.8, 0.3), // Green
            Color::srgb(0.8, 0.3, 0.5), // Pink
            Color::srgb(0.5, 0.3, 0.8), // Purple
        ];
        let color = base_colors[option.id % base_colors.len()];

        // Make correct answers brighter
        let display_color = if is_correct {
            Color::srgb(
                (color.to_srgba().red * 1.3).min(1.0),
                (color.to_srgba().green * 1.3).min(1.0),
                (color.to_srgba().blue * 1.3).min(1.0),
            )
        } else {
            color
        };

        // Create the main option container
        let option_entity = commands
            .spawn((
                Name::new(format!("Option Display: {}", option.name)),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(if is_correct { 2.0 } else { 1.0 })),
                    column_gap: Val::Px(8.0),
                    min_height: Val::Px(24.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(
                    display_color.to_srgba().red * 0.3,
                    display_color.to_srgba().green * 0.3,
                    display_color.to_srgba().blue * 0.3,
                    0.6,
                )),
                BorderColor(if is_correct {
                    Color::srgb(1.0, 0.9, 0.3) // Golden border for correct answer
                } else {
                    display_color
                }),
                BorderRadius::all(Val::Px(3.0)),
                OptionItemDisplay {
                    option_id: option.id,
                },
            ))
            .id();

        // Create color indicator circle
        let color_indicator = commands
            .spawn((
                Name::new("Option Color"),
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(display_color),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .id();

        // Create option text
        let option_text = commands
            .spawn((
                Name::new("Option Text"),
                Text(option.name.clone()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(if is_correct {
                    Color::srgb(1.0, 0.9, 0.3) // Golden text for correct answer
                } else {
                    Color::WHITE
                }),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ))
            .id();

        // Create correct answer indicator if needed
        let mut children = vec![color_indicator, option_text];

        if is_correct {
            let correct_indicator = commands
                .spawn((
                    Name::new("Correct Indicator"),
                    Text("✓".to_string()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.3)),
                ))
                .id();
            children.push(correct_indicator);
        }

        // Set up parent-child relationships
        commands.entity(option_entity).add_children(&children);
        commands.entity(container_entity).add_child(option_entity);

        info!("Created option display {} for '{}'", index, option.name);
    }

    info!("Updated options display with {} options", options.len());
}

fn spawn_options_panel(commands: &mut Commands) -> Entity {
    // Create options header
    let options_header = commands
        .spawn((
            Name::new("Options Header"),
            Text("Available Options:".to_string()),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ))
        .id();

    // Create options container
    let options_container = commands
        .spawn((
            Name::new("Options Container"),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                align_items: AlignItems::Stretch,
                ..default()
            },
        ))
        .id();

    // Create the main options panel
    let panel = commands
        .spawn((
            Name::new("Options Panel"),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::top(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                row_gap: Val::Px(5.0),
                align_items: AlignItems::Stretch,
                width: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.7)),
            BorderColor(Color::srgb(0.3, 0.3, 0.4)),
            BorderRadius::all(Val::Px(5.0)),
            OptionsDisplay,
        ))
        .id();

    // Set up parent-child relationships
    commands
        .entity(panel)
        .add_children(&[options_header, options_container]);

    panel
}
