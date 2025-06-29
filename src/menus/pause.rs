use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{
    EguiContextPass,
    egui::{self, Widget},
};
use konnektoren_bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        EguiContextPass,
        pause_menu_egui_ui.run_if(in_state(Menu::Pause)),
    );
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
    );
}

fn pause_menu_egui_ui(
    mut contexts: bevy_egui::EguiContexts,
    theme: Res<KonnektorenTheme>,
    responsive: Res<ResponsiveInfo>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let ctx = contexts.ctx_mut();

    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(theme.base_100))
        .show(ctx, |ui| {
            // Vertically center the menu
            let available_height = ui.available_height();
            let menu_height = 320.0;
            let top_space = ((available_height - menu_height) / 2.0).max(0.0);
            ui.add_space(top_space);

            ui.vertical_centered(|ui| {
                ResponsiveText::new("Game paused", ResponsiveFontSize::Title, theme.primary)
                    .responsive(&responsive)
                    .strong()
                    .ui(ui);

                ui.add_space(responsive.spacing(ResponsiveSpacing::Large));

                // Continue
                if ThemedButton::new("Continue", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    next_menu.set(Menu::None);
                }

                ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));

                // Settings
                if ThemedButton::new("Settings", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    next_menu.set(Menu::Settings);
                }

                ui.add_space(responsive.spacing(ResponsiveSpacing::Medium));

                // Quit to title
                if ThemedButton::new("Quit to title", &theme)
                    .responsive(&responsive)
                    .width(250.0)
                    .show(ui)
                    .clicked()
                {
                    next_screen.set(Screen::Title);
                }
            });
        });
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
