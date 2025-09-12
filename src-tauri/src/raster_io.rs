use crate::types::*;
use chrono::{Datelike, Timelike};
use gdal::raster::{Buffer, ResampleAlg};
use gdal::Metadata;
use gdal::{Dataset, DriverManager};
use geo_types::{Coord, Polygon};
use ndarray::{s, Array2, Array3};
use std::path::Path;

pub struct RasterIO;

impl RasterIO {
    pub fn read_raster(path: &Path) -> Result<RasterData, ShadowError> {
        let dataset = Dataset::open(path)?;
        let transform = dataset.geo_transform()?;
        let projection = dataset.projection();

        let band = dataset.rasterband(1)?;
        let no_data_value = band.no_data_value();
        let (width, height) = band.size();

        let mut data = vec![0f32; width * height];
        band.read_into_slice(
            (0, 0),
            (width, height),
            (width, height),
            &mut data,
            Some(ResampleAlg::NearestNeighbour),
        )?;

        let array = Array2::from_shape_vec((height, width), data)
            .map_err(|e| ShadowError::Config(format!("Failed to create array: {}", e)))?;

        Ok(RasterData {
            data: array.insert_axis(ndarray::Axis(0)),
            transform,
            projection,
            no_data_value: no_data_value.map(|v| v as f32),
        })
    }

    pub fn read_multiband_raster(path: &Path) -> Result<RasterData, ShadowError> {
        let dataset = Dataset::open(path)?;
        let transform = dataset.geo_transform()?;
        let projection = dataset.projection();

        let n_bands = dataset.raster_count() as usize;
        if n_bands == 0 {
            return Err(ShadowError::Config("No bands found in raster".to_string()));
        }

        // Get dimensions from first band
        let band = dataset.rasterband(1)?;
        let no_data_value = band.no_data_value();
        let (width, height) = band.size();

        // Create 3D array to hold all bands
        let mut all_data = Array3::<f32>::zeros((n_bands, height, width));

        // Read each band
        for band_idx in 0..n_bands {
            let band = dataset.rasterband((band_idx + 1) as isize)?;
            let mut band_data = vec![0f32; width * height];

            band.read_into_slice(
                (0, 0),
                (width, height),
                (width, height),
                &mut band_data,
                Some(ResampleAlg::NearestNeighbour),
            )?;

            let band_array = Array2::from_shape_vec((height, width), band_data).map_err(|e| {
                ShadowError::Config(format!(
                    "Failed to create array for band {}: {}",
                    band_idx + 1,
                    e
                ))
            })?;

            all_data.slice_mut(s![band_idx, .., ..]).assign(&band_array);
        }

        Ok(RasterData {
            data: all_data,
            transform,
            projection,
            no_data_value: no_data_value.map(|v| v as f32),
        })
    }

    pub fn read_multiband_raster_with_descriptions(
        path: &Path,
    ) -> Result<(RasterData, Vec<String>), ShadowError> {
        let dataset = Dataset::open(path)?;
        let transform = dataset.geo_transform()?;
        let projection = dataset.projection();

        let n_bands = dataset.raster_count() as usize;
        if n_bands == 0 {
            return Err(ShadowError::Config("No bands found in raster".to_string()));
        }

        // Get dimensions from first band
        let band = dataset.rasterband(1)?;
        let no_data_value = band.no_data_value();
        let (width, height) = band.size();

        // Create 3D array to hold all bands
        let mut all_data = Array3::<f32>::zeros((n_bands, height, width));
        let mut band_descriptions = Vec::new();

        // Read each band and its description
        for band_idx in 0..n_bands {
            let band = dataset.rasterband((band_idx + 1) as isize)?;
            let mut band_data = vec![0f32; width * height];

            band.read_into_slice(
                (0, 0),
                (width, height),
                (width, height),
                &mut band_data,
                Some(ResampleAlg::NearestNeighbour),
            )?;

            let band_array = Array2::from_shape_vec((height, width), band_data).map_err(|e| {
                ShadowError::Config(format!(
                    "Failed to create array for band {}: {}",
                    band_idx + 1,
                    e
                ))
            })?;

            all_data.slice_mut(s![band_idx, .., ..]).assign(&band_array);

            // Try to read band description
            let description = band
                .description()
                .unwrap_or(format!("Band_{}", band_idx + 1));
            band_descriptions.push(description);
        }

        Ok((
            RasterData {
                data: all_data,
                transform,
                projection,
                no_data_value: no_data_value.map(|v| v as f32),
            },
            band_descriptions,
        ))
    }

