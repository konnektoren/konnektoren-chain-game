use bevy::prelude::*;

mod components;
mod systems;

pub use components::*;
use systems::*;

pub(super) fn plugin(app: &mut App) {
    #[cfg(feature = "particles")]
    app.add_plugins(bevy_hanabi::HanabiPlugin);

    app.register_type::<ChainExplosionEffect>();
    app.register_type::<CollectionEffect>();

    app.add_event::<SpawnExplosionEvent>();
    app.add_event::<SpawnCollectionEvent>();

    app.add_systems(
        OnEnter(crate::screens::Screen::Gameplay),
        setup_particle_effects,
    );

    app.add_systems(
        Update,
        (
            handle_explosion_events.in_set(crate::AppSystems::Update),
            handle_collection_events.in_set(crate::AppSystems::Update),
            cleanup_finished_effects.in_set(crate::AppSystems::Update),
        )
            .run_if(in_state(crate::screens::Screen::Gameplay))
            .in_set(crate::PausableSystems),
    );
}
