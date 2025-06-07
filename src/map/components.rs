use bevy::prelude::*;

/// Resource for configuring map properties
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub background_color: Color,
    pub grid_color: Color,
    pub show_grid_lines: bool,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width: super::DEFAULT_GRID_WIDTH,
            height: super::DEFAULT_GRID_HEIGHT,
            cell_size: super::DEFAULT_CELL_SIZE,
            background_color: super::BACKGROUND_COLOR,
            grid_color: super::GRID_COLOR,
            show_grid_lines: true,
        }
    }
}

impl MapConfig {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    pub fn with_cell_size(mut self, cell_size: f32) -> Self {
        self.cell_size = cell_size;
        self
    }

    pub fn with_colors(mut self, background: Color, grid: Color) -> Self {
        self.background_color = background;
        self.grid_color = grid;
        self
    }
}

/// Resource representing the game grid map
#[derive(Resource, Component, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct GridMap {
    pub width: usize,
    pub height: usize,
    pub cell_size: f32,
    pub cells: Vec<Vec<GridCell>>,
}

impl GridMap {
    pub fn from_config(config: &MapConfig) -> Self {
        let mut cells = Vec::with_capacity(config.height);
        for y in 0..config.height {
            let mut row = Vec::with_capacity(config.width);
            for x in 0..config.width {
                row.push(GridCell::new(x, y));
            }
            cells.push(row);
        }

        Self {
            width: config.width,
            height: config.height,
            cell_size: config.cell_size,
            cells,
        }
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

    pub fn world_width(&self) -> f32 {
        self.width as f32 * self.cell_size
    }

    pub fn world_height(&self) -> f32 {
        self.height as f32 * self.cell_size
    }

    pub fn half_width(&self) -> f32 {
        self.world_width() / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.world_height() / 2.0
    }
}

impl Default for GridMap {
    fn default() -> Self {
        Self::from_config(&MapConfig::default())
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
