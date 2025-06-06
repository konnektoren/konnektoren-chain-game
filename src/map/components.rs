use bevy::prelude::*;

/// Resource representing the game grid map
#[derive(Resource, Component, Reflect, Clone)]
#[reflect(Resource)]
pub struct GridMap {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub cells: Vec<Vec<GridCell>>,
}

impl Default for GridMap {
    fn default() -> Self {
        let width = super::GRID_SIZE;
        let height = super::GRID_SIZE;
        let cell_size = super::CELL_SIZE;

        let mut cells = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                row.push(GridCell::new(x, y));
            }
            cells.push(row);
        }

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }
}

impl GridMap {
    pub fn new(width: usize, height: usize, cell_size: f32) -> Self {
        let mut cells = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                row.push(GridCell::new(x, y));
            }
            cells.push(row);
        }

        Self {
            width,
            height,
            cell_size,
            cells,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&GridCell> {
        self.cells.get(y)?.get(x)
    }

    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut GridCell> {
        self.cells.get_mut(y)?.get_mut(x)
    }

    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<(usize, usize)> {
        let half_width = (self.width as f32 * self.cell_size) / 2.0;
        let half_height = (self.height as f32 * self.cell_size) / 2.0;

        let adjusted_x = world_pos.x + half_width;
        let adjusted_y = world_pos.y + half_height;

        if adjusted_x < 0.0 || adjusted_y < 0.0 {
            return None;
        }

        let grid_x = (adjusted_x / self.cell_size) as usize;
        let grid_y = (adjusted_y / self.cell_size) as usize;

        if grid_x >= self.width || grid_y >= self.height {
            None
        } else {
            Some((grid_x, grid_y))
        }
    }

    pub fn grid_to_world(&self, grid_x: usize, grid_y: usize) -> Vec2 {
        let half_width = (self.width as f32 * self.cell_size) / 2.0;
        let half_height = (self.height as f32 * self.cell_size) / 2.0;

        Vec2::new(
            (grid_x as f32 * self.cell_size) - half_width + (self.cell_size / 2.0),
            (grid_y as f32 * self.cell_size) - half_height + (self.cell_size / 2.0),
        )
    }
}

/// Represents a single cell in the grid
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct GridCell {
    pub x: usize,
    pub y: usize,
    pub cell_type: GridCellType,
    pub is_occupied: bool,
    pub particle_intensity: f32, // For future particle effects
}

impl GridCell {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            cell_type: GridCellType::Empty,
            is_occupied: false,
            particle_intensity: 0.0,
        }
    }
}

/// Types of grid cells for future expansion
#[derive(Reflect, Clone, Debug, PartialEq)]
pub enum GridCellType {
    Empty,
    Wall,
    QuestionZone,
    AnswerZone,
    ParticleSource, // For chain reaction effects
}

/// Component for entities that have a position on the grid
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
}

impl GridPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Marker component for the grid visualization entity
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GridVisualization;
