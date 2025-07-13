use super::components::*;
use crate::{game_state::GameState, resources::MultipleChoiceChallenge, screens::Screen};
use bevy::prelude::*;
use konnektoren_bevy::assets::*;

/// System to set up the question system when entering gameplay
pub fn setup_question_system(
    mut commands: Commands,
    time: Res<Time>,
    game_state: Res<GameState>,
    asset_registry: Option<Res<KonnektorenAssetRegistry>>,
    challenge_assets: Option<Res<Assets<ChallengeAsset>>>,
) {
    // Wait for game state to be ready
    if !game_state.is_ready() {
        info!("Waiting for level and challenge assets to load...");
        return;
    }

    let Some(challenge_id) = &game_state.current_challenge_id else {
        error!("No challenge ID available in game state");
        return;
    };

    // Load the challenge using the asset system
    let Some((registry, assets)) = asset_registry.zip(challenge_assets) else {
        error!("Asset system not available - cannot load challenge");
        return;
    };

    let Some(multiple_choice_challenge) =
        MultipleChoiceChallenge::from_asset_system(&registry, &assets, challenge_id)
    else {
        error!("Failed to load challenge '{}' from assets", challenge_id);
        return;
    };

    let multiple_choice = multiple_choice_challenge.get();

    info!(
        "Setting up question system with {} questions from challenge '{}'",
        multiple_choice.questions.len(),
        challenge_id
    );

    // Use Bevy's elapsed time as seed (works on all platforms)
    let seed = (time.elapsed_secs() * 1000000.0) as u64;

    // Initialize the question system
    let question_system = QuestionSystem::new(multiple_choice, seed);

    // Spawn the question UI
    spawn_question_ui(&mut commands, &question_system);

    // Insert the question system as a resource
    commands.insert_resource(question_system);

    // Also insert the challenge resource for other systems that might need it
    commands.insert_resource(multiple_choice_challenge);
}

/// Spawn the question UI overlay
fn spawn_question_ui(commands: &mut Commands, question_system: &QuestionSystem) {
    let current_question = question_system
        .get_current_question()
        .expect("Should have at least one question");

    commands.spawn((
        Name::new("Question Overlay"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent background
        BorderRadius::all(Val::Px(10.0)),
        StateScoped(Screen::Gameplay),
        QuestionTimer::default(),
        children![
            // Question text
            (
                Name::new("Question Text"),
                Text(current_question.question.clone()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                QuestionDisplay,
            ),
            // Help text
            (
                Name::new("Help Text"),
                Text(if current_question.help.is_empty() {
                    "Choose the correct answer...".to_string()
                } else {
                    current_question.help.clone()
                }),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                QuestionHelpDisplay,
            ),
        ],
    ));
}

/// System to update the question timer and handle question changes
pub fn update_question_timer(
    time: Res<Time>,
    mut question_system: ResMut<QuestionSystem>,
    mut timer_query: Query<&mut QuestionTimer>,
) {
    for mut question_timer in &mut timer_query {
        // Update main timer
        question_timer.timer.tick(time.delta());

        // Handle fading
        if question_timer.is_fading {
            question_timer.fade_timer.tick(time.delta());

            if question_timer.fade_timer.finished() {
                if !question_timer.fade_in {
                    // Fade out finished, change question and start fade in
                    question_system.advance_question();
                    question_timer.fade_in = true;
                    question_timer.fade_timer.reset();
                } else {
                    // Fade in finished, stop fading
                    question_timer.is_fading = false;
                }
            }
        }

        // Check if it's time to change question
        if question_timer.timer.just_finished() && !question_timer.is_fading {
            // Start fade out
            question_timer.is_fading = true;
            question_timer.fade_in = false;
            question_timer.fade_timer.reset();
        }
    }
}

/// System to update the question display when questions change
pub fn update_question_display(
    question_system: Res<QuestionSystem>,
    timer_query: Query<&QuestionTimer>,
    mut question_query: Query<&mut Text, (With<QuestionDisplay>, Without<QuestionHelpDisplay>)>,
    mut help_query: Query<&mut Text, (With<QuestionHelpDisplay>, Without<QuestionDisplay>)>,
    mut ui_query: Query<&mut BackgroundColor, With<QuestionTimer>>,
) {
    if question_system.is_changed() {
        // Update question text
        if let Some(current_question) = question_system.get_current_question() {
            for mut text in &mut question_query {
                text.0 = current_question.question.clone();
                info!("Updated question text to: {}", current_question.question);
            }

            for mut text in &mut help_query {
                text.0 = if current_question.help.is_empty() {
                    "Choose the correct answer...".to_string()
                } else {
                    current_question.help.clone()
                };
            }
        }
    }

    // Handle fade effects
    for question_timer in &timer_query {
        if question_timer.is_fading {
            let fade_progress = question_timer.fade_timer.fraction();
            let alpha = if question_timer.fade_in {
                fade_progress * 0.7 // Fade in to 70% opacity
            } else {
                (1.0 - fade_progress) * 0.7 // Fade out from 70% opacity
            };

            for mut background in &mut ui_query {
                *background = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));
            }
        } else {
            // Ensure full opacity when not fading
            for mut background in &mut ui_query {
                *background = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7));
            }
        }
    }
}