    pub fn clip_to_aoi(
        raster: &RasterData,
        aoi: &Polygon<f64>,
        buffer_m: f64,
    ) -> Result<RasterData, ShadowError> {
        let transform = &raster.transform;

        // For WGS84 rasters, buffer_m is already in degrees (converted in main.rs)
        // For projected rasters, we'd need to handle differently
        // For now, assume the buffer is already in the correct units

        let inv_transform = Self::invert_transform(transform);

        // Get AOI bounds with buffer
        let bounds = Self::get_buffered_bounds(aoi, buffer_m);

        println!("Clipping bounds: {:?}", bounds);

        // Convert to pixel coordinates
        let (min_col, min_row) = Self::world_to_pixel(bounds.0, bounds.3, &inv_transform);
        let (max_col, max_row) = Self::world_to_pixel(bounds.2, bounds.1, &inv_transform);

        println!(
            "Pixel coordinates: min_col={}, min_row={}, max_col={}, max_row={}",
            min_col, min_row, max_col, max_row
        );

        // Ensure within raster bounds
        let (height, width) = (raster.data.shape()[1], raster.data.shape()[2]);

        // Check if coordinates are valid
        if min_col > width as i32 || min_row > height as i32 || max_col < 0 || max_row < 0 {
            return Err(ShadowError::Config(format!(
                "AOI is outside raster bounds. Pixel coords: ({},{}) to ({},{}), Raster size: {}x{}",
                min_col, min_row, max_col, max_row, width, height
            )));
        }

        let min_row = min_row.max(0) as usize;
        let min_col = min_col.max(0) as usize;
        let max_row = (max_row.min(height as i32 - 1) as usize) + 1;
        let max_col = (max_col.min(width as i32 - 1) as usize) + 1;

        // Check if we have a valid region
        if max_row <= min_row || max_col <= min_col {
            return Err(ShadowError::Config(format!(
                "Invalid clip region: rows {}-{}, cols {}-{}",
                min_row, max_row, min_col, max_col
            )));
        }

        // Extract subset
        let subset = raster
            .data
            .slice(s![.., min_row..max_row, min_col..max_col])
            .to_owned();

        // Update transform for subset
        let mut new_transform = *transform;
        new_transform[0] = transform[0] + min_col as f64 * transform[1];
        new_transform[3] = transform[3] + min_row as f64 * transform[5];

        Ok(RasterData {
            data: subset,
            transform: new_transform,
            projection: raster.projection.clone(),
            no_data_value: raster.no_data_value,
        })
    }

    #[allow(dead_code)]
    pub fn write_geotiff(
        path: &Path,
        data: &Array3<f32>,
        transform: &[f64; 6],
        projection: &str,
    ) -> Result<(), ShadowError> {
        Self::write_geotiff_with_descriptions(path, data, transform, projection, &[])
    }

