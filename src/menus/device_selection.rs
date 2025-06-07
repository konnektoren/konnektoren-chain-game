use crate::{menus::Menu, settings::*};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DeviceSelectionState>();
    app.register_type::<PlayerConfigPanel>();
    app.register_type::<DeviceSelectionArea>();
    app.register_type::<crate::settings::device_selection_ui::PlayerActionButtons>();
    app.register_type::<crate::settings::device_selection_ui::PlayerToggleButton>();
    app.register_type::<crate::settings::device_selection_ui::PlayerConfigureButton>();

    app.init_resource::<DeviceSelectionState>();

    app.add_systems(
        OnEnter(Menu::DeviceSelection),
        crate::settings::device_selection_ui::spawn_device_selection_ui,
    );

    app.add_systems(
        Update,
        (
            crate::settings::device_selection_ui::update_player_panels,
            crate::settings::device_selection_ui::setup_action_buttons,
            crate::settings::device_selection_ui::handle_button_actions,
            crate::settings::device_selection_ui::handle_panel_interactions,
            crate::settings::device_selection_ui::handle_device_selection_input,
            crate::settings::device_selection_ui::handle_selection_timeout,
            back_to_settings,
        )
            .chain() // Run in order to ensure proper entity lifecycle
            .run_if(in_state(Menu::DeviceSelection)),
    );
}

fn back_to_settings(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_menu: ResMut<NextState<Menu>>,
    selection_state: Res<DeviceSelectionState>,
) {
    // Only allow going back if not currently selecting
    if selection_state.selecting_player.is_none() && keyboard.just_pressed(KeyCode::Escape) {
        next_menu.set(Menu::Settings);
    }
}
