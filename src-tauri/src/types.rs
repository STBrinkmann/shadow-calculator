use chrono::{DateTime, Utc};
use geo_types::Polygon;
use ndarray::Array3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub dtm_path: String,
    pub dsm_path: String,
    pub aoi: Vec<Vec<f64>>, // Simplified for JSON serialization
    pub start_date: String,
    pub end_date: String,
    pub hour_interval: f32,
    pub buffer_meters: f64,
    pub angle_precision: f64,
    pub shadow_quality: ShadowQuality,
}

impl Config {
    pub fn to_polygon(&self) -> Result<Polygon<f64>, String> {
        if self.aoi.is_empty() {
            return Err("AOI is empty".to_string());
        }
        
        let coords: Vec<(f64, f64)> = self.aoi
            .iter()
            .map(|coord| {
                if coord.len() >= 2 {
                    (coord[0], coord[1])
                } else {
                    (0.0, 0.0)
                }
            })
            .collect();
        
        Ok(Polygon::new(
            geo_types::LineString::from(coords),
            vec![],
        ))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShadowQuality {
    Fast,
    Normal,
    High,
    Scientific,
}

#[derive(Debug, Clone)]
pub struct RasterData {
    pub data: Array3<f32>,
    pub transform: [f64; 6],
    pub projection: String,
    pub no_data_value: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunPosition {
    pub azimuth: f64,
    pub elevation: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ShadowResult {
    pub shadow_fraction: Array3<f32>,
    pub timestamps: Vec<DateTime<Utc>>,
    pub summary_stats: SummaryStats,
}

#[derive(Debug, Clone)]
pub struct SummaryStats {
    pub total_shadow_hours: Array3<f32>,
    pub avg_shadow_percentage: Array3<f32>,
    pub max_consecutive_shadow: Array3<f32>,
    pub morning_shadow_hours: Array3<f32>,
    pub afternoon_shadow_hours: Array3<f32>,
}

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ShadowError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GDAL error: {0}")]
    Gdal(#[from] gdal::errors::GdalError),
    #[error("Invalid configuration: {0}")]
    Config(String),
    #[allow(dead_code)]
    #[error("General error: {0}")]
    General(String),
}