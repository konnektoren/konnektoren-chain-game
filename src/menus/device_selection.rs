use crate::menus::Menu;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Menu::DeviceSelection),
        (
            crate::settings::device_selection_ui::spawn_device_selection_ui,
            crate::settings::systems::clear_device_warnings,
        ),
    );

    app.add_systems(
        Update,
        (
            // UI management systems
            crate::settings::device_selection_ui::update_player_panels,
            crate::settings::device_selection_ui::setup_device_section_content,
            crate::settings::device_selection_ui::setup_device_buttons,
            crate::settings::device_selection_ui::handle_device_button_clicks,
            crate::settings::device_selection_ui::update_device_button_appearance,
            crate::settings::device_selection_ui::update_current_device_display,
            crate::settings::device_selection_ui::handle_scroll_input,
            back_to_settings,
        )
            .chain()
            .run_if(in_state(Menu::DeviceSelection)),
    );
}

fn back_to_settings(keyboard: Res<ButtonInput<KeyCode>>, mut next_menu: ResMut<NextState<Menu>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_menu.set(Menu::Settings);
    }
}