    pub fn write_geotiff_with_descriptions(
        path: &Path,
        data: &Array3<f32>,
        transform: &[f64; 6],
        projection: &str,
        band_descriptions: &[String],
    ) -> Result<(), ShadowError> {
        // Use DriverManager instead of deprecated Driver::get_by_name
        let driver = DriverManager::get_driver_by_name("GTiff")?;
        let (n_bands, height, width) = data.dim();

        let mut dataset = driver.create_with_band_type::<f32, _>(
            path,
            width as isize,
            height as isize,
            n_bands as isize,
        )?;

        dataset.set_geo_transform(transform)?;
        dataset.set_projection(projection)?;

        for band_idx in 0..n_bands {
            let mut band = dataset.rasterband((band_idx + 1) as isize)?;

            // Set band description if available
            if band_idx < band_descriptions.len() {
                band.set_description(&band_descriptions[band_idx])?;
            }

            // Set NoData value for proper masking
            band.set_no_data_value(Some(f32::NAN as f64))?;

            let band_data = data.slice(s![band_idx, .., ..]);
            let vec_data: Vec<f32> = band_data.iter().cloned().collect();
            let buffer = Buffer::new((width, height), vec_data);

            band.write((0, 0), (width, height), &buffer)?;
        }

        Ok(())
    }

    pub fn write_csv_with_aoi_mask(
        path: &Path,
        shadow_data: &Array3<f32>,
        timestamps: &[chrono::DateTime<chrono::Utc>],
        transform: &[f64; 6],
        aoi: &Polygon<f64>,
    ) -> Result<(), ShadowError> {
        use geo::algorithm::contains::Contains;
        use geo_types::Coord;
        use std::io::Write;

        let mut file = std::fs::File::create(path)?;
        writeln!(file, "cell_id,lat,lon,datetime,shadow_fraction")?;

        let (n_times, n_rows, n_cols) = shadow_data.dim();
        let mut cell_id = 0;

        for row in 0..n_rows {
            for col in 0..n_cols {
                let (lon, lat) = Self::pixel_to_world(col, row, transform);
                let point = Coord { x: lon, y: lat };

                // Only export cells inside AOI
                if aoi.contains(&point) {
                    for t_idx in 0..n_times {
                        let shadow_val = shadow_data[[t_idx, row, col]];
                        writeln!(
                            file,
                            "{},{:.6},{:.6},{},{}",
                            cell_id,
                            lat,
                            lon,
                            timestamps[t_idx].to_rfc3339(),
                            shadow_val
                        )?;
                    }
                    cell_id += 1;
                }
            }
        }

        Ok(())
    }

    pub fn write_csv(
        path: &Path,
        shadow_data: &Array3<f32>,
        timestamps: &[chrono::DateTime<chrono::Utc>],
        transform: &[f64; 6],
    ) -> Result<(), ShadowError> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;

        writeln!(file, "cell_id,lat,lon,datetime,shadow_fraction")?;

        let (n_times, n_rows, n_cols) = shadow_data.dim();
        let mut cell_id = 0;

        for row in 0..n_rows {
            for col in 0..n_cols {
                let (lon, lat) = Self::pixel_to_world(col, row, transform);

                for t_idx in 0..n_times {
                    let shadow_val = shadow_data[[t_idx, row, col]];
                    writeln!(
                        file,
                        "{},{:.6},{:.6},{},{}",
                        cell_id,
                        lat,
                        lon,
                        timestamps[t_idx].to_rfc3339(),
                        shadow_val
                    )?;
                }
                cell_id += 1;
            }
        }

