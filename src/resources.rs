use bevy::prelude::*;
use konnektoren_core::challenges::multiple_choice::MultipleChoice;

#[derive(Resource, Debug, Clone)]
pub struct MultipleChoiceChallenge(MultipleChoice);

impl Default for MultipleChoiceChallenge {
    fn default() -> Self {
        let challenge: MultipleChoice =
            serde_yaml::from_str(include_str!("../assets/articles.yml"))
                .expect("Failed to parse multiple choice data");
        Self(challenge)
    }
}

impl MultipleChoiceChallenge {
    pub fn get(&self) -> &MultipleChoice {
        &self.0
    }
}
