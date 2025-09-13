#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod raster_io;
mod shadow_engine;
mod sun_position;
mod types;

use raster_io::RasterIO;
use serde::{Deserialize, Serialize};
use shadow_engine::ShadowEngine;
use std::path::Path;
use std::sync::Mutex;
use tauri::State;
use types::*;

struct AppState {
    current_config: Mutex<Option<Config>>,
    current_results: Mutex<Option<ShadowResult>>,
    raster_bounds: Mutex<Option<RasterBounds>>,
    clipped_raster_info: Mutex<Option<ClippedRasterInfo>>,
    aoi_polygon: Mutex<Option<geo_types::Polygon<f64>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClippedRasterInfo {
    bounds: RasterBounds,
    transform: Vec<f64>,
    dimensions: (usize, usize), // (rows, cols)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RasterBounds {
    min_lon: f64,
    max_lon: f64,
    min_lat: f64,
    max_lat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MonthlyShadowStatsData {
    month: u32,
    year: i32,
    total_shadow_hours: Vec<Vec<f32>>,
    avg_shadow_percentage: Vec<Vec<f32>>,
    max_consecutive_shadow: Vec<Vec<f32>>,
    solar_efficiency_percentage: Vec<Vec<f32>>,
    days_in_analysis: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SeasonStatsData {
    season_name: String,
    months: Vec<u32>,
    total_shadow_hours: Vec<Vec<f32>>,
    avg_shadow_percentage: Vec<Vec<f32>>,
    max_consecutive_shadow: Vec<Vec<f32>>,
    solar_efficiency_percentage: Vec<Vec<f32>>,
    total_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SeasonalAnalysisData {
    monthly_stats: Vec<MonthlyShadowStatsData>,
    seasonal_summaries: Vec<SeasonStatsData>,
    analysis_period: (String, String), // ISO 8601 datetime strings
}

#[tauri::command]
async fn load_rasters(
    dtm_path: String,
    dsm_path: String,
    state: State<'_, AppState>,
) -> Result<RasterBounds, String> {
    let dtm = RasterIO::read_raster(Path::new(&dtm_path))
        .map_err(|e| format!("Failed to load DTM: {}", e))?;
    let dsm = RasterIO::read_raster(Path::new(&dsm_path))
        .map_err(|e| format!("Failed to load DSM: {}", e))?;

    // Validate matching dimensions
    if dtm.data.shape() != dsm.data.shape() {
        return Err("DTM and DSM must have the same dimensions".to_string());
    }

    // Calculate bounds (assuming WGS84 or getting from transform)
    let (height, width) = (dtm.data.shape()[1], dtm.data.shape()[2]);
    let transform = &dtm.transform;

    // Calculate corner coordinates
    let min_lon = transform[0];
    let max_lon = transform[0] + (width as f64 * transform[1]);
    let max_lat = transform[3];
    let min_lat = transform[3] + (height as f64 * transform[5]); // transform[5] is negative

    let bounds = RasterBounds {
        min_lon,
        max_lon,
        min_lat: min_lat.min(max_lat),
        max_lat: max_lat.max(min_lat),
    };

    // Store bounds in state
    let mut bounds_guard = state.raster_bounds.lock().unwrap();
    *bounds_guard = Some(bounds.clone());

    println!(
        "Raster bounds: lon [{:.6}, {:.6}], lat [{:.6}, {:.6}]",
        bounds.min_lon, bounds.max_lon, bounds.min_lat, bounds.max_lat
    );

    Ok(bounds)
}

#[tauri::command]
async fn calculate_shadows(
    config: Config,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    println!("Starting shadow calculation with config: {:?}", config);

    // Load rasters
    let dtm_data = RasterIO::read_raster(Path::new(&config.dtm_path))
        .map_err(|e| format!("Failed to load DTM: {}", e))?;
    let dsm_data = RasterIO::read_raster(Path::new(&config.dsm_path))
        .map_err(|e| format!("Failed to load DSM: {}", e))?;

    // Convert AOI to polygon
    let polygon = config
        .to_polygon()
        .map_err(|e| format!("Failed to parse AOI: {}", e))?;

    // Calculate automatic buffer based on terrain and solar geometry
    let auto_buffer_meters = RasterIO::calculate_automatic_buffer(
        &dtm_data,
        &dsm_data,
        &polygon,
        &config.start_date,
        &config.end_date,
    )
    .map_err(|e| format!("Failed to calculate automatic buffer: {}", e))?;

    // Get the center latitude for conversion
    let center_lat = polygon.exterior().coords().map(|c| c.y).sum::<f64>()
        / polygon.exterior().coords().count() as f64;

    let buffer_degrees = meters_to_degrees(auto_buffer_meters, center_lat);
    println!(
        "Automatic buffer: {:.1}m = {:.6}° at latitude {:.3}°",
        auto_buffer_meters, buffer_degrees, center_lat
    );

    // Clip to AOI with buffer (now in degrees)
    let dtm_clipped = RasterIO::clip_to_aoi(&dtm_data, &polygon, buffer_degrees)
        .map_err(|e| format!("Failed to clip DTM: {}", e))?;
    let dsm_clipped = RasterIO::clip_to_aoi(&dsm_data, &polygon, buffer_degrees)
        .map_err(|e| format!("Failed to clip DSM: {}", e))?;

    // Extract 2D arrays
    let dtm_2d = dtm_clipped.data.slice(ndarray::s![0, .., ..]).to_owned();
    let dsm_2d = dsm_clipped.data.slice(ndarray::s![0, .., ..]).to_owned();

    // Store clipped raster information for later visualization
    let (n_rows, n_cols) = dtm_2d.dim();
    let clipped_bounds = RasterBounds {
        min_lon: dtm_clipped.transform[0],
        max_lon: dtm_clipped.transform[0] + (n_cols as f64 * dtm_clipped.transform[1]),
        min_lat: dtm_clipped.transform[3] + (n_rows as f64 * dtm_clipped.transform[5]), // transform[5] is negative
        max_lat: dtm_clipped.transform[3],
    };

    let clipped_info = ClippedRasterInfo {
        bounds: clipped_bounds,
        transform: dtm_clipped.transform.to_vec(),
        dimensions: (n_rows, n_cols),
    };

    let mut clipped_info_guard = state.clipped_raster_info.lock().unwrap();
    *clipped_info_guard = Some(clipped_info);
    drop(clipped_info_guard);

    // Calculate pixel resolution in meters
    let resolution = degrees_to_meters(dtm_clipped.transform[1].abs(), center_lat);
    println!(
        "Pixel resolution: {:.6}° = {:.2}m at latitude {:.3}°",
        dtm_clipped.transform[1].abs(),
        resolution,
        center_lat
    );

    // Create shadow engine with automatic buffer in meters
    let mut config_with_meter_buffer = config.clone();
    config_with_meter_buffer.buffer_meters = Some(auto_buffer_meters); // Use automatic buffer

    let engine = ShadowEngine::new_with_app_handle(
        dtm_2d,
        dsm_2d,
        resolution,
        dtm_clipped.transform,
        config_with_meter_buffer,
        app_handle,
    );
    let results = engine
        .calculate_shadows()
        .map_err(|e| format!("Shadow calculation failed: {}", e))?;

    let num_timestamps = results.timestamps.len();

    // Results are now calculated only for AOI cells - no masking needed
    println!("Shadow calculation complete with AOI-optimized arrays");

    // Store results in state
    let mut results_guard = state.current_results.lock().unwrap();
    *results_guard = Some(results);

    let mut config_guard = state.current_config.lock().unwrap();
    *config_guard = Some(config);

    Ok(format!(
        "Calculated shadows for {} timestamps",
        num_timestamps
    ))
}

// Helper function to convert meters to degrees
fn meters_to_degrees(meters: f64, latitude: f64) -> f64 {
    // At the equator: 1 degree ≈ 111,320 meters
    // At latitude φ: 1 degree longitude ≈ 111,320 * cos(φ) meters
    // 1 degree latitude ≈ 111,320 meters (approximately constant)

    let lat_rad = latitude.to_radians();

    // For longitude: meters / (111320 * cos(latitude))
    // For latitude: meters / 111320
    // We'll use an average for a rough square buffer

    let lon_meters_per_degree = 111320.0 * lat_rad.cos();
    let lat_meters_per_degree = 111320.0;

    // Use the average for a roughly square buffer
    let avg_meters_per_degree = (lon_meters_per_degree + lat_meters_per_degree) / 2.0;

    meters / avg_meters_per_degree
}

// Helper function to convert degrees to meters
fn degrees_to_meters(degrees: f64, latitude: f64) -> f64 {
    let lat_rad = latitude.to_radians();
    let lon_meters_per_degree = 111320.0 * lat_rad.cos();
    degrees * lon_meters_per_degree
}

#[tauri::command]
async fn get_shadow_at_time(
    time_index: usize,
    state: State<'_, AppState>,
) -> Result<Vec<Vec<f32>>, String> {
    let results = state.current_results.lock().unwrap();

    match results.as_ref() {
        Some(results) => {
            let (n_times, n_aoi_cells, _) = results.shadow_fraction.dim();

            if time_index >= n_times {
                return Err(format!(
                    "Time index {} out of range (max: {})",
                    time_index,
                    n_times - 1
                ));
            }

            // Get clipped raster info for dimensions
            let clipped_info = state.clipped_raster_info.lock().unwrap();
            let (n_rows, n_cols) = match clipped_info.as_ref() {
                Some(info) => info.dimensions,
                None => return Err("No raster info available".to_string()),
            };

            // Reconstruct full raster from AOI-only data
            let mut data: Vec<Vec<f32>> = vec![vec![f32::NAN; n_cols]; n_rows];

            // Map AOI-only results back to full raster coordinates
            for aoi_idx in 0..n_aoi_cells {
                if aoi_idx < results.aoi_cells.len() {
                    let (row, col) = results.aoi_cells[aoi_idx];
                    if row < n_rows && col < n_cols {
                        data[row][col] = results.shadow_fraction[[time_index, aoi_idx, 0]];
                    }
                }
            }

            Ok(data)
        }
        None => Err("No results available".to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RasterData {
    data: Vec<Vec<f32>>,
    bounds: RasterBounds,
    transform: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AllSummaryData {
    total_shadow_hours: Vec<Vec<f32>>,
    avg_shadow_percentage: Vec<Vec<f32>>,
    max_consecutive_shadow: Vec<Vec<f32>>,
    morning_shadow_hours: Vec<Vec<f32>>,
    noon_shadow_hours: Vec<Vec<f32>>,
    afternoon_shadow_hours: Vec<Vec<f32>>,
    daily_solar_hours: Vec<Vec<f32>>,
    total_available_solar_hours: Vec<Vec<f32>>,
    bounds: RasterBounds,
    transform: Vec<f64>,
}

#[tauri::command]
async fn get_average_shadow_raster(state: State<'_, AppState>) -> Result<RasterData, String> {
    let results = state.current_results.lock().unwrap();
    let clipped_info = state.clipped_raster_info.lock().unwrap();

    match (results.as_ref(), clipped_info.as_ref()) {
        (Some(results), Some(clipped_info)) => {
            let (n_rows, n_cols) = clipped_info.dimensions;

            // Reconstruct full raster from AOI-only data
            let mut data: Vec<Vec<f32>> = vec![vec![f32::NAN; n_cols]; n_rows];

            // Map AOI-only results back to full raster coordinates
            let (_n_times, n_aoi_cells, _) = results.summary_stats.avg_shadow_percentage.dim();
            for aoi_idx in 0..n_aoi_cells {
                if aoi_idx < results.aoi_cells.len() {
                    let (row, col) = results.aoi_cells[aoi_idx];
                    if row < n_rows && col < n_cols {
                        data[row][col] = results.summary_stats.avg_shadow_percentage[[0, aoi_idx, 0]];
                    }
                }
            }

            Ok(RasterData {
                data,
                bounds: clipped_info.bounds.clone(),
                transform: clipped_info.transform.clone(),
            })
        }
        _ => Err("No results or clipped raster information available".to_string()),
    }
}

#[tauri::command]
async fn get_all_summary_data(state: State<'_, AppState>) -> Result<AllSummaryData, String> {
    let results = state.current_results.lock().unwrap();
    let clipped_info = state.clipped_raster_info.lock().unwrap();

    match (results.as_ref(), clipped_info.as_ref()) {
        (Some(results), Some(clipped_info)) => {
            let (n_rows, n_cols) = clipped_info.dimensions;
            let (_n_times, n_aoi_cells, _) = results.summary_stats.total_shadow_hours.dim();

            // Helper function to reconstruct full raster from AOI-only data
            let reconstruct_raster = |aoi_data: &ndarray::Array3<f32>| -> Vec<Vec<f32>> {
                let mut data: Vec<Vec<f32>> = vec![vec![f32::NAN; n_cols]; n_rows];
                for aoi_idx in 0..n_aoi_cells {
                    if aoi_idx < results.aoi_cells.len() {
                        let (row, col) = results.aoi_cells[aoi_idx];
                        if row < n_rows && col < n_cols {
                            data[row][col] = aoi_data[[0, aoi_idx, 0]];
                        }
                    }
                }
                data
            };

            // Reconstruct all summary layers from AOI-only data
            let total_shadow_hours = reconstruct_raster(&results.summary_stats.total_shadow_hours);
            let avg_shadow_percentage = reconstruct_raster(&results.summary_stats.avg_shadow_percentage);
            let max_consecutive_shadow = reconstruct_raster(&results.summary_stats.max_consecutive_shadow);
            let morning_shadow_hours = reconstruct_raster(&results.summary_stats.morning_shadow_hours);
            let noon_shadow_hours = reconstruct_raster(&results.summary_stats.noon_shadow_hours);
            let afternoon_shadow_hours = reconstruct_raster(&results.summary_stats.afternoon_shadow_hours);
            let daily_solar_hours = reconstruct_raster(&results.summary_stats.daily_solar_hours);
            let total_available_solar_hours = reconstruct_raster(&results.summary_stats.total_available_solar_hours);

            Ok(AllSummaryData {
                total_shadow_hours,
                avg_shadow_percentage,
                max_consecutive_shadow,
                morning_shadow_hours,
                noon_shadow_hours,
                afternoon_shadow_hours,
                daily_solar_hours,
                total_available_solar_hours,
                bounds: clipped_info.bounds.clone(),
                transform: clipped_info.transform.clone(),
            })
        }
        _ => Err("No results or clipped raster information available".to_string()),
    }
}

#[tauri::command]
async fn export_results(
    output_path: String,
    _format: String, // Only GeoTIFF + GPKG supported now
    state: State<'_, AppState>,
) -> Result<String, String> {
    let results = state.current_results.lock().unwrap();
    let config = state.current_config.lock().unwrap();
    let clipped_info = state.clipped_raster_info.lock().unwrap();

    match (results.as_ref(), config.as_ref(), clipped_info.as_ref()) {
        (Some(results), Some(config), Some(clipped_info)) => {
            // Create output directory in user's home or documents folder
            let output_dir = match dirs::document_dir() {
                Some(dir) => dir.join("ShadowCalculator_Exports"),
                None => std::env::current_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."))
                    .parent()
                    .unwrap_or(&std::path::PathBuf::from("."))
                    .join("exports"),
            };

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&output_dir)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;

            // Generate timestamp for unique filename
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

            // Build full paths with timestamp for both TIF and GPKG
            let tif_filename = format!("shadows_{}.tif", timestamp);
            let gpkg_filename = format!("aoi_{}.gpkg", timestamp);

            let tif_path = output_dir.join(&tif_filename);
            let gpkg_path = output_dir.join(&gpkg_filename);

            println!("Exporting GeoTIFF to: {:?}", tif_path);
            println!("Exporting AOI GPKG to: {:?}", gpkg_path);

            // Get AOI polygon (needed for both exports)
            let polygon = config
                .to_polygon()
                .map_err(|e| format!("Failed to parse AOI: {}", e))?;

            // Export GeoTIFF using stored clipped raster info
            {
                    // Use the stored clipped raster info from calculation
                    let (n_rows, n_cols) = clipped_info.dimensions;
                    let (n_times, n_aoi_cells, _) = results.shadow_fraction.dim();
                    let n_summary = 9;

                    // Reconstruct full raster arrays from AOI-only results
                    let mut combined =
                        ndarray::Array3::<f32>::zeros((n_summary + n_times, n_rows, n_cols));
                    combined.fill(f32::NAN); // Fill with NaN for cells outside AOI

                    // Map AOI-only results back to full raster coordinates
                    // Need to convert original AOI cell coordinates to the current clipped raster coordinate system
                    for aoi_idx in 0..n_aoi_cells {
                        if aoi_idx < results.aoi_cells.len() {
                            let (orig_row, orig_col) = results.aoi_cells[aoi_idx];

                            // Convert original cell coordinates to world coordinates
                            // This needs to be done using the transform from when the calculation was performed
                            // For now, let's work directly with pixel coordinates and add bounds checking

                            // Simple approach: check bounds before accessing
                            if orig_row < n_rows && orig_col < n_cols {
                                // Summary stats layers
                                combined[[0, orig_row, orig_col]] = results.summary_stats.total_shadow_hours[[0, aoi_idx, 0]];
                                combined[[1, orig_row, orig_col]] = results.summary_stats.avg_shadow_percentage[[0, aoi_idx, 0]];
                                combined[[2, orig_row, orig_col]] = results.summary_stats.max_consecutive_shadow[[0, aoi_idx, 0]];
                                combined[[3, orig_row, orig_col]] = results.summary_stats.morning_shadow_hours[[0, aoi_idx, 0]];
                                combined[[4, orig_row, orig_col]] = results.summary_stats.noon_shadow_hours[[0, aoi_idx, 0]];
                                combined[[5, orig_row, orig_col]] = results.summary_stats.afternoon_shadow_hours[[0, aoi_idx, 0]];
                                combined[[6, orig_row, orig_col]] = results.summary_stats.solar_efficiency_percentage[[0, aoi_idx, 0]];
                                combined[[7, orig_row, orig_col]] = results.summary_stats.daily_solar_hours[[0, aoi_idx, 0]];
                                combined[[8, orig_row, orig_col]] = results.summary_stats.total_available_solar_hours[[0, aoi_idx, 0]];

                                // Time series layers
                                for t_idx in 0..n_times {
                                    combined[[n_summary + t_idx, orig_row, orig_col]] = results.shadow_fraction[[t_idx, aoi_idx, 0]];
                                }
                            } else {
                                println!("Warning: AOI cell coordinate ({}, {}) is out of bounds for current raster ({}, {})", orig_row, orig_col, n_rows, n_cols);
                            }
                        }
                    }

                    // Results are already masked to AOI during calculation

                    // Create band descriptions for better identification
                    let mut band_descriptions = vec![
                        "Total_Shadow_Hours".to_string(),
                        "Average_Shadow_Fraction_(0-1)".to_string(),
                        "Max_Consecutive_Shadow_Hours".to_string(),
                        "Morning_Shadow_Hours_(before_solar_noon_minus_2h)".to_string(),
                        "Noon_Shadow_Hours_(solar_noon_±2h)".to_string(),
                        "Afternoon_Shadow_Hours_(after_solar_noon_plus_2h)".to_string(),
                        "Solar_Efficiency_Fraction_(0-1)".to_string(),
                        "Average_Daily_Solar_Hours".to_string(),
                        "Total_Available_Solar_Hours".to_string(),
                    ];

                    // Add timestamp descriptions for each time layer
                    for timestamp in &results.timestamps {
                        band_descriptions.push(timestamp.format("%Y-%m-%d_%H:%M_UTC").to_string());
                    }

                    // Write GeoTIFF with band descriptions using stored transform
                    // Convert Vec<f64> back to [f64; 6] for the transform
                    let transform_array: [f64; 6] = clipped_info.transform.as_slice().try_into()
                        .map_err(|_| "Invalid transform array length".to_string())?;

                    RasterIO::write_geotiff_with_descriptions(
                        &tif_path,
                        &combined,
                        &transform_array,
                        "EPSG:4326", // Use WGS84 as default projection
                        &band_descriptions,
                    )
                    .map_err(|e| format!("Failed to write GeoTIFF: {}", e))?;
            }

            // Export AOI polygon as GPKG
            RasterIO::write_gpkg(&gpkg_path, &polygon)
                .map_err(|e| format!("Failed to write AOI GPKG: {}", e))?;

            Ok(format!(
                "Export complete:\n- GeoTIFF: {}\n- AOI GPKG: {}",
                tif_path.display(),
                gpkg_path.display()
            ))
        }
        _ => Err("No results, config, or raster info available to export".to_string()),
    }
}

#[tauri::command]
async fn get_timestamps(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let results = state.current_results.lock().unwrap();

    match results.as_ref() {
        Some(results) => {
            let timestamps: Vec<String> =
                results.timestamps.iter().map(|t| t.to_rfc3339()).collect();
            Ok(timestamps)
        }
        None => Err("No results available".to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpuInfo {
    total_cores: usize,
    logical_cores: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResultsMetadata {
    start_date: String,
    end_date: String,
    hour_interval: f64,
    total_timestamps: usize,
    summary_layers: Vec<String>,
    bounds: RasterBounds,
}

#[tauri::command]
async fn validate_results_file(file_path: String) -> Result<ResultsMetadata, String> {
    let path = Path::new(&file_path);

    // Check file extension
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "tif" | "tiff" => {
            // Handle TIF files
            let (raster_data, band_descriptions) = RasterIO::read_multiband_raster_with_descriptions(path)
                .map_err(|e| format!("Failed to read raster file: {}", e))?;

            return validate_tif_file(raster_data, band_descriptions);
        },
        "gpkg" => {
            // Handle GPKG files - they only contain AOI polygon
            let _polygon = RasterIO::read_gpkg(path)
                .map_err(|e| format!("Failed to read GPKG file: {}", e))?;

            return Ok(ResultsMetadata {
                start_date: "N/A".to_string(),
                end_date: "N/A".to_string(),
                hour_interval: 0.0,
                total_timestamps: 0,
                summary_layers: vec!["AOI Polygon".to_string()],
                bounds: RasterBounds {
                    min_lon: 0.0,
                    max_lon: 0.0,
                    min_lat: 0.0,
                    max_lat: 0.0,
                },
            });
        },
        _ => {
            return Err("File must be a .tif, .tiff, or .gpkg file".to_string());
        }
    }
}

fn validate_tif_file(raster_data: crate::types::RasterData, band_descriptions: Vec<String>) -> Result<ResultsMetadata, String> {
    // Try to read the raster file with all bands and descriptions

    let shape = raster_data.data.shape();
    let (n_bands, n_rows, n_cols) = (shape[0], shape[1], shape[2]);

    // Validate that we have at least 9 summary bands
    if n_bands < 9 {
        return Err(format!(
            "Results file must have at least 9 bands (summary layers), found {}",
            n_bands
        ));
    }

    // Expected summary layers (first 9 bands)
    let summary_layers = vec![
        "Total Shadow Hours".to_string(),
        "Average Shadow Fraction".to_string(),
        "Max Consecutive Shadow Hours".to_string(),
        "Morning Shadow Hours".to_string(),
        "Noon Shadow Hours".to_string(),
        "Afternoon Shadow Hours".to_string(),
        "Solar Efficiency Fraction".to_string(),
        "Average Daily Solar Hours".to_string(),
        "Total Available Solar Hours".to_string(),
    ];

    // Calculate bounds
    let transform = &raster_data.transform;
    let width = n_cols;
    let height = n_rows;

    let min_lon = transform[0];
    let max_lon = transform[0] + (width as f64 * transform[1]);
    let max_lat = transform[3];
    let min_lat = transform[3] + (height as f64 * transform[5]);

    let bounds = RasterBounds {
        min_lon,
        max_lon,
        min_lat: min_lat.min(max_lat),
        max_lat: max_lat.max(min_lat),
    };

    // Extract metadata from band descriptions
    let num_time_bands = n_bands - 9; // First 9 bands are summary layers

    let (start_date, end_date, estimated_hour_interval) = if num_time_bands > 0
        && band_descriptions.len() > 9
    {
        // Parse timestamps from band descriptions (bands 9+ contain timestamp info)
        let timestamps: Vec<chrono::DateTime<chrono::Utc>> = band_descriptions[9..]
            .iter()
            .filter_map(|desc| {
                // Try to parse timestamp from description format: "YYYY-MM-DD_HH:MM_UTC"
                chrono::DateTime::parse_from_str(
                    &desc.replace("_UTC", " +0000"),
                    "%Y-%m-%d_%H:%M %z",
                )
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .or_else(|_| {
                    // Try fallback format without timezone
                    chrono::NaiveDateTime::parse_from_str(desc, "%Y-%m-%d_%H:%M")
                        .map(|ndt| chrono::DateTime::from_naive_utc_and_offset(ndt, chrono::Utc))
                })
                .ok()
            })
            .collect();

        if timestamps.len() >= 2 {
            let start = timestamps.iter().min().unwrap();
            let end = timestamps.iter().max().unwrap();
            let duration = *end - *start;
            let estimated_interval = if timestamps.len() > 1 {
                duration.num_hours() as f64 / (timestamps.len() - 1) as f64
            } else {
                1.0
            };

            (start.to_rfc3339(), end.to_rfc3339(), estimated_interval)
        } else {
            // Fallback to current time if parsing fails
            let now = chrono::Utc::now().to_rfc3339();
            (now.clone(), now, 1.0)
        }
    } else {
        // No time bands, use current time
        let now = chrono::Utc::now().to_rfc3339();
        (now.clone(), now, 1.0)
    };

    Ok(ResultsMetadata {
        start_date,
        end_date,
        hour_interval: estimated_hour_interval,
        total_timestamps: num_time_bands,
        summary_layers,
        bounds,
    })
}

#[tauri::command]
async fn load_results_file(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = Path::new(&file_path);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "tif" | "tiff" => {
            return load_tif_results(path, state).await;
        },
        "gpkg" => {
            return load_gpkg_results(path, state).await;
        },
        _ => {
            return Err("File must be a .tif, .tiff, or .gpkg file".to_string());
        }
    }
}

async fn load_tif_results(
    path: &Path,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Read the results file with all bands and descriptions
    let (raster_data, band_descriptions) = RasterIO::read_multiband_raster_with_descriptions(path)
        .map_err(|e| format!("Failed to read results file: {}", e))?;

    // Check for accompanying GPKG file with same base name
    let mut aoi_polygon: Option<geo_types::Polygon<f64>> = None;
    if let Some(parent_dir) = path.parent() {
        if let Some(file_stem) = path.file_stem() {
            if let Some(stem_str) = file_stem.to_str() {
                // Look for GPKG with pattern: aoi_<timestamp>.gpkg where TIF is shadows_<timestamp>.tif
                if stem_str.starts_with("shadows_") {
                    let timestamp = &stem_str[8..]; // Remove "shadows_" prefix
                    let gpkg_name = format!("aoi_{}.gpkg", timestamp);
                    let gpkg_path = parent_dir.join(gpkg_name);

                    if gpkg_path.exists() {
                        match RasterIO::read_gpkg(&gpkg_path) {
                            Ok(polygon) => {
                                aoi_polygon = Some(polygon);
                                println!("Loaded AOI polygon from: {:?}", gpkg_path);
                            },
                            Err(e) => {
                                println!("Warning: Failed to load AOI GPKG: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    let shape = raster_data.data.shape();
    let (n_bands, n_rows, n_cols) = (shape[0], shape[1], shape[2]);

    if n_bands < 9 {
        return Err("Invalid results file: missing summary layers".to_string());
    }

    // Extract summary stats (first 9 bands)
    let summary_stats = SummaryStats {
        total_shadow_hours: raster_data.data.slice(ndarray::s![0..1, .., ..]).to_owned(),
        avg_shadow_percentage: raster_data.data.slice(ndarray::s![1..2, .., ..]).to_owned(),
        max_consecutive_shadow: raster_data.data.slice(ndarray::s![2..3, .., ..]).to_owned(),
        morning_shadow_hours: raster_data.data.slice(ndarray::s![3..4, .., ..]).to_owned(),
        noon_shadow_hours: raster_data.data.slice(ndarray::s![4..5, .., ..]).to_owned(),
        afternoon_shadow_hours: raster_data.data.slice(ndarray::s![5..6, .., ..]).to_owned(),
        solar_efficiency_percentage: raster_data.data.slice(ndarray::s![6..7, .., ..]).to_owned(),
        daily_solar_hours: raster_data.data.slice(ndarray::s![7..8, .., ..]).to_owned(),
        total_available_solar_hours: raster_data.data.slice(ndarray::s![8..9, .., ..]).to_owned(),
    };

    // Extract time series data (bands 9+)
    let num_time_bands = n_bands - 9;
    let shadow_fraction = if num_time_bands > 0 {
        raster_data.data.slice(ndarray::s![9.., .., ..]).to_owned()
    } else {
        // Create empty time series if no time data
        ndarray::Array3::<f32>::zeros((0, n_rows, n_cols))
    };

    // Parse timestamps from band descriptions (bands 9+ contain timestamp info)
    let timestamps: Vec<chrono::DateTime<chrono::Utc>> = if num_time_bands > 0 {
        band_descriptions[9..]
            .iter()
            .map(|desc| {
                // Try to parse timestamp from description format: "YYYY-MM-DD_HH:MM_UTC"
                chrono::DateTime::parse_from_str(
                    &desc.replace("_UTC", " +0000"),
                    "%Y-%m-%d_%H:%M %z",
                )
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| {
                    // If parsing fails, try fallback format without timezone
                    chrono::NaiveDateTime::parse_from_str(desc, "%Y-%m-%d_%H:%M")
                        .map(|ndt| chrono::DateTime::from_naive_utc_and_offset(ndt, chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()) // Last resort fallback
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    // Create shadow results
    let results = ShadowResult {
        shadow_fraction,
        timestamps,
        summary_stats,
        aoi_cells: Vec::new(), // Empty for seasonal analysis
    };

    // Calculate bounds from transform
    let transform = &raster_data.transform;
    let min_lon = transform[0];
    let max_lon = transform[0] + (n_cols as f64 * transform[1]);
    let max_lat = transform[3];
    let min_lat = transform[3] + (n_rows as f64 * transform[5]);

    let bounds = RasterBounds {
        min_lon,
        max_lon,
        min_lat: min_lat.min(max_lat),
        max_lat: max_lat.max(min_lat),
    };

    // Store results and metadata in state
    let clipped_info = ClippedRasterInfo {
        bounds: bounds.clone(),
        transform: raster_data.transform.to_vec(),
        dimensions: (n_rows, n_cols),
    };

    let mut results_guard = state.current_results.lock().unwrap();
    *results_guard = Some(results);

    let mut clipped_info_guard = state.clipped_raster_info.lock().unwrap();
    *clipped_info_guard = Some(clipped_info);

    let mut bounds_guard = state.raster_bounds.lock().unwrap();
    *bounds_guard = Some(bounds);

    // Store AOI polygon if loaded
    let mut aoi_guard = state.aoi_polygon.lock().unwrap();
    *aoi_guard = aoi_polygon;

    let aoi_message = if aoi_guard.is_some() {
        " (with AOI polygon)"
    } else {
        ""
    };

    Ok(format!(
        "Loaded results with {} summary layers and {} timestamps{}",
        5, num_time_bands, aoi_message
    ))
}

async fn load_gpkg_results(
    path: &Path,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Load AOI polygon from GPKG file
    let polygon = RasterIO::read_gpkg(path)
        .map_err(|e| format!("Failed to read GPKG file: {}", e))?;

    // Store AOI polygon in state
    let mut aoi_guard = state.aoi_polygon.lock().unwrap();
    *aoi_guard = Some(polygon);

    // Clear results and metadata since GPKG only contains AOI
    let mut results_guard = state.current_results.lock().unwrap();
    *results_guard = None;

    let mut clipped_info_guard = state.clipped_raster_info.lock().unwrap();
    *clipped_info_guard = None;

    let mut bounds_guard = state.raster_bounds.lock().unwrap();
    *bounds_guard = None;

    Ok("Loaded AOI polygon from GPKG file. Ready for new calculation.".to_string())
}

#[tauri::command]
async fn debug_tiff_structure(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);

    // Try to read the raster file with all bands
    let raster_data = RasterIO::read_multiband_raster(path)
        .map_err(|e| format!("Failed to read raster file: {}", e))?;

    let shape = raster_data.data.shape();
    let (n_bands, n_rows, n_cols) = (shape[0], shape[1], shape[2]);

    let mut debug_info = format!(
        "File: {}\nShape: {:?}\nBands: {}, Rows: {}, Cols: {}\n",
        file_path, shape, n_bands, n_rows, n_cols
    );

    // Try to get band descriptions if available
    debug_info.push_str(&format!("Transform: {:?}\n", raster_data.transform));
    debug_info.push_str(&format!("Projection: {:?}\n", raster_data.projection));

    // Sample some values from the first few bands
    for band in 0..(n_bands.min(10)) {
        let band_slice =
            raster_data
                .data
                .slice(ndarray::s![band, 0..5.min(n_rows), 0..5.min(n_cols)]);
        debug_info.push_str(&format!("Band {} sample values:\n{:?}\n", band, band_slice));
    }

    Ok(debug_info)
}

#[tauri::command]
async fn get_cpu_info() -> Result<CpuInfo, String> {
    let total_cores = num_cpus::get();
    let logical_cores = num_cpus::get(); // In most cases, this is the same as total cores

    Ok(CpuInfo {
        total_cores,
        logical_cores,
    })
}

#[tauri::command]
async fn get_seasonal_analysis(state: State<'_, AppState>) -> Result<SeasonalAnalysisData, String> {
    let results = state.current_results.lock().unwrap();

    match results.as_ref() {
        Some(results) => {
            // Inline seasonal analysis calculation (simplified version)
            use chrono::Datelike;
            use ndarray::Array2;
            use std::collections::HashMap;

            let (n_times, n_rows, n_cols) = results.shadow_fraction.dim();

            // Group timestamps by month-year
            let mut monthly_groups: HashMap<(u32, i32), Vec<usize>> = HashMap::new();
            for (idx, timestamp) in results.timestamps.iter().enumerate() {
                let month = timestamp.month();
                let year = timestamp.year();
                monthly_groups.entry((month, year)).or_default().push(idx);
            }

            // Calculate days per month by grouping timestamps by date
            let mut monthly_date_groups: HashMap<
                (u32, i32),
                std::collections::HashSet<chrono::NaiveDate>,
            > = HashMap::new();
            for (_idx, timestamp) in results.timestamps.iter().enumerate() {
                let month = timestamp.month();
                let year = timestamp.year();
                let date = timestamp.date_naive();
                monthly_date_groups
                    .entry((month, year))
                    .or_default()
                    .insert(date);
            }

            // Calculate monthly statistics
            let mut monthly_stats_data = Vec::new();
            for ((month, year), time_indices) in monthly_groups.iter() {
                let unique_dates = monthly_date_groups.get(&(*month, *year)).unwrap();
                let days_in_analysis = unique_dates.len() as u32;

                let mut month_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
                let mut month_solar_efficiency = Array2::<f32>::zeros((n_rows, n_cols));
                let mut month_max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));

                // Calculate statistics for each cell
                for row in 0..n_rows {
                    for col in 0..n_cols {
                        let mut total_shadow = 0.0f32;
                        let mut consecutive_shadow = 0.0f32;
                        let mut max_consecutive = 0.0f32;
                        let mut sunlit_hours = 0.0f32;

                        for &time_idx in time_indices {
                            let shadow_val = results.shadow_fraction[[time_idx, row, col]];

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
                        if time_indices.len() > 0 {
                            avg_shadow_percentage[[row, col]] = (month_shadow_hours[[row, col]]
                                / time_indices.len() as f32)
                                * 100.0;
                        }
                    }
                }

                monthly_stats_data.push(MonthlyShadowStatsData {
                    month: *month,
                    year: *year,
                    total_shadow_hours: month_shadow_hours
                        .outer_iter()
                        .map(|row| row.to_vec())
                        .collect(),
                    avg_shadow_percentage: avg_shadow_percentage
                        .outer_iter()
                        .map(|row| row.to_vec())
                        .collect(),
                    max_consecutive_shadow: month_max_consecutive
                        .outer_iter()
                        .map(|row| row.to_vec())
                        .collect(),
                    solar_efficiency_percentage: month_solar_efficiency
                        .outer_iter()
                        .map(|row| row.to_vec())
                        .collect(),
                    days_in_analysis,
                });
            }

            // Sort monthly stats by year and month
            monthly_stats_data.sort_by(|a, b| (a.year, a.month).cmp(&(b.year, b.month)));

            // Calculate seasonal summaries by aggregating monthly data
            let seasons = vec![
                ("Spring", vec![3, 4, 5]),
                ("Summer", vec![6, 7, 8]),
                ("Fall", vec![9, 10, 11]),
                ("Winter", vec![12, 1, 2]),
            ];

            let mut seasonal_summaries_data = Vec::new();
            for (season_name, season_months) in seasons {
                let season_months_set: std::collections::HashSet<u32> =
                    season_months.iter().cloned().collect();

                // Find matching monthly stats for this season
                let season_monthly_stats: Vec<&MonthlyShadowStatsData> = monthly_stats_data
                    .iter()
                    .filter(|ms| season_months_set.contains(&ms.month))
                    .collect();

                if !season_monthly_stats.is_empty() && n_rows > 0 && n_cols > 0 {
                    // Aggregate seasonal data
                    let mut season_shadow_hours = Array2::<f32>::zeros((n_rows, n_cols));
                    let mut season_shadow_percentage = Array2::<f32>::zeros((n_rows, n_cols));
                    let mut season_max_consecutive = Array2::<f32>::zeros((n_rows, n_cols));
                    let mut season_solar_efficiency = Array2::<f32>::zeros((n_rows, n_cols));
                    let mut total_days = 0;

                    for monthly_stat in &season_monthly_stats {
                        // Add to totals
                        for row in 0..n_rows {
                            for col in 0..n_cols {
                                season_shadow_hours[[row, col]] +=
                                    monthly_stat.total_shadow_hours[row][col];
                                season_shadow_percentage[[row, col]] +=
                                    monthly_stat.avg_shadow_percentage[row][col];
                                season_solar_efficiency[[row, col]] +=
                                    monthly_stat.solar_efficiency_percentage[row][col];

                                // Max consecutive is the maximum across months
                                season_max_consecutive[[row, col]] = season_max_consecutive
                                    [[row, col]]
                                .max(monthly_stat.max_consecutive_shadow[row][col]);
                            }
                        }
                        total_days += monthly_stat.days_in_analysis;
                    }

                    // Average the percentage values
                    let season_count = season_monthly_stats.len() as f32;
                    for row in 0..n_rows {
                        for col in 0..n_cols {
                            season_shadow_percentage[[row, col]] /= season_count;
                            season_solar_efficiency[[row, col]] /= season_count;
                        }
                    }

                    seasonal_summaries_data.push(SeasonStatsData {
                        season_name: season_name.to_string(),
                        months: season_months,
                        total_shadow_hours: season_shadow_hours
                            .outer_iter()
                            .map(|row| row.to_vec())
                            .collect(),
                        avg_shadow_percentage: season_shadow_percentage
                            .outer_iter()
                            .map(|row| row.to_vec())
                            .collect(),
                        max_consecutive_shadow: season_max_consecutive
                            .outer_iter()
                            .map(|row| row.to_vec())
                            .collect(),
                        solar_efficiency_percentage: season_solar_efficiency
                            .outer_iter()
                            .map(|row| row.to_vec())
                            .collect(),
                        total_days,
                    });
                } else {
                    // Empty season data
                    seasonal_summaries_data.push(SeasonStatsData {
                        season_name: season_name.to_string(),
                        months: season_months,
                        total_shadow_hours: vec![],
                        avg_shadow_percentage: vec![],
                        max_consecutive_shadow: vec![],
                        solar_efficiency_percentage: vec![],
                        total_days: 0,
                    });
                }
            }

            // Determine analysis period
            let start_time = results.timestamps.iter().min().cloned().unwrap_or_default();
            let end_time = results.timestamps.iter().max().cloned().unwrap_or_default();

            Ok(SeasonalAnalysisData {
                monthly_stats: monthly_stats_data,
                seasonal_summaries: seasonal_summaries_data,
                analysis_period: (start_time.to_rfc3339(), end_time.to_rfc3339()),
            })
        }
        None => Err("No shadow calculation results available".to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            current_config: Mutex::new(None),
            current_results: Mutex::new(None),
            raster_bounds: Mutex::new(None),
            clipped_raster_info: Mutex::new(None),
            aoi_polygon: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            load_rasters,
            calculate_shadows,
            export_results,
            get_shadow_at_time,
            get_timestamps,
            get_average_shadow_raster,
            get_all_summary_data,
            get_cpu_info,
            get_seasonal_analysis,
            validate_results_file,
            load_results_file,
            debug_tiff_structure
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
