use bevy::prelude::*;
use bevy_egui::{
    EguiContextPass,
    egui::{self, Widget},
};
use konnektoren_bevy::prelude::*;

use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), setup_main_menu_marker);
    app.add_systems(
        EguiContextPass,
        main_menu_egui_ui.run_if(in_state(Menu::Main)),
    );
    app.add_systems(OnExit(Menu::Main), cleanup_main_menu_marker);
}

/// Marker component to help with cleanup if needed
#[derive(Component)]
struct MainMenuMarker;

fn setup_main_menu_marker(mut commands: Commands) {
    commands.spawn(MainMenuMarker);
}

fn cleanup_main_menu_marker(mut commands: Commands, query: Query<Entity, With<MainMenuMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn main_menu_egui_ui(
    mut contexts: bevy_egui::EguiContexts,
    theme: Res<KonnektorenTheme>,
    responsive: Res<ResponsiveInfo>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut next_screen: ResMut<NextState<Screen>>,
    resource_handles: Res<ResourceHandles>,
    #[cfg(not(target_family = "wasm"))] mut app_exit: EventWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(theme.base_100))
        .show(ctx, |ui| {
            // Calculate vertical centering
            let available_height = ui.available_height();
            let menu_height = 420.0; // Estimate your menu's height (adjust as needed)
            let top_space = ((available_height - menu_height) / 2.0).max(0.0);

            ui.add_space(top_space);

            ui.vertical_centered(|ui| {
                // Title
                ResponsiveText::new(
                    "Konnektoren Chain Game",
                    ResponsiveFontSize::Title,
                    theme.primary,
                )
                .responsive(&responsive)
                .strong()
                .ui(ui);

                ui.add_space(responsive.spacing(ResponsiveSpacing::Large));

                // Play button
                if ThemedButton::new("Play", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    if resource_handles.is_all_done() {
                        next_screen.set(Screen::Gameplay);
                    } else {
                        next_screen.set(Screen::Loading);
                    }
                }

                ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));

                // Settings button
                if ThemedButton::new("Settings", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    next_menu.set(Menu::Settings);
                }

                ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));

                // Credits button
                if ThemedButton::new("Credits", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    next_menu.set(Menu::Credits);
                }

                ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));

                // Website button
                if ThemedButton::new("konnektoren.help", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    open_konnektoren_website();
                }

                #[cfg(not(target_family = "wasm"))]
                {
                    ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));
                    if ThemedButton::new("Exit", &theme)
                        .responsive(&responsive)
                        .width(250.0)
                        .show(ui)
                        .clicked()
                    {
                        app_exit.write(AppExit::Success);
                    }
                }
            });

            // Optionally, add bottom space for perfect centering
            // ui.add_space(top_space);
        });
}

fn open_konnektoren_website() {
    let url = "https://konnektoren.help";
    info!("🚀 Opening konnektoren.help...");
    #[cfg(target_family = "wasm")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(not(target_family = "wasm"))]
    {
        let _ = webbrowser::open(url);
    }
}
