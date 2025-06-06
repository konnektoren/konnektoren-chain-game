use super::components::*;
use super::{BACKGROUND_COLOR, GRID_COLOR, GRID_LINE_WIDTH};
use crate::screens::Screen;
use bevy::prelude::*;

/// System to set up the grid map resource and spawn the visual grid
pub fn setup_grid_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Initialize the grid map resource
    let grid_map = GridMap::default();

    info!(
        "Setting up grid map: {}x{} cells",
        grid_map.width, grid_map.height
    );

    // Spawn the visual grid background
    spawn_grid_background(&mut commands, &grid_map, &mut meshes, &mut materials);

    // Insert the grid map as a resource
    commands.insert_resource(grid_map);
}

/// Spawn the visual representation of the grid
fn spawn_grid_background(
    commands: &mut Commands,
    grid_map: &GridMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    let total_width = grid_map.width as f32 * grid_map.cell_size;
    let total_height = grid_map.height as f32 * grid_map.cell_size;

    // Create background quad
    let background_mesh = meshes.add(Rectangle::new(total_width, total_height));
    let background_material = materials.add(ColorMaterial::from(BACKGROUND_COLOR));

    // Create grid line mesh
    let grid_mesh = create_grid_mesh(grid_map, meshes);
    let grid_material = materials.add(ColorMaterial::from(GRID_COLOR));

    commands.spawn((
        Name::new("Grid Background"),
        Mesh2d(background_mesh),
        MeshMaterial2d(background_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)),
        GridVisualization,
        StateScoped(Screen::Gameplay),
    ));

    commands.spawn((
        Name::new("Grid Lines"),
        Mesh2d(grid_mesh),
        MeshMaterial2d(grid_material),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GridVisualization,
        StateScoped(Screen::Gameplay),
    ));
}

/// Create a mesh for the grid lines
fn create_grid_mesh(grid_map: &GridMap, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::LineList,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );

    let total_width = grid_map.width as f32 * grid_map.cell_size;
    let total_height = grid_map.height as f32 * grid_map.cell_size;
    let half_width = total_width / 2.0;
    let half_height = total_height / 2.0;

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

/// Helper function to convert world position to grid coordinates
pub fn world_to_grid_position(world_pos: Vec2, grid_map: &GridMap) -> Option<GridPosition> {
    grid_map
        .world_to_grid(world_pos)
        .map(|(x, y)| GridPosition::new(x, y))
}

/// Helper function to convert grid position to world coordinates
pub fn grid_to_world_position(grid_pos: &GridPosition, grid_map: &GridMap) -> Vec2 {
    grid_map.grid_to_world(grid_pos.x, grid_pos.y)
}

/// System to snap entities with GridPosition to the grid
pub fn snap_to_grid(
    grid_map: Res<GridMap>,
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_pos, mut transform) in &mut query {
        let world_pos = grid_to_world_position(grid_pos, &grid_map);
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}
