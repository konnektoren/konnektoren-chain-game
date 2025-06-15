//! A splash screen that plays briefly at startup using konnektoren-bevy.

use bevy::prelude::*;
use konnektoren_bevy::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // The ScreensPlugin from konnektoren-bevy handles the splash functionality
    // We just need to set up our game-specific splash screen
    app.add_systems(OnEnter(Screen::Splash), spawn_splash_screen)
        .add_systems(OnExit(Screen::Splash), cleanup_splash_screen)
        .add_systems(
            Update,
            handle_splash_events.run_if(in_state(Screen::Splash)),
        );
}

fn spawn_splash_screen(mut commands: Commands) {
    // Create a custom splash screen configuration for the chain game
    let splash_config = SplashConfig::new("Konnektoren Chain Game")
        .with_image_logo("logo.png")
        .with_subtitle("Connect the Words")
        .with_duration(1.8)
        .with_manual_dismissal(true) // Allow escape key dismissal
        .with_logo_size(1.2); // Slightly larger logo

    commands.spawn_splash(splash_config);
}

fn cleanup_splash_screen(
    mut commands: Commands,
    splash_entities: Query<Entity, With<SplashConfig>>,
) {
    // Clean up any splash screen entities
    for entity in splash_entities.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_splash_events(
    mut splash_events: EventReader<SplashDismissed>,
    mut next_screen: ResMut<NextState<Screen>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // Handle splash screen dismissal
    for _event in splash_events.read() {
        next_screen.set(Screen::Title);
    }

    // Handle escape key for early exit (additional to the built-in handling)
    if input.just_pressed(KeyCode::Escape) {
        next_screen.set(Screen::Title);
    }
}
