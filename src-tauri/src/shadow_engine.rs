use crate::sun_position::SunCalculator;
use crate::types::*;
use chrono::Timelike;
use indicatif::{ProgressBar, ProgressStyle};
use ndarray::{s, Array2, Array3};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub progress: f64,
    pub current_step: String,
    pub total_steps: Option<usize>,
    pub current_step_number: Option<usize>,
}

pub struct ShadowEngine {
    _dtm: Array2<f32>,
    dsm: Array2<f32>,
    heights: Array2<f32>,
    resolution: f64,
    transform: [f64; 6],
    aoi_polygon: geo_types::Polygon<f64>,
    sun_calculator: Arc<Mutex<SunCalculator>>,
    config: Config,
    app_handle: Option<AppHandle>,
    optimization_logged: Arc<Mutex<bool>>,
}

impl ShadowEngine {
    #[allow(dead_code)]
    pub fn new(
        dtm: Array2<f32>,
        dsm: Array2<f32>,
        resolution: f64,
        transform: [f64; 6],
        config: Config,
    ) -> Self {
        let heights = &dsm - &dtm;
        let polygon = config.to_polygon().unwrap_or(geo_types::Polygon::new(
            geo_types::LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        ));
        let centroid = geo::algorithm::centroid::Centroid::centroid(&polygon)
            .unwrap_or(geo_types::Point::new(0.0, 0.0));
        let sun_calculator = Arc::new(Mutex::new(SunCalculator::new(
            centroid.y(),
            centroid.x(),
            config.angle_precision,
        )));

        Self {
            _dtm: dtm,
            dsm,
            heights,
            resolution,
            transform,
            aoi_polygon: polygon,
            sun_calculator,
            config,
            app_handle: None,
            optimization_logged: Arc::new(Mutex::new(false)),
        }
    }

    pub fn new_with_app_handle(
        dtm: Array2<f32>,
        dsm: Array2<f32>,
        resolution: f64,
        transform: [f64; 6],
        config: Config,
        app_handle: AppHandle,
    ) -> Self {
        // Configure Rayon thread pool with specified CPU cores
        let cpu_cores = config.get_cpu_cores();
        println!("Setting up Rayon thread pool with {} cores", cpu_cores);

        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(cpu_cores)
            .build_global()
        {
            eprintln!("Warning: Failed to configure Rayon thread pool: {}", e);
            println!("Using default thread pool configuration");
        }

        let heights = &dsm - &dtm;
        let polygon = config.to_polygon().unwrap_or(geo_types::Polygon::new(
            geo_types::LineString::from(vec![
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.0),
            ]),
            vec![],
        ));
        let centroid = geo::algorithm::centroid::Centroid::centroid(&polygon)
            .unwrap_or(geo_types::Point::new(0.0, 0.0));
        let sun_calculator = Arc::new(Mutex::new(SunCalculator::new(
            centroid.y(),
            centroid.x(),
            config.angle_precision,
        )));

