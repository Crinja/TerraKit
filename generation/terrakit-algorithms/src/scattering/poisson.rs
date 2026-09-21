// Deterministic Poisson-disc object scattering.
// Poisson-disc sampling to dprevent objects from appearing too close together.

use super::random::{
    random_y_rotation,
    ScatterRng,
};

use terrakit_core::ScatterPoint;

use super::types::{
    ScatterArea,
    ScatterError,
    ScatterSettings,
};

// Configuration for Poisson-disc scattering.
#[derive(Debug, Clone, Copy)]
pub struct PoissonSettings {   
    pub seed: u64, // Random seed.   
    pub radius: f32, // Minimum distance between objects.  
    pub attempts: u32, // Number of candidate attempts for each active point.  
    pub prototype_id: u32, // Object prototype ID. 
    pub min_scale: f32, // Minimum scale.
    pub max_scale: f32, // Maximum scale.
    pub random_rotation: bool, // Whether to randomly rotate objects.
}

impl Default for PoissonSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            radius: 5.0,
            attempts: 30,
            prototype_id: 0,
            min_scale: 1.0,
            max_scale: 1.0,
            random_rotation: true,
        }
    }
}

impl PoissonSettings {
    // Validates the settings.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.radius.is_finite() || self.radius <= 0.0 {
            return Err(ScatterError::InvalidRadius);
        }

        if self.attempts == 0 {
            return Err(ScatterError::InvalidAttempts);
        }

        if !self.min_scale.is_finite()
            || !self.max_scale.is_finite()
            || self.min_scale < 0.0
            || self.max_scale < self.min_scale
        {
            return Err(ScatterError::InvalidSettings(
                "invalid scale range",
            ));
        }

        Ok(())
    }
}

// Generates Poisson-disc distributed points.

// Every generated point attempts to maintain at least `radius` distance from all existing points.
pub fn scatter_poisson(
    settings: PoissonSettings,
    area: ScatterArea,
) -> Result<Vec<ScatterPoint>, ScatterError> {
    settings.validate()?;
    area.validate()?;

    let mut rng = ScatterRng::new(settings.seed);

    let cell_size = settings.radius / 2.0_f32.sqrt();

    let grid_width =
        (area.width() / cell_size).ceil() as usize + 1;

    let grid_height =
        (area.depth() / cell_size).ceil() as usize + 1;

    let mut grid: Vec<Option<usize>> =
        vec![None; grid_width * grid_height];

    let mut points: Vec<[f32; 2]> = Vec::new();
    let mut active: Vec<usize> = Vec::new();

    // Initial point.
    let initial_x =
        rng.range_f32(area.min_x, area.max_x);

    let initial_z =
        rng.range_f32(area.min_z, area.max_z);

    let initial = [initial_x, initial_z];

    points.push(initial);
    active.push(0);

    let gx = grid_x(
        initial[0],
        &area,
        cell_size,
    );

    let gz = grid_z(
        initial[1],
        &area,
        cell_size,
    );

    grid[gz * grid_width + gx] = Some(0);

    while !active.is_empty() {
        let active_position =
            rng.range_u32(0, active.len() as u32) as usize;

        let point_index = active[active_position];

        let origin = points[point_index];

        let mut found = false;

        for _ in 0..settings.attempts {
            let angle =
                rng.next_f32() * std::f32::consts::TAU;

            let distance =
                settings.radius
                    * (1.0 + rng.next_f32());

            let candidate = [
                origin[0] + angle.cos() * distance,
                origin[1] + angle.sin() * distance,
            ];

            if !inside_area(candidate, &area) {
                continue;
            }

            if !is_valid_candidate(
                candidate,
                &points,
                &grid,
                grid_width,
                cell_size,
                &area,
                settings.radius,
            ) {
                continue;
            }

            let new_index = points.len();

            points.push(candidate);
            active.push(new_index);

            let gx = grid_x(
                candidate[0],
                &area,
                cell_size,
            );

            let gz = grid_z(
                candidate[1],
                &area,
                cell_size,
            );

            grid[gz * grid_width + gx] =
                Some(new_index);

            found = true;

            break;
        }

        if !found {
            active.swap_remove(active_position);
        }
    }

    let mut result =
        Vec::with_capacity(points.len());

    for point in points {
        let scale = rng.range_f32(
            settings.min_scale,
            settings.max_scale,
        );

        let rotation =
            if settings.random_rotation {
                random_y_rotation(&mut rng)
            } else {
                [0.0, 0.0, 0.0, 1.0]
            };

        result.push(ScatterPoint {
            position: [
                point[0],
                0.0,
                point[1],
            ],
            rotation,
            scale: [
                scale,
                scale,
                scale,
            ],
            prototype_id: settings.prototype_id,
        });
    }

    Ok(result)
}

// Converts world X into a grid coordinate.
fn grid_x(
    x: f32,
    area: &ScatterArea,
    cell_size: f32,
) -> usize {
    ((x - area.min_x) / cell_size).floor() as usize
}

// Converts world Z into a grid coordinate.
fn grid_z(
    z: f32,
    area: &ScatterArea,
    cell_size: f32,
) -> usize {
    ((z - area.min_z) / cell_size).floor() as usize
}

// Checks whether a point is inside the scattering area.
fn inside_area(
    point: [f32; 2],
    area: &ScatterArea,
) -> bool {
    point[0] >= area.min_x
        && point[0] < area.max_x
        && point[1] >= area.min_z
        && point[1] < area.max_z
}

// Checks whether a candidate is sufficiently far from all nearby points.
fn is_valid_candidate(
    candidate: [f32; 2],
    points: &[[f32; 2]],
    grid: &[Option<usize>],
    grid_width: usize,
    cell_size: f32,
    area: &ScatterArea,
    radius: f32,
) -> bool {
    let gx = grid_x(
        candidate[0],
        area,
        cell_size,
    );

    let gz = grid_z(
        candidate[1],
        area,
        cell_size,
    );

    let radius_squared = radius * radius;

    let min_x =
        gx.saturating_sub(2);

    let max_x =
        (gx + 2).min(grid_width - 1);

    let grid_height =
        grid.len() / grid_width;

    let min_z =
        gz.saturating_sub(2);

    let max_z =
        (gz + 2).min(grid_height - 1);

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let index =
                z * grid_width + x;

            if let Some(point_index) = grid[index] {
                let point = points[point_index];

                let dx =
                    candidate[0] - point[0];

                let dz =
                    candidate[1] - point[1];

                let distance_squared =
                    dx * dx + dz * dz;

                if distance_squared < radius_squared {
                    return false;
                }
            }
        }
    }

    true
}