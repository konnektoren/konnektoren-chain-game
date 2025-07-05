use super::components::*;
use bevy::prelude::*;

#[cfg(feature = "particles")]
use bevy_hanabi::prelude::*;

/// System to set up particle effects when entering gameplay
pub fn setup_particle_effects(mut commands: Commands) {
    commands.insert_resource(ParticleEffects::default());
    info!("Particle effects initialized");
}

/// System to handle explosion events
pub fn handle_explosion_events(
    mut commands: Commands,
    mut explosion_events: EventReader<SpawnExplosionEvent>,
    #[cfg(feature = "particles")] mut effects: ResMut<Assets<EffectAsset>>,
) {
    for event in explosion_events.read() {
        #[cfg(feature = "particles")]
        {
            // Create a custom effect with the ball's color
            let explosion_effect = create_colored_explosion_effect(&mut effects, event.color);
            commands.spawn((
                Name::new("Chain Explosion Effect"),
                ChainExplosionEffect::new(2.0, event.intensity),
                ParticleEffect::new(explosion_effect),
                Transform::from_translation(event.position),
                StateScoped(crate::screens::Screen::Gameplay),
            ));
        }

        #[cfg(not(feature = "particles"))]
        {
            commands.spawn((
                Name::new("Chain Explosion Effect"),
                ChainExplosionEffect::new(2.0, event.intensity),
                Transform::from_translation(event.position),
                StateScoped(crate::screens::Screen::Gameplay),
            ));
        }

        info!(
            "Spawned explosion effect at {:?} with color {:?}",
            event.position, event.color
        );
    }
}

/// System to handle collection events
pub fn handle_collection_events(
    mut commands: Commands,
    mut collection_events: EventReader<SpawnCollectionEvent>,
    #[cfg(feature = "particles")] mut effects: ResMut<Assets<EffectAsset>>,
) {
    for event in collection_events.read() {
        #[cfg(feature = "particles")]
        {
            // Use the existing create_colored_collection_effect function
            let collection_effect = create_colored_collection_effect(&mut effects, event.color);
            commands.spawn((
                Name::new("Collection Effect"),
                CollectionEffect::new(1.0),
                ParticleEffect::new(collection_effect),
                Transform::from_translation(event.position),
                StateScoped(crate::screens::Screen::Gameplay),
            ));
        }

        #[cfg(not(feature = "particles"))]
        {
            commands.spawn((
                Name::new("Collection Effect"),
                CollectionEffect::new(1.0),
                Transform::from_translation(event.position),
                StateScoped(crate::screens::Screen::Gameplay),
            ));
        }
    }
}

#[cfg(feature = "particles")]
/// Create a collection effect with a specific color
fn create_colored_collection_effect(
    effects: &mut Assets<EffectAsset>,
    color: Color,
) -> Handle<EffectAsset> {
    // Convert Bevy Color to Vec4 properly
    let linear_color = color.to_linear();
    let base_color = Vec4::new(
        linear_color.red,
        linear_color.green,
        linear_color.blue,
        linear_color.alpha,
    );
    let bright_color = base_color * 2.5; // Make it bright
    let mid_color = base_color * 1.8;
    let fade_color = base_color * 1.2;

    // Create a gradient based on the ball's color
    let mut gradient = Gradient::new();
    gradient.add_key(
        0.0,
        Vec4::new(bright_color.x, bright_color.y, bright_color.z, 1.0),
    );
    gradient.add_key(0.7, Vec4::new(mid_color.x, mid_color.y, mid_color.z, 0.8));
    gradient.add_key(
        1.0,
        Vec4::new(fade_color.x, fade_color.y, fade_color.z, 0.0),
    );

    let writer = ExprWriter::new();

    // Initialize position in a small sphere
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(1.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Initialize velocity upward with some spread
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(25.0).expr(),
    };

    // Set lifetime
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.0).expr());

    // Set age
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());

    // Add upward acceleration
    let upward_accel = AccelModifier::new(writer.lit(Vec3::new(0.0, 30.0, 0.0)).expr());

    let effect = EffectAsset::new(
        32,
        SpawnerSettings::burst(16.0.into(), 0.05.into()),
        writer.finish(),
    )
    .with_name(format!("colored_collection_{:?}", color))
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(upward_accel)
    .render(ColorOverLifetimeModifier {
        gradient,
        blend: ColorBlendMode::Overwrite,
        mask: ColorBlendMask::RGBA,
    });

    effects.add(effect)
}

#[cfg(feature = "particles")]
/// Create a collection effect with a specific color
fn create_colored_explosion_effect(
    effects: &mut Assets<EffectAsset>,
    color: Color,
) -> Handle<EffectAsset> {
    // Convert Bevy Color to Vec4 properly
    let linear_color = color.to_linear();
    let base_color = Vec4::new(
        linear_color.red,
        linear_color.green,
        linear_color.blue,
        linear_color.alpha,
    );
    let bright_color = base_color * 3.0; // Make it bright for HDR
    let mid_color = base_color * 1.5;
    let fade_color = base_color * 0.3;

    // Create a gradient based on the ball's color
    let mut gradient = Gradient::new();
    gradient.add_key(
        0.0,
        Vec4::new(bright_color.x, bright_color.y, bright_color.z, 1.0),
    );
    gradient.add_key(0.5, Vec4::new(mid_color.x, mid_color.y, mid_color.z, 0.8));
    gradient.add_key(
        1.0,
        Vec4::new(fade_color.x, fade_color.y, fade_color.z, 0.0),
    );

    let writer = ExprWriter::new();

    // Initialize position around a sphere for 3D explosion
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.0).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Initialize velocity outward from center
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(50.0).expr(),
    };

    // Set lifetime
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.5).expr());

    // Set age
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());

    // Gravity effect
    let gravity = AccelModifier::new(writer.lit(Vec3::new(0.0, -50.0, 0.0)).expr());

    // Linear drag
    let drag = LinearDragModifier::new(writer.lit(2.0).expr());

    let effect = EffectAsset::new(
        64,
        SpawnerSettings::burst(32.0.into(), 0.1.into()),
        writer.finish(),
    )
    .with_name(format!("colored_explosion_{:?}", color))
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(gravity)
    .update(drag)
    .render(ColorOverLifetimeModifier {
        gradient,
        blend: ColorBlendMode::Add,
        mask: ColorBlendMask::RGBA,
    });

    effects.add(effect)
}

/// System to cleanup finished effects
pub fn cleanup_finished_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut explosion_query: Query<(Entity, &mut ChainExplosionEffect)>,
    mut collection_query: Query<(Entity, &mut CollectionEffect)>,
) {
    // Cleanup explosion effects
    for (entity, mut effect) in &mut explosion_query {
        effect.lifetime.tick(time.delta());
        if effect.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }

    // Cleanup collection effects
    for (entity, mut effect) in &mut collection_query {
        effect.lifetime.tick(time.delta());
        if effect.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