        Self {
            _dtm: dtm,
            dsm,
            heights,
            resolution,
            transform,
            aoi_polygon: polygon,
            sun_calculator,
            config,
            app_handle: Some(app_handle),
            optimization_logged: Arc::new(Mutex::new(false)),
        }
    }

    fn emit_progress(
        &self,
        progress: f64,
        step: String,
        total_steps: Option<usize>,
        current_step: Option<usize>,
    ) {
        if let Some(app) = &self.app_handle {
            let update = ProgressUpdate {
                progress,
                current_step: step,
                total_steps,
                current_step_number: current_step,
            };
            let _ = app.emit_all("progress-update", &update);
        }
    }

    pub fn calculate_shadows(&self) -> Result<ShadowResult, ShadowError> {
        let timestamps = self.generate_timestamps();
        let n_times = timestamps.len();
        let (n_rows, n_cols) = self.heights.dim();

        let mut shadow_fraction = Array3::<f32>::zeros((n_times, n_rows, n_cols));

        // Emit initial progress
        self.emit_progress(
            0.0,
            "Initializing shadow calculation...".to_string(),
            Some(n_times),
            Some(0),
        );

        // Create progress bar for console (keep for debugging)
        let pb = ProgressBar::new(n_times as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap(),
        );

        // Process timestamps sequentially to emit proper progress
        for (t_idx, timestamp) in timestamps.iter().enumerate() {
            let mut sun_calc = self.sun_calculator.lock().unwrap();
            let (azimuth, elevation) = sun_calc.get_position(timestamp);
            drop(sun_calc);

            let step_description = "Calculating shadows...".to_string();
            let progress = (t_idx as f64) / (n_times as f64) * 100.0;

            self.emit_progress(progress, step_description, Some(n_times), Some(t_idx + 1));

            let shadow_map = if elevation <= 0.0 {
                Array2::<f32>::ones((n_rows, n_cols))
            } else {
                self.calculate_shadow_map(azimuth, elevation)
            };

            shadow_fraction
                .slice_mut(s![t_idx, .., ..])
                .assign(&shadow_map);

            pb.set_position(t_idx as u64 + 1);
        }

        pb.finish_with_message("Shadow calculation complete");
        self.emit_progress(
            100.0,
            "Shadow calculation complete".to_string(),
            Some(n_times),
            Some(n_times),
        );

        let summary_stats = self.calculate_summary_stats(&shadow_fraction, &timestamps);

        Ok(ShadowResult {
            shadow_fraction,
            timestamps,
            summary_stats,
        })
    }

    fn calculate_shadow_map(&self, azimuth: f64, elevation: f64) -> Array2<f32> {
        let (n_rows, n_cols) = self.heights.dim();
        let mut shadow_map = Array2::<f32>::zeros((n_rows, n_cols));

        // Convert sun angles to ray direction
        let sun_dir = self.sun_direction(azimuth, elevation);

        // Get only cells that can potentially cast shadows on AOI
        let relevant_cells = self.get_shadow_relevant_cells(azimuth, elevation);

        // Process only relevant cells in parallel (major optimization!)
        let shadow_values: Vec<((usize, usize), f32)> = relevant_cells
            .par_iter()
            .map(|&(row, col)| ((row, col), self.calculate_cell_shadow(row, col, sun_dir)))
            .collect();

        // Assign results back to the shadow map
        for ((row, col), shadow_value) in shadow_values {
            shadow_map[[row, col]] = shadow_value;
        }

        // Apply edge refinement based on quality setting
        if !matches!(self.config.shadow_quality, ShadowQuality::Fast) {
            shadow_map = self.refine_shadow_edges(shadow_map, sun_dir);
        }

        shadow_map
    }

    fn get_shadow_relevant_cells(&self, azimuth: f64, elevation: f64) -> Vec<(usize, usize)> {
        let (n_rows, n_cols) = self.heights.dim();
        let mut relevant_cells = Vec::new();

        // Get AOI bounds in pixel coordinates
        let aoi_bounds = self.get_aoi_pixel_bounds();
        if aoi_bounds.is_none() {
            // Fallback: return all cells if we can't determine AOI bounds
            return (0..n_rows)
                .flat_map(|row| (0..n_cols).map(move |col| (row, col)))
                .collect();
        }

        let (aoi_min_row, aoi_max_row, aoi_min_col, aoi_max_col) = aoi_bounds.unwrap();

        // Calculate shadow direction (opposite of sun direction)
        let sun_azimuth_rad = azimuth.to_radians();
        let shadow_dir_x = -sun_azimuth_rad.sin();
        let shadow_dir_y = sun_azimuth_rad.cos();

        // Maximum shadow distance based on terrain and sun angle
        let max_shadow_distance_pixels =
            (self.config.buffer_meters.unwrap_or(1000.0) / self.resolution) as i32;

        // Check each cell to see if it could cast a shadow on the AOI
        for row in 0..n_rows {
            for col in 0..n_cols {
                // Skip cells with minimal height
                if self.heights[[row, col]] < 0.5 {
                    continue;
                }

                // For cells inside AOI, always include them
                if row >= aoi_min_row
                    && row <= aoi_max_row
                    && col >= aoi_min_col
                    && col <= aoi_max_col
                {
                    relevant_cells.push((row, col));
                    continue;
                }

                // For cells outside AOI, check if they could cast shadows into AOI
                if self.can_cast_shadow_into_aoi(
                    row,
                    col,
                    aoi_min_row,
                    aoi_max_row,
                    aoi_min_col,
                    aoi_max_col,
                    shadow_dir_x,
                    shadow_dir_y,
                    max_shadow_distance_pixels,
                    elevation,
                ) {
                    relevant_cells.push((row, col));
                }
            }
        }

        let total_cells = n_rows * n_cols;
        let relevant_count = relevant_cells.len();

        // Only log optimization stats once per calculation session
        let mut logged = self.optimization_logged.lock().unwrap();
        if !*logged {
            println!(
                "Shadow optimization: Processing {} of {} cells ({:.1}% reduction)",
                relevant_count,
                total_cells,
                100.0 * (1.0 - relevant_count as f64 / total_cells as f64)
            );
            *logged = true;
        }

        relevant_cells
    }

    fn get_aoi_pixel_bounds(&self) -> Option<(usize, usize, usize, usize)> {
        let (n_rows, n_cols) = self.heights.dim();

        // Get AOI bounds in world coordinates
        let coords: Vec<_> = self.aoi_polygon.exterior().coords().collect();
        let aoi_min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
        let aoi_max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max);
        let aoi_min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
        let aoi_max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max);

        // Convert to pixel coordinates
        let inv_transform = self.invert_transform(&self.transform);
        let (min_col, max_row) = self.world_to_pixel(aoi_min_x, aoi_min_y, &inv_transform);
        let (max_col, min_row) = self.world_to_pixel(aoi_max_x, aoi_max_y, &inv_transform);

        // Ensure within raster bounds
        let min_row = min_row.max(0) as usize;
        let min_col = min_col.max(0) as usize;
        let max_row = max_row.min(n_rows as i32 - 1) as usize;
        let max_col = max_col.min(n_cols as i32 - 1) as usize;

        Some((min_row, max_row, min_col, max_col))
    }

    fn can_cast_shadow_into_aoi(
        &self,
        cell_row: usize,
        cell_col: usize,
        aoi_min_row: usize,
        aoi_max_row: usize,
        aoi_min_col: usize,
        aoi_max_col: usize,
        shadow_dir_x: f64,
        shadow_dir_y: f64,
        max_distance: i32,
        elevation: f64,
    ) -> bool {
        // Quick distance check
        let dist_to_aoi = self.distance_to_aoi_bounds(
            cell_row as i32,
            cell_col as i32,
            aoi_min_row as i32,
            aoi_max_row as i32,
            aoi_min_col as i32,
            aoi_max_col as i32,
        );

        if dist_to_aoi > max_distance {
            return false;
        }

        // Direction check: is the cell on the sun side of the AOI?
        let aoi_center_row = (aoi_min_row + aoi_max_row) / 2;
        let aoi_center_col = (aoi_min_col + aoi_max_col) / 2;

        let to_aoi_x = aoi_center_col as f64 - cell_col as f64;
        let to_aoi_y = aoi_center_row as f64 - cell_row as f64;

        // Dot product to check if shadow direction aligns with direction to AOI
        let alignment = shadow_dir_x * to_aoi_x + shadow_dir_y * to_aoi_y;
        if alignment < 0.0 {
            return false; // Cell is on wrong side to cast shadow
        }

        // Height check: can this cell's height cast a shadow that far?
        let cell_height = self.heights[[cell_row, cell_col]] as f64;
        let required_height = dist_to_aoi as f64 * self.resolution * elevation.to_radians().tan();

        cell_height >= required_height * 0.5 // Allow some margin
    }

    fn distance_to_aoi_bounds(
        &self,
        row: i32,
        col: i32,
        min_row: i32,
        max_row: i32,
        min_col: i32,
        max_col: i32,
    ) -> i32 {
        let row_dist = if row < min_row {
            min_row - row
        } else if row > max_row {
            row - max_row
        } else {
            0
        };

        let col_dist = if col < min_col {
            min_col - col
        } else if col > max_col {
            col - max_col
        } else {
            0
        };

        ((row_dist * row_dist + col_dist * col_dist) as f64).sqrt() as i32
    }

    fn invert_transform(&self, transform: &[f64; 6]) -> [f64; 6] {
        let det = transform[1] * transform[5] - transform[2] * transform[4];
        [
            -transform[0] * transform[5] / det + transform[2] * transform[3] / det,
            transform[5] / det,
            -transform[2] / det,
            transform[0] * transform[4] / det - transform[1] * transform[3] / det,
            -transform[4] / det,
            transform[1] / det,
        ]
    }

    fn world_to_pixel(&self, x: f64, y: f64, inv_transform: &[f64; 6]) -> (i32, i32) {
        let col = inv_transform[0] + inv_transform[1] * x + inv_transform[2] * y;
        let row = inv_transform[3] + inv_transform[4] * x + inv_transform[5] * y;
        (col.round() as i32, row.round() as i32)
    }

    fn calculate_cell_shadow(&self, row: usize, col: usize, sun_dir: (f64, f64, f64)) -> f32 {
        let cell_height = self.dsm[[row, col]];
        let (dx, dy, dz) = sun_dir;

        // Ray marching from cell toward sun
        let mut current_x = col as f64;
        let mut current_y = row as f64;
        let mut current_z = cell_height as f64;

        let step_size = 0.5;
        let max_distance = self.config.buffer_meters.unwrap_or(1000.0) / self.resolution;
        let mut distance = 0.0;

        while distance < max_distance {
            current_x += dx * step_size;
            current_y -= dy * step_size;
            current_z += dz * step_size;
            distance += step_size;

            // Check bounds
            if current_x < 0.0
                || current_y < 0.0
                || current_x >= self.dsm.ncols() as f64 - 1.0
                || current_y >= self.dsm.nrows() as f64 - 1.0
            {
                break;
            }

            // Bilinear interpolation for terrain height
            let terrain_height = self.interpolate_height(current_y, current_x);

            // FIX: Cast current_z to f32 for comparison
            if terrain_height > current_z as f32 {
                return 1.0;
            }
        }

        0.0
    }

    fn refine_shadow_edges(
        &self,
        shadow_map: Array2<f32>,
        sun_dir: (f64, f64, f64),
    ) -> Array2<f32> {
        let (n_rows, n_cols) = shadow_map.dim();
        let mut refined = shadow_map.clone();

        // Create a vector of edge cell coordinates for parallel processing
        let edge_coords: Vec<(usize, usize)> = (1..n_rows - 1)
            .flat_map(|row| (1..n_cols - 1).map(move |col| (row, col)))
            .filter(|&(row, col)| self.is_shadow_edge(&shadow_map, row, col))
            .collect();

        let sub_samples = match self.config.shadow_quality {
            ShadowQuality::Normal => 2,
            ShadowQuality::High => 4,
            ShadowQuality::Scientific => 8,
            _ => 1,
        };

        // Process edge cells in parallel
        let refined_values: Vec<f32> = edge_coords
            .par_iter()
            .map(|&(row, col)| self.subsample_cell(row, col, sun_dir, sub_samples))
            .collect();

        // Assign refined values back to the shadow map
        for ((row, col), &refined_value) in edge_coords.iter().zip(refined_values.iter()) {
            refined[[*row, *col]] = refined_value;
        }

        refined
    }

    fn is_shadow_edge(&self, shadow_map: &Array2<f32>, row: usize, col: usize) -> bool {
        let center = shadow_map[[row, col]];
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = (row as i32 + dr) as usize;
                let c = (col as i32 + dc) as usize;
                if (shadow_map[[r, c]] - center).abs() > 0.5 {
                    return true;
                }
            }
        }
        false
    }

    fn subsample_cell(
        &self,
        row: usize,
        col: usize,
        sun_dir: (f64, f64, f64),
        samples: usize,
    ) -> f32 {
        let mut shadow_sum = 0.0;
        let step = 1.0 / samples as f64;

        for i in 0..samples {
            for j in 0..samples {
                let sub_row = row as f64 + (i as f64 + 0.5) * step;
                let sub_col = col as f64 + (j as f64 + 0.5) * step;

                shadow_sum += self.calculate_subpixel_shadow(sub_row, sub_col, sun_dir);
            }
        }

        shadow_sum / (samples * samples) as f32
    }

    fn calculate_subpixel_shadow(&self, row: f64, col: f64, sun_dir: (f64, f64, f64)) -> f32 {
        let cell_height = self.interpolate_height(row, col);
        let (dx, dy, dz) = sun_dir;

        let mut current_x = col;
        let mut current_y = row;
        let mut current_z = cell_height as f64;

        let step_size = 0.25;
        let max_distance = self.config.buffer_meters.unwrap_or(1000.0) / self.resolution;
        let mut distance = 0.0;

        while distance < max_distance {
            current_x += dx * step_size;
            current_y -= dy * step_size;
            current_z += dz * step_size;
            distance += step_size;

            if current_x < 0.0
                || current_y < 0.0
                || current_x >= self.dsm.ncols() as f64 - 1.0
                || current_y >= self.dsm.nrows() as f64 - 1.0
            {
                break;
            }

            let terrain_height = self.interpolate_height(current_y, current_x);

            // FIX: Cast current_z to f32 for comparison
            if terrain_height > current_z as f32 {
                return 1.0;
            }
        }

        0.0
    }

    fn interpolate_height(&self, row: f64, col: f64) -> f32 {
        let r0 = row.floor() as usize;
        let c0 = col.floor() as usize;
        let r1 = (r0 + 1).min(self.dsm.nrows() - 1);
        let c1 = (c0 + 1).min(self.dsm.ncols() - 1);

        let fx = col - c0 as f64;
        let fy = row - r0 as f64;

        let h00 = self.dsm[[r0, c0]] as f64;
        let h01 = self.dsm[[r0, c1]] as f64;
        let h10 = self.dsm[[r1, c0]] as f64;
        let h11 = self.dsm[[r1, c1]] as f64;

        let h0 = h00 * (1.0 - fx) + h01 * fx;
        let h1 = h10 * (1.0 - fx) + h11 * fx;

        (h0 * (1.0 - fy) + h1 * fy) as f32
    }

    fn sun_direction(&self, azimuth: f64, elevation: f64) -> (f64, f64, f64) {
        let az_rad = azimuth.to_radians();
        let el_rad = elevation.to_radians();

        let dx = az_rad.sin() * el_rad.cos();
        let dy = az_rad.cos() * el_rad.cos();
        let dz = el_rad.sin();

        (dx, dy, dz)
    }

    fn generate_timestamps(&self) -> Vec<chrono::DateTime<chrono::Utc>> {
        let mut timestamps = Vec::new();

        let start = chrono::DateTime::parse_from_rfc3339(&self.config.start_date)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);
        let end = chrono::DateTime::parse_from_rfc3339(&self.config.end_date)
            .unwrap_or_else(|_| (chrono::Utc::now() + chrono::Duration::days(30)).into())
            .with_timezone(&chrono::Utc);

        // Generate solar-aware timestamps: only during daylight hours for each day
        let mut current_date = start.date_naive();
        let end_date = end.date_naive();

        let interval_duration =
            chrono::Duration::minutes((self.config.hour_interval * 60.0) as i64);

        while current_date <= end_date {
            let current_datetime = current_date.and_hms_opt(12, 0, 0).unwrap().and_utc();

            // Get sunrise and sunset for this day using the AOI center coordinates
            let aoi_center = self.get_aoi_center();
            let sun_calc = crate::sun_position::SunCalculator::new(
                aoi_center.1, // latitude
                aoi_center.0, // longitude
                self.config.angle_precision,
            );

            if let Some((sunrise, sunset)) = sun_calc.calculate_sunrise_sunset(&current_datetime) {
                // Generate timestamps at specified intervals between sunrise and sunset
                let mut daylight_time = sunrise;

                // Ensure we start within the requested date range
                if daylight_time < start {
                    daylight_time = start;
                }

                while daylight_time <= sunset && daylight_time <= end {
                    timestamps.push(daylight_time);
                    daylight_time += interval_duration;
                }
            } else {
                // Handle polar conditions - if solar noon has positive elevation, include some timestamps
                let mut sun_calc_mut = sun_calc;
                let (_, noon_elevation) = sun_calc_mut.get_position(&current_datetime);
                if noon_elevation > 0.0 {
                    // Polar summer: sun never sets, use traditional approach for this day
                    let day_start = current_date
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc()
                        .max(start);
                    let day_end = current_date
                        .and_hms_opt(23, 59, 59)
                        .unwrap()
                        .and_utc()
                        .min(end);

                    let mut day_time = day_start;
                    while day_time <= day_end {
                        timestamps.push(day_time);
                        day_time += interval_duration;
                    }
                }
                // Polar winter: sun never rises, skip this day entirely
            }

            current_date += chrono::Duration::days(1);
        }

        timestamps
    }

    fn get_aoi_center(&self) -> (f64, f64) {
        if let Ok(polygon) = self.config.to_polygon() {
            let coords: Vec<_> = polygon.exterior().coords().collect();
            if !coords.is_empty() {
                let sum_x: f64 = coords.iter().map(|c| c.x).sum();
                let sum_y: f64 = coords.iter().map(|c| c.y).sum();
                (sum_x / coords.len() as f64, sum_y / coords.len() as f64)
            } else {
                (0.0, 0.0) // Fallback
            }
        } else {
            (0.0, 0.0) // Fallback
        }
    }

    fn calculate_summary_stats(
        &self,
        shadow_fraction: &Array3<f32>,
        timestamps: &[chrono::DateTime<chrono::Utc>],
    ) -> SummaryStats {
        let (_n_times, n_rows, n_cols) = shadow_fraction.dim();

        // Pre-calculate solar data for all days in the analysis period
        let aoi_center = self.get_aoi_center();
        let sun_calc = crate::sun_position::SunCalculator::new(
            aoi_center.1, // latitude
            aoi_center.0, // longitude
            self.config.angle_precision,
        );

        // Group timestamps by date and calculate solar hours per day
        let mut daily_solar_hours = std::collections::HashMap::new();
        let mut solar_noon_times = std::collections::HashMap::new();

        for timestamp in timestamps {
            let date = timestamp.date_naive();
            if !daily_solar_hours.contains_key(&date) {
                let solar_hours = sun_calc.get_solar_hours_for_day(timestamp);
                let solar_noon = sun_calc.calculate_solar_noon(timestamp);
                daily_solar_hours.insert(date, solar_hours);
                solar_noon_times.insert(date, solar_noon);
            }
        }

        let total_analysis_days = daily_solar_hours.len() as f32;
        let total_available_solar: f32 = daily_solar_hours.values().map(|&x| x as f32).sum();
        let avg_daily_solar = if total_analysis_days > 0.0 {
            total_available_solar / total_analysis_days
        } else {
            0.0
        };

        println!("Solar calculation debug:");
        println!("  Total analysis days: {}", total_analysis_days);
        println!("  Total available solar hours: {}", total_available_solar);
        println!("  Average daily solar hours: {}", avg_daily_solar);
        println!("  Number of timestamps: {}", timestamps.len());

        // Create arrays for new statistics
        let mut total_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut morning_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut noon_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut afternoon_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));
        let mut solar_efficiency = Array2::<f32>::zeros((n_rows, n_cols));

        // Create a vector of all cell coordinates for parallel processing
        let cell_coords: Vec<(usize, usize)> = (0..n_rows)
            .flat_map(|row| (0..n_cols).map(move |col| (row, col)))
            .collect();

        // Calculate statistics for all cells in parallel
        let stats_results: Vec<(f32, f32, f32, f32, f32, f32)> = cell_coords
            .par_iter()
            .map(|&(row, col)| {
                let cell_series = shadow_fraction.slice(s![.., row, col]);

                // Calculate total shadow hours and morning/noon/afternoon split
                let mut total_shadow_hours_cell = 0.0;
                let mut morning_shadow_hours_cell = 0.0;
                let mut noon_shadow_hours_cell = 0.0;
                let mut afternoon_shadow_hours_cell = 0.0;

                // Calculate shadow hours for each timestamp (simple approach)
                for (t_idx, &timestamp) in timestamps.iter().enumerate() {
                    let shadow_contribution = cell_series[t_idx] * self.config.hour_interval;
                    total_shadow_hours_cell += shadow_contribution;

                    // Morning/noon/afternoon classification using solar noon ± 2 hours
                    let date = timestamp.date_naive();
                    if let Some(&solar_noon) = solar_noon_times.get(&date) {
                        let noon_start = solar_noon - chrono::Duration::hours(2);
                        let noon_end = solar_noon + chrono::Duration::hours(2);

                        if timestamp < noon_start {
                            morning_shadow_hours_cell += shadow_contribution;
                        } else if timestamp <= noon_end {
                            noon_shadow_hours_cell += shadow_contribution;
                        } else {
                            afternoon_shadow_hours_cell += shadow_contribution;
                        }
                    } else {
                        // Fallback to clock-based classification if solar noon calculation fails
                        let hour = timestamp.hour();
                        if hour < 10 {
                            // Before 10 AM
                            morning_shadow_hours_cell += shadow_contribution;
                        } else if hour < 14 {
                            // 10 AM - 2 PM (noon period)
                            noon_shadow_hours_cell += shadow_contribution;
                        } else {
                            // After 2 PM
                            afternoon_shadow_hours_cell += shadow_contribution;
                        }
                    }
                }

                // Max consecutive shadow hours (using actual time intervals)
                let mut current_consecutive = 0.0;
                let mut max_consec = 0.0f32;
                for (_t_idx, &val) in cell_series.iter().enumerate() {
                    if val > 0.5 {
                        current_consecutive += self.config.hour_interval;
                        max_consec = max_consec.max(current_consecutive);
                    } else {
                        current_consecutive = 0.0;
                    }
                }

                // Solar efficiency: fraction of total available solar hours that are not shadowed (0.0-1.0)
                let efficiency = if total_available_solar > 0.0 {
                    ((total_available_solar - total_shadow_hours_cell) / total_available_solar)
                        .max(0.0)
                } else {
                    0.0
                };

                (
                    total_shadow_hours_cell,
                    morning_shadow_hours_cell,
                    noon_shadow_hours_cell,
                    afternoon_shadow_hours_cell,
                    max_consec,
                    efficiency,
                )
            })
            .collect();

        // Assign results back to arrays
        for ((row, col), &(total, morning, noon, afternoon, max_consec, efficiency)) in
            cell_coords.iter().zip(stats_results.iter())
        {
            total_shadow_hours[[*row, *col]] = total;
            morning_shadow_hours[[*row, *col]] = morning;
            noon_shadow_hours[[*row, *col]] = noon;
            afternoon_shadow_hours[[*row, *col]] = afternoon;
            max_consecutive[[*row, *col]] = max_consec;
            solar_efficiency[[*row, *col]] = efficiency;
        }

        // Calculate average shadow percentage as fraction (0.0-1.0, not 0-100)
        // Total measurement time = number of timestamps * hour interval
        let total_measurement_hours = timestamps.len() as f32 * self.config.hour_interval;
        println!("  Total measurement hours: {}", total_measurement_hours);

        let avg_shadow_percentage = if total_measurement_hours > 0.0 {
            &total_shadow_hours / total_measurement_hours
        } else {
            Array2::<f32>::zeros((n_rows, n_cols))
        };

        // Convert to Array3 with summary layers
        let mut total_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut avg_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut max_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut morning_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut noon_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut afternoon_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut efficiency_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut daily_solar_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut total_available_3d = Array3::<f32>::zeros((1, n_rows, n_cols));

        total_3d
            .slice_mut(s![0, .., ..])
            .assign(&total_shadow_hours);
        avg_3d
            .slice_mut(s![0, .., ..])
            .assign(&avg_shadow_percentage);
        max_3d.slice_mut(s![0, .., ..]).assign(&max_consecutive);
        morning_3d
            .slice_mut(s![0, .., ..])
            .assign(&morning_shadow_hours);
        noon_3d.slice_mut(s![0, .., ..]).assign(&noon_shadow_hours);
        afternoon_3d
            .slice_mut(s![0, .., ..])
            .assign(&afternoon_shadow_hours);
        efficiency_3d
            .slice_mut(s![0, .., ..])
            .assign(&solar_efficiency);
        // Each cell gets the average daily solar hours for the analysis period
        daily_solar_3d.fill(avg_daily_solar);
        // Each cell gets total available solar hours for the entire analysis period
        total_available_3d.fill(total_available_solar);

        SummaryStats {
            total_shadow_hours: total_3d,
            avg_shadow_percentage: avg_3d,
            max_consecutive_shadow: max_3d,
            morning_shadow_hours: morning_3d,
            noon_shadow_hours: noon_3d,
            afternoon_shadow_hours: afternoon_3d,
            solar_efficiency_percentage: efficiency_3d,
            daily_solar_hours: daily_solar_3d,
            total_available_solar_hours: total_available_3d,
        }
    }

    pub fn calculate_seasonal_analysis(
        &self,
        shadow_fraction: &Array3<f32>,
        timestamps: &[chrono::DateTime<chrono::Utc>],
    ) -> SeasonalAnalysis {
        use crate::types::{MonthlyShadowStats, SeasonStats, SeasonalAnalysis};
        use chrono::Datelike;
        use std::collections::HashMap;

        let (n_times, n_rows, n_cols) = shadow_fraction.dim();

        // Group timestamps by month-year
        let mut monthly_groups: HashMap<(u32, i32), Vec<usize>> = HashMap::new();
        for (idx, timestamp) in timestamps.iter().enumerate() {
            let month = timestamp.month();
            let year = timestamp.year();
            monthly_groups.entry((month, year)).or_default().push(idx);
        }

        // Calculate monthly statistics
        let mut monthly_stats = Vec::new();
        for ((month, year), time_indices) in monthly_groups.iter() {
            let days_in_analysis = time_indices.len() as f32 / 24.0; // Approximate days

            let mut month_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
            let mut month_shadow_count = Array2::<f32>::zeros((n_rows, n_cols));
            let mut month_max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));
            let mut month_solar_efficiency = Array2::<f32>::zeros((n_rows, n_cols));

            // Calculate statistics for each cell
            for row in 0..n_rows {
                for col in 0..n_cols {
                    let mut total_shadow = 0.0f32;
                    let mut consecutive_shadow = 0.0f32;
                    let mut max_consecutive = 0.0f32;
                    let mut sunlit_hours = 0.0f32;

                    for &time_idx in time_indices {
                        let shadow_val = shadow_fraction[[time_idx, row, col]];

                        if shadow_val > 0.5 {
                            total_shadow += 1.0;
                            consecutive_shadow += 1.0;
                        } else {
                            sunlit_hours += 1.0;
                            max_consecutive = max_consecutive.max(consecutive_shadow);
                            consecutive_shadow = 0.0;
                        }
                    }

                    // Final check for consecutive shadows
                    max_consecutive = max_consecutive.max(consecutive_shadow);

                    month_shadow_hours[[row, col]] = total_shadow;
                    month_shadow_count[[row, col]] = time_indices.len() as f32;
                    month_max_consecutive[[row, col]] = max_consecutive;

                    // Solar efficiency: percentage of time with good solar access
                    if time_indices.len() > 0 {
                        month_solar_efficiency[[row, col]] =
                            (sunlit_hours / time_indices.len() as f32) * 100.0;
                    }
                }
            }

            // Calculate average shadow percentage
            let mut avg_shadow_percentage = Array2::<f32>::zeros((n_rows, n_cols));
            for row in 0..n_rows {
                for col in 0..n_cols {
                    if month_shadow_count[[row, col]] > 0.0 {
                        avg_shadow_percentage[[row, col]] = (month_shadow_hours[[row, col]]
                            / month_shadow_count[[row, col]])
                            * 100.0;
                    }
                }
            }

            monthly_stats.push(MonthlyShadowStats {
                month: *month,
                year: *year,
                total_shadow_hours: month_shadow_hours,
                avg_shadow_percentage,
                max_consecutive_shadow: month_max_consecutive,
                solar_efficiency_percentage: month_solar_efficiency,
                days_in_analysis: days_in_analysis as u32,
            });
        }

        // Sort monthly stats by year and month
        monthly_stats.sort_by(|a, b| (a.year, a.month).cmp(&(b.year, b.month)));

        // Calculate seasonal summaries
        let seasons = vec![
            ("Spring", vec![3, 4, 5]),
            ("Summer", vec![6, 7, 8]),
            ("Fall", vec![9, 10, 11]),
            ("Winter", vec![12, 1, 2]),
        ];

        let mut seasonal_summaries = Vec::new();
        for (season_name, season_months) in seasons {
            let season_months_set: std::collections::HashSet<u32> =
                season_months.iter().cloned().collect();

            // Find matching monthly stats for this season
            let season_stats: Vec<&MonthlyShadowStats> = monthly_stats
                .iter()
                .filter(|ms| season_months_set.contains(&ms.month))
                .collect();

            if !season_stats.is_empty() {
                // Aggregate seasonal data
                let mut season_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
                let mut season_shadow_percentage = Array2::<f32>::zeros((n_rows, n_cols));
                let mut season_max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));
                let mut season_solar_efficiency = Array2::<f32>::zeros((n_rows, n_cols));
                let mut total_days = 0;

                for monthly_stat in &season_stats {
                    season_shadow_hours = &season_shadow_hours + &monthly_stat.total_shadow_hours;
                    season_shadow_percentage =
                        &season_shadow_percentage + &monthly_stat.avg_shadow_percentage;
                    season_solar_efficiency =
                        &season_solar_efficiency + &monthly_stat.solar_efficiency_percentage;
                    total_days += monthly_stat.days_in_analysis;

                    // Max consecutive is the maximum across months
                    for row in 0..n_rows {
                        for col in 0..n_cols {
                            season_max_consecutive[[row, col]] = season_max_consecutive[[row, col]]
                                .max(monthly_stat.max_consecutive_shadow[[row, col]]);
                        }
                    }
                }

                // Average the percentage values
                let season_count = season_stats.len() as f32;
                season_shadow_percentage.mapv_inplace(|x| x / season_count);
                season_solar_efficiency.mapv_inplace(|x| x / season_count);

                seasonal_summaries.push(SeasonStats {
                    season_name: season_name.to_string(),
                    months: season_months,
                    total_shadow_hours: season_shadow_hours,
                    avg_shadow_percentage: season_shadow_percentage,
                    max_consecutive_shadow: season_max_consecutive,
                    solar_efficiency_percentage: season_solar_efficiency,
                    total_days,
                });
            }
        }

        // Determine analysis period
        let start_time = timestamps.iter().min().cloned().unwrap_or_default();
        let end_time = timestamps.iter().max().cloned().unwrap_or_default();

        SeasonalAnalysis {
            monthly_stats,
            seasonal_summaries,
            analysis_period: (start_time, end_time),
        }
    }
}
