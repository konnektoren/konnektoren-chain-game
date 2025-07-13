use bevy::prelude::*;
use konnektoren_bevy::assets::*;
use konnektoren_core::challenges::multiple_choice::MultipleChoice;

#[derive(Resource, Debug, Clone)]
pub struct MultipleChoiceChallenge(MultipleChoice);

impl MultipleChoiceChallenge {
    pub fn get(&self) -> &MultipleChoice {
        &self.0
    }

    /// Create from a loaded ChallengeAsset
    pub fn from_asset(challenge_asset: &ChallengeAsset) -> Option<Self> {
        if let konnektoren_core::challenges::challenge_type::ChallengeType::MultipleChoice(mc) =
            &challenge_asset.challenge_type
        {
            Some(Self(mc.clone()))
        } else {
            None
        }
    }

    /// Try to load from asset system
    pub fn from_asset_system(
        asset_registry: &KonnektorenAssetRegistry,
        challenge_assets: &Assets<ChallengeAsset>,
        challenge_id: &str,
    ) -> Option<Self> {
        if let Some(challenge_handle) = asset_registry.get_challenge_handle(challenge_id) {
            if let Some(challenge_asset) = challenge_assets.get(challenge_handle) {
                if let Some(challenge) = Self::from_asset(challenge_asset) {
                    info!("Loaded challenge '{}' from assets", challenge_id);
                    return Some(challenge);
                }
            }
        }

        warn!("Could not load challenge '{}' from assets", challenge_id);
        None
    }
}
