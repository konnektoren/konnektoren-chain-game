//! The credits menu.

use bevy::{
    ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};

use crate::{asset_tracking::LoadResource, audio::music, menus::Menu, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
    app.add_systems(
        Update,
        (
            go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
            handle_scroll_input.run_if(in_state(Menu::Credits)),
        ),
    );

    app.register_type::<CreditsAssets>();
    app.load_resource::<CreditsAssets>();
    app.add_systems(OnEnter(Menu::Credits), start_credits_music);
}

fn spawn_credits_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Credits Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Credits),
        children![
            // Scrollable content area
            (
                Name::new("Scrollable Content"),
                Node {
                    width: Percent(100.0),
                    height: Percent(85.0), // Leave space for fixed button
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Px(20.0)),
                    row_gap: Px(30.0),
                    ..default()
                },
                ScrollPosition::default(),
                CreditsScrollArea,
                children![
                    widget::header("Created by"),
                    created_by(),
                    widget::header("Assets"),
                    assets(),
                ],
            ),
            (
                Name::new("Fixed Button Container"),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Px(20.0),
                    left: Percent(50.0),
                    width: Px(450.0),
                    height: Px(80.0),
                    margin: UiRect::left(Px(-225.0)), // Center the button
                    flex_shrink: 0.0,
                    ..default()
                },
                children![widget::button("Back", go_back_on_click),],
            ),
        ],
    ));
}

#[derive(Component)]
struct CreditsScrollArea;

fn handle_scroll_input(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut scrolled_node_query: Query<&mut ScrollPosition, With<CreditsScrollArea>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    const LINE_HEIGHT: f32 = 30.0;

    // Handle mouse wheel scrolling
    for mouse_wheel_event in mouse_wheel_events.read() {
        let dy = match mouse_wheel_event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            bevy::input::mouse::MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        for mut scroll_position in scrolled_node_query.iter_mut() {
            scroll_position.offset_y -= dy;
            scroll_position.offset_y = scroll_position.offset_y.max(0.0);
        }
    }

    // Handle keyboard scrolling
    for mut scroll_position in scrolled_node_query.iter_mut() {
        let mut scroll_delta = 0.0;

        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            scroll_delta += LINE_HEIGHT * 0.5;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            scroll_delta -= LINE_HEIGHT * 0.5;
        }
        if keyboard_input.pressed(KeyCode::PageUp) {
            scroll_delta += LINE_HEIGHT * 5.0;
        }
        if keyboard_input.pressed(KeyCode::PageDown) {
            scroll_delta -= LINE_HEIGHT * 5.0;
        }
        if keyboard_input.pressed(KeyCode::Home) {
            scroll_position.offset_y = 0.0;
            continue;
        }

        if scroll_delta != 0.0 {
            scroll_position.offset_y -= scroll_delta;
            scroll_position.offset_y = scroll_position.offset_y.max(0.0);
        }
    }
}

fn created_by() -> impl Bundle {
    grid(vec![
        ["chriamue", "Implemented game, assisted by Claude"],
        ["Joe Shmoe", "Implemented alligator wrestling AI"],
        ["Jane Doe", "Made the music for the alien invasion"],
    ])
}

fn assets() -> impl Bundle {
    grid(vec![
        ["Ducky sprite", "CC0 by Caz Creates Games"],
        ["Button SFX", "CC0 by Jaszunio15"],
        ["Music", "CC BY 3.0 by Kevin MacLeod"],
        ["Gameplay SFX", "Licensed from Ovani Sound"],
        [
            "Audio License",
            "https://ovanisound.com/policies/terms-of-service",
        ],
        [
            "Bevy logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            max_width: Px(850.0),
            width: Percent(100.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    widget::label(text),
                    Node {
                        justify_self: if i % 2 == 0 {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn go_back_on_click(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct CreditsAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for CreditsAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Monkeys Spinning Monkeys.ogg"),
        }
    }
}

fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        StateScoped(Menu::Credits),
        music(credits_music.music.clone()),
    ));
}
