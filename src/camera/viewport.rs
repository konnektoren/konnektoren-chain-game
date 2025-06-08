use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct ViewportCalculator {
    margin: f32,
}

#[derive(Debug, Clone)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl ViewportCalculator {
    pub fn new(margin: f32) -> Self {
        Self { margin }
    }

    /// Calculate bounds that encompass all object positions with margin
    pub fn calculate_bounds(&self, positions: &[Vec2]) -> Option<Bounds> {
        if positions.is_empty() {
            return None;
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        // Find the actual bounds of all objects
        for pos in positions {
            min_x = min_x.min(pos.x);
            max_x = max_x.max(pos.x);
            min_y = min_y.min(pos.y);
            max_y = max_y.max(pos.y);
        }

        // Add margin to the bounds
        Some(Bounds {
            min: Vec2::new(min_x - self.margin, min_y - self.margin),
            max: Vec2::new(max_x + self.margin, max_y + self.margin),
        })
    }

    /// Calculate camera position and scale to fit the bounds
    pub fn calculate_camera_settings(
        &self,
        bounds: &Bounds,
        desired_viewport: Vec2,
    ) -> (Vec2, f32) {
        let center = (bounds.min + bounds.max) * 0.5;
        let size = bounds.max - bounds.min;

        // Calculate scale needed to fit both width and height
        let scale_x = desired_viewport.x / size.x;
        let scale_y = desired_viewport.y / size.y;

        // Use the smaller scale to ensure everything fits
        let scale = scale_x.min(scale_y);

        (center, scale)
    }

    /// Convenience method to calculate from Transform components
    pub fn calculate_from_transforms(
        &self,
        transforms: &[&Transform],
        desired_viewport: Vec2,
    ) -> Option<(Vec2, f32)> {
        let positions: Vec<Vec2> = transforms
            .iter()
            .map(|t| Vec2::new(t.translation.x, t.translation.y))
            .collect();

        self.calculate_bounds(&positions)
            .map(|bounds| self.calculate_camera_settings(&bounds, desired_viewport))
    }
}

impl Bounds {
    #[allow(dead_code)]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[allow(dead_code)]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[allow(dead_code)]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
}
