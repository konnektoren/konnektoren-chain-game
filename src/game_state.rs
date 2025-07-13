use bevy::prelude::*;
use konnektoren_bevy::assets::*;

/// Resource to track the current game state and level
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct GameState {
    pub current_level_id: String,
    pub current_challenge_id: Option<String>,
    pub level_loaded: bool,
    pub challenge_loaded: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            current_level_id: "level-a1".to_string(),
            current_challenge_id: None,
            level_loaded: false,
            challenge_loaded: false,
        }
    }
}

impl GameState {
    pub fn is_ready(&self) -> bool {
        self.level_loaded && self.challenge_loaded
    }
}

/// System to update game state when assets are loaded
pub fn update_game_state(
    mut game_state: ResMut<GameState>,
    asset_registry: Option<Res<KonnektorenAssetRegistry>>,
    level_assets: Option<Res<Assets<LevelAsset>>>,
    challenge_assets: Option<Res<Assets<ChallengeAsset>>>,
) {
    let Some(registry) = asset_registry else {
        return;
    };

    // Check if level is loaded
    if !game_state.level_loaded {
        if let Some(level_handle) = registry.get_level_handle(&game_state.current_level_id) {
            if let Some(level_assets) = level_assets.as_ref() {
                if let Some(level_asset) = level_assets.get(level_handle) {
                    game_state.level_loaded = true;

                    // Get the first challenge from the level
                    if let Some(first_challenge) = level_asset.game_path.challenges.first() {
                        game_state.current_challenge_id = Some(first_challenge.challenge.clone());
                        info!(
                            "Level {} loaded, setting challenge to {}",
                            level_asset.name(),
                            first_challenge.challenge
                        );
                    } else {
                        warn!("Level {} has no challenges defined", level_asset.name());
                        // Fallback to articles challenge
                        game_state.current_challenge_id = Some("articles".to_string());
                    }
                }
            }
        }
    }

    // Check if challenge is loaded - fix the borrowing issue by cloning the challenge_id
    if !game_state.challenge_loaded {
        // Clone the challenge_id to avoid borrowing conflict
        if let Some(challenge_id) = game_state.current_challenge_id.clone() {
            if let Some(challenge_handle) = registry.get_challenge_handle(&challenge_id) {
                if let Some(challenge_assets) = challenge_assets.as_ref() {
                    if challenge_assets.get(challenge_handle).is_some() {
                        game_state.challenge_loaded = true;
                        info!("Challenge {} loaded", challenge_id);
                    }
                }
            }
        }
    }
}
