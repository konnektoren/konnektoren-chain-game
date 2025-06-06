use bevy::prelude::*;
use konnektoren_core::challenges::multiple_choice::{
    MultipleChoice, MultipleChoiceOption, Question,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Resource that manages the overall question system
#[derive(Resource, Clone)]
pub struct QuestionSystem {
    pub current_question_index: usize,
    pub questions: Vec<Question>,
    pub options: Vec<MultipleChoiceOption>,
    pub question_order: Vec<usize>,
    pub rng: StdRng,
}

impl QuestionSystem {
    pub fn new(multiple_choice: &MultipleChoice, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        // Create randomized question order
        let mut question_order: Vec<usize> = (0..multiple_choice.questions.len()).collect();

        // Fisher-Yates shuffle
        for i in (1..question_order.len()).rev() {
            let j = rng.gen_range(0..=i);
            question_order.swap(i, j);
        }

        Self {
            current_question_index: 0,
            questions: multiple_choice.questions.clone(),
            options: multiple_choice.options.clone(),
            question_order,
            rng,
        }
    }

    pub fn get_current_question(&self) -> Option<&Question> {
        let shuffled_index = self.question_order.get(self.current_question_index)?;
        self.questions.get(*shuffled_index)
    }

    pub fn get_current_options(&self) -> &Vec<MultipleChoiceOption> {
        &self.options
    }

    pub fn advance_question(&mut self) {
        self.current_question_index = (self.current_question_index + 1) % self.question_order.len();

        // Re-shuffle if we've gone through all questions
        if self.current_question_index == 0 {
            self.reshuffle_questions();
        }
    }

    fn reshuffle_questions(&mut self) {
        // Fisher-Yates shuffle
        for i in (1..self.question_order.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            self.question_order.swap(i, j);
        }
    }
}

/// Timer component for question changes
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuestionTimer {
    pub timer: Timer,
    pub fade_timer: Timer,
    pub is_fading: bool,
    pub fade_in: bool,
}

impl Default for QuestionTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(super::QUESTION_DURATION, TimerMode::Repeating),
            fade_timer: Timer::from_seconds(super::QUESTION_FADE_DURATION, TimerMode::Once),
            is_fading: false,
            fade_in: true,
        }
    }
}

/// Component for the question display UI
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuestionDisplay;

/// Component for the help text display
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct QuestionHelpDisplay;

/// Resource for the random seed
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct QuestionSeed(pub u64);

impl Default for QuestionSeed {
    fn default() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self(seed)
    }
}
