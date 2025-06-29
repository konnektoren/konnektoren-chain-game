use crate::menus::Menu;
use bevy::prelude::*;
use konnektoren_bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_screen);
    app.add_systems(
        Update,
        handle_credits_events.run_if(in_state(Menu::Credits)),
    );
    app.add_systems(OnExit(Menu::Credits), cleanup_credits_screen);
}

fn spawn_credits_screen(mut commands: Commands) {
    // You can customize this config as you like!
    let credits_config = CreditsConfig::new("Konnektoren Chain Game")
        .with_subtitle("Game Credits")
        .add_team_member("chriamue", "Implemented game, assisted by Claude")
        .add_asset("Button SFX", "CC0 by Jaszunio15")
        .add_asset("Music", "CC BY 3.0 by Kevin MacLeod")
        .add_asset("Gameplay SFX", "Licensed from Ovani Sound")
        .add_asset("Audio License", "https://ovanisound.com/policies/terms-of-service")
        .add_asset("Bevy logo", "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified")
        .add_special_thanks("You!", "For playing the game!")
        .with_dismiss_button_text("← Back");

    commands.spawn((
        Name::new("Credits Screen"),
        credits_config,
        StateScoped(Menu::Credits),
    ));
}

fn handle_credits_events(
    mut events: EventReader<CreditsDismissed>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    for _ in events.read() {
        next_menu.set(Menu::Main);
    }
}

fn cleanup_credits_screen(
    mut commands: Commands,
    config_query: Query<Entity, With<CreditsConfig>>,
    active_query: Query<Entity, With<ActiveCredits>>,
) {
    for entity in config_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in active_query.iter() {
        commands.entity(entity).despawn();
    }
}
