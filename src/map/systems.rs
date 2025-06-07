use super::components::*;
use crate::screens::Screen;
use bevy::prelude::*;

/// System to set up the grid map from configuration
pub fn setup_grid_map(
    mut commands: Commands,
    map_config: Res<MapConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create grid map from configuration
    let grid_map = GridMap::from_config(&map_config);

    info!(
        "Setting up grid map: {}x{} cells ({}x{} world units)",
        grid_map.width,
        grid_map.height,
        grid_map.world_width(),
        grid_map.world_height()
    );

    // Spawn the visual grid background
    spawn_grid_background(
        &mut commands,
        &grid_map,
        &map_config,
        &mut meshes,
        &mut materials,
    );

    // Insert the grid map as a resource
    commands.insert_resource(grid_map);
}

/// Spawn the visual representation of the grid
fn spawn_grid_background(
    commands: &mut Commands,
    grid_map: &GridMap,
    map_config: &MapConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let total_width = grid_map.world_width();
    let total_height = grid_map.world_height();

    // Create background quad
    let background_mesh = meshes.add(Rectangle::new(total_width, total_height));
    let background_material = materials.add(ColorMaterial::from(map_config.background_color));

    commands.spawn((
        Name::new("Grid Background"),
        Mesh2d(background_mesh),
        MeshMaterial2d(background_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        GridVisualization,
        StateScoped(Screen::Gameplay),
    ));

    // Only create grid lines if enabled
    if map_config.show_grid_lines {
        let grid_mesh = create_grid_mesh(grid_map, meshes);
        let grid_material = materials.add(ColorMaterial::from(map_config.grid_color));

        commands.spawn((
            Name::new("Grid Lines"),
            Mesh2d(grid_mesh),
            MeshMaterial2d(grid_material),
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            GridVisualization,
            StateScoped(Screen::Gameplay),
        ));
    }
}

/// Create a mesh for the grid lines
fn create_grid_mesh(grid_map: &GridMap, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let half_width = grid_map.half_width();
    let half_height = grid_map.half_height();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut index = 0u32;

    // Vertical lines
    for i in 0..=grid_map.width {
        let x = (i as f32 * grid_map.cell_size) - half_width;
        vertices.push([x, -half_height, 0.0]);
        vertices.push([x, half_height, 0.0]);
        indices.push(index);
        indices.push(index + 1);
        index += 2;
    }

    // Horizontal lines
    for i in 0..=grid_map.height {
        let y = (i as f32 * grid_map.cell_size) - half_height;
        vertices.push([-half_width, y, 0.0]);
        vertices.push([half_width, y, 0.0]);
        indices.push(index);
        indices.push(index + 1);
        index += 2;
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    meshes.add(mesh)
}

/// System to handle map configuration changes at runtime
pub fn handle_map_config_changes(
    mut commands: Commands,
    map_config: Res<MapConfig>,
    mut grid_map: ResMut<GridMap>,
    grid_entities: Query<Entity, With<GridVisualization>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if map_config.is_changed() {
        info!("Map configuration changed, rebuilding grid...");

        // Remove old grid visualization
        for entity in &grid_entities {
            commands.entity(entity).despawn();
        }

        // Create new grid map
        *grid_map = GridMap::from_config(&map_config);

        // Spawn new visualization
        spawn_grid_background(
            &mut commands,
            &grid_map,
            &map_config,
            &mut meshes,
            &mut materials,
        );

        info!(
            "Grid rebuilt: {}x{} cells ({}x{} world units)",
            grid_map.width,
            grid_map.height,
            grid_map.world_width(),
            grid_map.world_height()
        );
    }
}

/// System to update grid visualization based on changes
pub fn update_grid_visualization(
    grid_map: Res<GridMap>,
    mut grid_query: Query<&mut Transform, With<GridVisualization>>,
) {
    // This system can be expanded later to handle dynamic grid changes
    // For now, it's a placeholder for future particle effects and cell state changes
    if grid_map.is_changed() {
        // Update visualization if needed
        for mut _transform in &mut grid_query {
            // Future: Update particle effects, cell colors, etc.
        }
    }
}