        Ok(())
    }

    fn invert_transform(transform: &[f64; 6]) -> [f64; 6] {
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

    fn world_to_pixel(x: f64, y: f64, inv_transform: &[f64; 6]) -> (i32, i32) {
        let col = inv_transform[0] + inv_transform[1] * x + inv_transform[2] * y;
        let row = inv_transform[3] + inv_transform[4] * x + inv_transform[5] * y;
        (col.round() as i32, row.round() as i32)
    }

    pub fn pixel_to_world(col: usize, row: usize, transform: &[f64; 6]) -> (f64, f64) {
        let x = transform[0] + col as f64 * transform[1] + row as f64 * transform[2];
        let y = transform[3] + col as f64 * transform[4] + row as f64 * transform[5];
        (x, y)
    }

    pub fn calculate_automatic_buffer(
        dtm: &RasterData,
        dsm: &RasterData,
        aoi: &Polygon<f64>,
        start_date: &str,
        end_date: &str,
    ) -> Result<f64, ShadowError> {
        use chrono::{DateTime, Utc};

        // Parse dates
        let start_dt = DateTime::parse_from_rfc3339(start_date)
            .map_err(|e| ShadowError::Config(format!("Invalid start date: {}", e)))?
            .with_timezone(&Utc);
        let end_dt = DateTime::parse_from_rfc3339(end_date)
            .map_err(|e| ShadowError::Config(format!("Invalid end date: {}", e)))?
            .with_timezone(&Utc);

        // Get AOI center for solar calculations
        let center_lat = aoi.exterior().coords().map(|c| c.y).sum::<f64>()
            / aoi.exterior().coords().count() as f64;
        let center_lon = aoi.exterior().coords().map(|c| c.x).sum::<f64>()
            / aoi.exterior().coords().count() as f64;

        // Calculate terrain height difference around AOI
        let max_height_diff = Self::calculate_max_height_difference_around_aoi(dtm, dsm, aoi)?;

        // Calculate minimum solar elevation during analysis period
        let min_solar_elevation =
            Self::calculate_min_solar_elevation(center_lat, center_lon, start_dt, end_dt)?;

        // Calculate maximum shadow length using trigonometry
        // shadow_length = height / tan(elevation_angle)
        let max_shadow_length = if min_solar_elevation > 0.1 {
            max_height_diff / min_solar_elevation.to_radians().tan()
        } else {
            // For very low sun angles, use a reasonable maximum
            2000.0
        };

        // Apply safety factor and bounds
        let safety_factor = 1.2;
        let buffer = (max_shadow_length * safety_factor).max(50.0).min(5000.0);

        println!("Automatic buffer calculation:");
        println!("  Max height difference: {:.1}m", max_height_diff);
        println!("  Min solar elevation: {:.1}°", min_solar_elevation);
        println!("  Max shadow length: {:.1}m", max_shadow_length);
        println!("  Buffer with safety factor: {:.1}m", buffer);

        Ok(buffer)
    }

    fn calculate_max_height_difference_around_aoi(
        dtm: &RasterData,
        dsm: &RasterData,
        aoi: &Polygon<f64>,
    ) -> Result<f64, ShadowError> {
        use geo::algorithm::contains::Contains;
        use geo_types::Coord;

        // Get AOI bounds for search area
        let coords: Vec<Coord<f64>> = aoi.exterior().coords().cloned().collect();
        let min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
        let max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
        let max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max);

        // Expand search area by a reasonable amount (500m equivalent in degrees)
        let search_buffer = 0.005; // Rough approximation

        let mut max_height = f32::NEG_INFINITY;
        let mut min_terrain_in_aoi = f32::INFINITY;

        // Sample heights in and around AOI
        for row in 0..dtm.data.shape()[1] {
            for col in 0..dtm.data.shape()[2] {
                let (world_x, world_y) = Self::pixel_to_world(col, row, &dtm.transform);

                // Check if within expanded search area
                if world_x >= min_x - search_buffer
                    && world_x <= max_x + search_buffer
                    && world_y >= min_y - search_buffer
                    && world_y <= max_y + search_buffer
                {
                    let dtm_height = dtm.data[[0, row, col]];
                    let dsm_height = dsm.data[[0, row, col]];

                    if dtm_height.is_finite() && dsm_height.is_finite() {
                        // Track maximum DSM height around AOI
                        max_height = max_height.max(dsm_height);

                        // Track minimum terrain height inside AOI
                        let point = Coord {
                            x: world_x,
                            y: world_y,
                        };
                        if aoi.contains(&point) {
                            min_terrain_in_aoi = min_terrain_in_aoi.min(dtm_height);
                        }
                    }
                }
            }
        }

        // Calculate maximum possible height difference
        let height_diff = if min_terrain_in_aoi.is_finite() && max_height.is_finite() {
            (max_height - min_terrain_in_aoi) as f64
        } else {
            100.0 // Default fallback
        };

        Ok(height_diff.max(10.0)) // Minimum 10m for safety
    }

    fn calculate_min_solar_elevation(
        lat: f64,
        lon: f64,
        start_dt: chrono::DateTime<chrono::Utc>,
        end_dt: chrono::DateTime<chrono::Utc>,
    ) -> Result<f64, ShadowError> {
        let mut sun_calc = crate::sun_position::SunCalculator::new(lat, lon, 0.1);
        let mut min_elevation: f64 = 90.0;

        // Sample throughout the analysis period
        let mut current = start_dt;
        let step = chrono::Duration::days(7); // Sample weekly

        while current <= end_dt {
            // Check winter solstice period (lowest sun angles)
            let winter_check = if lat > 0.0 {
                // Northern hemisphere: check around December 21
                current
                    .with_month(12)
                    .unwrap_or(current)
                    .with_day(21)
                    .unwrap_or(current)
            } else {
                // Southern hemisphere: check around June 21
                current
                    .with_month(6)
                    .unwrap_or(current)
                    .with_day(21)
                    .unwrap_or(current)
            };

            // Sample at different times of day
            for hour in [8, 10, 12, 14, 16] {
                let check_time = winter_check
                    .with_hour(hour as u32)
                    .unwrap_or(winter_check)
                    .with_minute(0)
                    .unwrap_or(winter_check)
                    .with_second(0)
                    .unwrap_or(winter_check);

                if check_time >= start_dt && check_time <= end_dt {
                    let (_, elevation) = sun_calc.get_position(&check_time);
                    if elevation > 0.0 && elevation < 90.0 {
                        min_elevation = min_elevation.min(elevation);
                    }
                }
            }

            current = current + step;
        }

        // Also check the actual start and end dates
        for check_date in [start_dt, end_dt] {
            for hour in [8, 10, 12, 14, 16] {
                let check_time = check_date
                    .with_hour(hour as u32)
                    .unwrap_or(check_date)
                    .with_minute(0)
                    .unwrap_or(check_date)
                    .with_second(0)
                    .unwrap_or(check_date);

                let (_, elevation) = sun_calc.get_position(&check_time);
                if elevation > 0.0 && elevation < 90.0 {
                    min_elevation = min_elevation.min(elevation);
                }
            }
        }

        Ok(min_elevation.min(89.0)) // Cap at reasonable maximum
    }

    pub fn mask_results_to_aoi(
        data: &mut Array3<f32>,
        aoi: &Polygon<f64>,
        transform: &[f64; 6],
        no_data_value: f32,
    ) -> Result<(), ShadowError> {
        use geo::algorithm::contains::Contains;
        use geo_types::Coord;

        let (n_bands, n_rows, n_cols) = data.dim();

        for band in 0..n_bands {
            for row in 0..n_rows {
                for col in 0..n_cols {
                    let (world_x, world_y) = Self::pixel_to_world(col, row, transform);
                    let point = Coord {
                        x: world_x,
                        y: world_y,
                    };

                    // Set values outside AOI to NoData
                    if !aoi.contains(&point) {
                        data[[band, row, col]] = no_data_value;
                    }
                }
            }
        }

        Ok(())
    }

    fn get_buffered_bounds(polygon: &Polygon<f64>, buffer_m: f64) -> (f64, f64, f64, f64) {
        let coords: Vec<Coord<f64>> = polygon.exterior().coords().cloned().collect();
        let min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min) - buffer_m;
        let max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max) + buffer_m;
        let min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min) - buffer_m;
        let max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max) + buffer_m;
        (min_x, min_y, max_x, max_y)
    }
}
