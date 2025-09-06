use crate::types::*;
use gdal::{Dataset, DriverManager};
use gdal::raster::{ResampleAlg, Buffer};
use gdal::Metadata;
use ndarray::{Array2, Array3, s};
use geo_types::{Polygon, Coord};
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
        
        let mut data = vec![0f32; (width * height) as usize];
        band.read_into_slice(
            (0, 0),
            (width, height),
            (width, height),
            &mut data,
            Some(ResampleAlg::NearestNeighbour),
        )?;
        
        let array = Array2::from_shape_vec((height as usize, width as usize), data)
            .map_err(|e| ShadowError::Config(format!("Failed to create array: {}", e)))?;
        
        Ok(RasterData {
            data: array.insert_axis(ndarray::Axis(0)),
            transform,
            projection,
            no_data_value: no_data_value.map(|v| v as f32),
        })
    }
    
    pub fn clip_to_aoi(raster: &RasterData, aoi: &Polygon<f64>, buffer_m: f64) -> Result<RasterData, ShadowError> {
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
        
        println!("Pixel coordinates: min_col={}, min_row={}, max_col={}, max_row={}", 
                 min_col, min_row, max_col, max_row);
        
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
        let subset = raster.data.slice(s![.., min_row..max_row, min_col..max_col]).to_owned();
        
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

            let band_data = data.slice(s![band_idx, .., ..]);
            let vec_data: Vec<f32> = band_data.iter().cloned().collect();
            let buffer = Buffer::new((width, height), vec_data);
            
            band.write(
                (0, 0),
                (width, height),
                &buffer,
            )?;
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
    
    fn get_buffered_bounds(polygon: &Polygon<f64>, buffer_m: f64) -> (f64, f64, f64, f64) {
        let coords: Vec<Coord<f64>> = polygon.exterior().coords().cloned().collect();
        let min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min) - buffer_m;
        let max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max) + buffer_m;
        let min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min) - buffer_m;
        let max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max) + buffer_m;
        (min_x, min_y, max_x, max_y)
    }
}