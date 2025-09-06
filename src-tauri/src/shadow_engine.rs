use crate::types::*;
use crate::sun_position::SunCalculator;
use ndarray::{Array2, Array3, s};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use chrono::Timelike;  // IMPORTANT: Add this import for hour() method

pub struct ShadowEngine {
    _dtm: Array2<f32>,
    dsm: Array2<f32>,
    heights: Array2<f32>,
    resolution: f64,
    sun_calculator: Arc<Mutex<SunCalculator>>,
    config: Config,
}

impl ShadowEngine {
    pub fn new(dtm: Array2<f32>, dsm: Array2<f32>, resolution: f64, config: Config) -> Self {
        let heights = &dsm - &dtm;
        let polygon = config.to_polygon().unwrap_or(geo_types::Polygon::new(
            geo_types::LineString::from(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.0, 0.0)]),
            vec![],
        ));
        let centroid = geo::algorithm::centroid::Centroid::centroid(&polygon).unwrap_or(geo_types::Point::new(0.0, 0.0));
        let sun_calculator = Arc::new(Mutex::new(
            SunCalculator::new(centroid.y(), centroid.x(), config.angle_precision)
        ));
        
        Self {
            _dtm: dtm,  // Store with underscore to show it's kept but not used later
            dsm,
            heights,
            resolution,
            sun_calculator,
            config,
        }
    }

    pub fn calculate_shadows(&self) -> Result<ShadowResult, ShadowError> {
        let timestamps = self.generate_timestamps();
        let n_times = timestamps.len();
        let (n_rows, n_cols) = self.heights.dim();
        
        let mut shadow_fraction = Array3::<f32>::zeros((n_times, n_rows, n_cols));
        
        let pb = ProgressBar::new(n_times as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap());
        
        // Parallel processing per timestamp
        let results: Vec<_> = timestamps
            .par_iter()
            .enumerate()
            .map(|(t_idx, timestamp)| {
                let mut sun_calc = self.sun_calculator.lock().unwrap();
                let (azimuth, elevation) = sun_calc.get_position(timestamp);
                drop(sun_calc);
                
                let shadow_map = if elevation <= 0.0 {
                    Array2::<f32>::ones((n_rows, n_cols))
                } else {
                    self.calculate_shadow_map(azimuth, elevation)
                };
                
                pb.inc(1);
                (t_idx, shadow_map)
            })
            .collect();
        
        pb.finish_with_message("Shadow calculation complete");
        
        // Assemble results
        for (t_idx, shadow_map) in results {
            shadow_fraction.slice_mut(s![t_idx, .., ..]).assign(&shadow_map);
        }
        
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
        
        // Process each cell
        for row in 0..n_rows {
            for col in 0..n_cols {
                let shadow_value = self.calculate_cell_shadow(row, col, sun_dir);
                shadow_map[[row, col]] = shadow_value;
            }
        }
        
        // Apply edge refinement based on quality setting
        if !matches!(self.config.shadow_quality, ShadowQuality::Fast) {
            shadow_map = self.refine_shadow_edges(shadow_map, sun_dir);
        }
        
        shadow_map
    }

    fn calculate_cell_shadow(&self, row: usize, col: usize, sun_dir: (f64, f64, f64)) -> f32 {
        let cell_height = self.dsm[[row, col]];
        let (dx, dy, dz) = sun_dir;
        
        // Ray marching from cell toward sun
        let mut current_x = col as f64;
        let mut current_y = row as f64;
        let mut current_z = cell_height as f64;
        
        let step_size = 0.5;
        let max_distance = self.config.buffer_meters / self.resolution;
        let mut distance = 0.0;
        
        while distance < max_distance {
            current_x += dx * step_size;
            current_y -= dy * step_size;
            current_z += dz * step_size;
            distance += step_size;
            
            // Check bounds
            if current_x < 0.0 || current_y < 0.0 ||
               current_x >= self.dsm.ncols() as f64 - 1.0 ||
               current_y >= self.dsm.nrows() as f64 - 1.0 {
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

    fn refine_shadow_edges(&self, shadow_map: Array2<f32>, sun_dir: (f64, f64, f64)) -> Array2<f32> {
        let (n_rows, n_cols) = shadow_map.dim();
        let mut refined = shadow_map.clone();
        
        // Detect edges
        for row in 1..n_rows-1 {
            for col in 1..n_cols-1 {
                if self.is_shadow_edge(&shadow_map, row, col) {
                    let sub_samples = match self.config.shadow_quality {
                        ShadowQuality::Normal => 2,
                        ShadowQuality::High => 4,
                        ShadowQuality::Scientific => 8,
                        _ => 1,
                    };
                    
                    refined[[row, col]] = self.subsample_cell(row, col, sun_dir, sub_samples);
                }
            }
        }
        
        refined
    }

    fn is_shadow_edge(&self, shadow_map: &Array2<f32>, row: usize, col: usize) -> bool {
        let center = shadow_map[[row, col]];
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 { continue; }
                let r = (row as i32 + dr) as usize;
                let c = (col as i32 + dc) as usize;
                if (shadow_map[[r, c]] - center).abs() > 0.5 {
                    return true;
                }
            }
        }
        false
    }

    fn subsample_cell(&self, row: usize, col: usize, sun_dir: (f64, f64, f64), samples: usize) -> f32 {
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
        let max_distance = self.config.buffer_meters / self.resolution;
        let mut distance = 0.0;
        
        while distance < max_distance {
            current_x += dx * step_size;
            current_y -= dy * step_size;
            current_z += dz * step_size;
            distance += step_size;
            
            if current_x < 0.0 || current_y < 0.0 ||
               current_x >= self.dsm.ncols() as f64 - 1.0 ||
               current_y >= self.dsm.nrows() as f64 - 1.0 {
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
        
        let mut current = start;
        
        while current <= end {
            timestamps.push(current);
            current = current + chrono::Duration::hours(self.config.hour_interval as i64);
        }
        
        timestamps
    }

    fn calculate_summary_stats(&self, shadow_fraction: &Array3<f32>, timestamps: &[chrono::DateTime<chrono::Utc>]) -> SummaryStats {
        let (n_times, n_rows, n_cols) = shadow_fraction.dim();
        
        let mut total_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut morning_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut afternoon_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
        let mut max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));
        
        for row in 0..n_rows {
            for col in 0..n_cols {
                let cell_series = shadow_fraction.slice(s![.., row, col]);
                
                // Total shadow hours
                total_shadow_hours[[row, col]] = cell_series.sum() * self.config.hour_interval;
                
                // Morning vs afternoon
                for (t_idx, &timestamp) in timestamps.iter().enumerate() {
                    let hour = timestamp.hour();  // This now works with the Timelike import
                    let shadow_val = cell_series[t_idx] * self.config.hour_interval;
                    
                    if hour < 12 {
                        morning_shadow_hours[[row, col]] += shadow_val;
                    } else {
                        afternoon_shadow_hours[[row, col]] += shadow_val;
                    }
                }
                
                // Max consecutive shadow hours
                let mut current_consecutive = 0.0;
                let mut max_consec: f32 = 0.0;  // FIX: Explicitly typed as f32
                for &val in cell_series.iter() {
                    if val > 0.5 {
                        current_consecutive += self.config.hour_interval;
                        max_consec = max_consec.max(current_consecutive);
                    } else {
                        current_consecutive = 0.0;
                    }
                }
                max_consecutive[[row, col]] = max_consec;
            }
        }
        
        let avg_shadow_percentage = &total_shadow_hours / (n_times as f32 * self.config.hour_interval);
        
        // Convert to Array3 with summary layers
        let mut total_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut avg_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut max_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut morning_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        let mut afternoon_3d = Array3::<f32>::zeros((1, n_rows, n_cols));
        
        total_3d.slice_mut(s![0, .., ..]).assign(&total_shadow_hours);
        avg_3d.slice_mut(s![0, .., ..]).assign(&avg_shadow_percentage);
        max_3d.slice_mut(s![0, .., ..]).assign(&max_consecutive);
        morning_3d.slice_mut(s![0, .., ..]).assign(&morning_shadow_hours);
        afternoon_3d.slice_mut(s![0, .., ..]).assign(&afternoon_shadow_hours);
        
        SummaryStats {
            total_shadow_hours: total_3d,
            avg_shadow_percentage: avg_3d,
            max_consecutive_shadow: max_3d,
            morning_shadow_hours: morning_3d,
            afternoon_shadow_hours: afternoon_3d,
        }
    }
}