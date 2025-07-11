use super::*;
use crate::settings as game_settings;
use bevy_egui::EguiPlugin;
use konnektoren_bevy::prelude::*;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Konnektoren Chain Game".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        );

        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        });

        app.add_plugins((
            KonnektorenThemePlugin,
            UIPlugin,
            InputPlugin,
            ScreensPlugin,
            SettingsPlugin,
        ));

        // Initialize resources
        app.init_resource::<resources::MultipleChoiceChallenge>();

        app.add_plugins((game_settings::plugin,));

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            camera::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            map::plugin,
            player::plugin,
            chain::plugin,
            menus::plugin,
            options::plugin,
            question::plugin,
            screens::plugin,
            gameplay::plugin,
            theme::plugin,
            effects::plugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}
