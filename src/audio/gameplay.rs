use crate::{
    asset_tracking::LoadResource,
    audio::{music, sound_effect},
    player::OptionCollectedEvent,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GameplayAudioAssets>();
    app.load_resource::<GameplayAudioAssets>();

    // Add music system
    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        start_gameplay_music,
    );

    app.add_systems(
        Update,
        handle_option_collection_audio.run_if(in_state(crate::screens::Screen::Gameplay)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameplayAudioAssets {
    #[dependency]
    pub correct_sound: Handle<AudioSource>,
    #[dependency]
    pub incorrect_sound: Handle<AudioSource>,
    #[dependency]
    pub background_music: Handle<AudioSource>,
}

impl FromWorld for GameplayAudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            correct_sound: assets.load("audio/sound_effects/Coin 001.ogg"),
            incorrect_sound: assets.load("audio/sound_effects/UI Negative Signal 002.ogg"),
            background_music: assets.load("audio/music/Monkeys Spinning Monkeys.ogg"),
        }
    }
}

/// System to start background music when entering gameplay
fn start_gameplay_music(mut commands: Commands, gameplay_audio: Option<Res<GameplayAudioAssets>>) {
    let Some(audio_assets) = gameplay_audio else {
        warn!("Gameplay audio assets not loaded yet");
        return;
    };

    commands.spawn((
        Name::new("Gameplay Background Music"),
        StateScoped(crate::screens::Screen::Gameplay),
        music(audio_assets.background_music.clone()),
    ));

    info!("Started gameplay background music");
}

/// System to play audio feedback when options are collected
fn handle_option_collection_audio(
    mut commands: Commands,
    mut collection_events: EventReader<OptionCollectedEvent>,
    gameplay_audio: Option<Res<GameplayAudioAssets>>,
) {
    let Some(audio_assets) = gameplay_audio else {
        return;
    };

    for event in collection_events.read() {
        let sound_handle = if event.is_correct {
            audio_assets.correct_sound.clone()
        } else {
            audio_assets.incorrect_sound.clone()
        };

        commands.spawn((
            Name::new(if event.is_correct {
                "Correct Answer Sound"
            } else {
                "Incorrect Answer Sound"
            }),
            sound_effect(sound_handle),
        ));

        info!(
            "Playing {} sound for option: {}",
            if event.is_correct {
                "correct"
            } else {
                "incorrect"
            },
            event.option_text
        );
    }
}
