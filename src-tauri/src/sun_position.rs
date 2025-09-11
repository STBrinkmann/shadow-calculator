use chrono::{DateTime, Datelike, Timelike, Utc};
use std::collections::HashMap;

pub struct SunCalculator {
    latitude: f64,
    longitude: f64,
    angle_precision: f64,
    cache: HashMap<(u32, u32), (f64, f64)>,
}

impl SunCalculator {
    pub fn new(latitude: f64, longitude: f64, angle_precision: f64) -> Self {
        Self {
            latitude,
            longitude,
            angle_precision,
            cache: HashMap::new(),
        }
    }

    pub fn calculate_sunrise_sunset(&self, date: &DateTime<Utc>) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        // Use civil twilight threshold (-0.833° for practical sunrise/sunset)
        let target_elevation = -0.833;
        
        // Start from solar noon and search backwards/forwards for zero crossing
        let _noon = date.date_naive().and_hms_opt(12, 0, 0)?.and_utc();
        
        let sunrise = self.find_solar_event(date, target_elevation, false)?;
        let sunset = self.find_solar_event(date, target_elevation, true)?;
        
        Some((sunrise, sunset))
    }

    fn find_solar_event(&self, date: &DateTime<Utc>, target_elevation: f64, is_sunset: bool) -> Option<DateTime<Utc>> {
        let base_date = date.date_naive().and_hms_opt(12, 0, 0)?.and_utc();
        
        // Search range: 6 hours before/after solar noon
        let search_start = if is_sunset { base_date } else { base_date - chrono::Duration::hours(6) };
        let search_end = if is_sunset { base_date + chrono::Duration::hours(6) } else { base_date };
        
        // Binary search for elevation crossing
        let mut low = search_start;
        let mut high = search_end;
        let tolerance = chrono::Duration::minutes(1);
        
        while high - low > tolerance {
            let mid = low + (high - low) / 2;
            let (_, elevation) = self.calculate_position(&mid);
            
            if is_sunset {
                if elevation > target_elevation {
                    low = mid;
                } else {
                    high = mid;
                }
            } else {
                if elevation < target_elevation {
                    low = mid;
                } else {
                    high = mid;
                }
            }
        }
        
        Some(low + (high - low) / 2)
    }

    pub fn calculate_solar_noon(&self, date: &DateTime<Utc>) -> DateTime<Utc> {
        let base_date = date.date_naive().and_hms_opt(12, 0, 0).unwrap().and_utc();
        let julian_day = self.julian_day(&base_date);
        let equation_of_time = self.equation_of_time(julian_day);
        
        // Solar noon occurs when hour angle = 0, so solar time = 12
        // Local time = solar time - equation of time - longitude correction
        let solar_noon_hours = 12.0 - equation_of_time / 60.0 - self.longitude / 15.0;
        
        let hours = solar_noon_hours.floor() as u32;
        let minutes = ((solar_noon_hours - hours as f64) * 60.0) as u32;
        
        date.date_naive().and_hms_opt(hours, minutes, 0).unwrap_or(base_date.naive_utc()).and_utc()
    }

    pub fn get_solar_hours_for_day(&self, date: &DateTime<Utc>) -> f64 {
        if let Some((sunrise, sunset)) = self.calculate_sunrise_sunset(date) {
            let duration = sunset - sunrise;
            duration.num_seconds() as f64 / 3600.0 // Convert to hours
        } else {
            // Handle polar conditions
            let (_, elevation_noon) = self.calculate_position(&date.date_naive().and_hms_opt(12, 0, 0).unwrap().and_utc());
            if elevation_noon > 0.0 {
                24.0 // Polar summer - sun never sets
            } else {
                0.0  // Polar winter - sun never rises
            }
        }
    }

    pub fn get_position(&mut self, datetime: &DateTime<Utc>) -> (f64, f64) {
        let day = datetime.ordinal();
        let hour = datetime.hour();
        let key = (day, hour);

        if let Some(&cached) = self.cache.get(&key) {
            return cached;
        }

        let (azimuth, elevation) = self.calculate_position(datetime);
        let rounded = self.round_angles(azimuth, elevation);
        self.cache.insert(key, rounded);
        rounded
    }

    fn calculate_position(&self, datetime: &DateTime<Utc>) -> (f64, f64) {
        let julian_day = self.julian_day(datetime);
        let equation_of_time = self.equation_of_time(julian_day);
        let declination = self.solar_declination(julian_day);

        let solar_time = self.solar_time(datetime, equation_of_time);
        let hour_angle = 15.0 * (solar_time - 12.0);

        let lat_rad = self.latitude.to_radians();
        let dec_rad = declination.to_radians();
        let hour_rad = hour_angle.to_radians();

        // Solar elevation
        let elevation =
            (lat_rad.sin() * dec_rad.sin() + lat_rad.cos() * dec_rad.cos() * hour_rad.cos()).asin();

        // Solar azimuth
        let azimuth = ((dec_rad.sin() * lat_rad.cos()
            - dec_rad.cos() * lat_rad.sin() * hour_rad.cos())
            / elevation.cos())
        .acos();

        let azimuth_deg = if hour_angle > 0.0 {
            360.0 - azimuth.to_degrees()
        } else {
            azimuth.to_degrees()
        };

        (azimuth_deg, elevation.to_degrees())
    }

    fn round_angles(&self, azimuth: f64, elevation: f64) -> (f64, f64) {
        let inv_precision = 1.0 / self.angle_precision;
        (
            (azimuth * inv_precision).round() * self.angle_precision,
            (elevation * inv_precision).round() * self.angle_precision,
        )
    }

    fn julian_day(&self, datetime: &DateTime<Utc>) -> f64 {
        let a = (14 - datetime.month() as i32) / 12;
        let y = datetime.year() + 4800 - a;
        let m = datetime.month() as i32 + 12 * a - 3;

        datetime.day() as f64 + (153 * m + 2) as f64 / 5.0 + 365.0 * y as f64 + (y / 4) as f64
            - (y / 100) as f64
            + (y / 400) as f64
            - 32045.0
    }

    fn equation_of_time(&self, julian_day: f64) -> f64 {
        let n = julian_day - 2451545.0;
        let l = (280.460 + 0.9856474 * n) % 360.0;
        let g = ((357.528 + 0.9856003 * n) % 360.0).to_radians();
        let lambda = (l + 1.915 * g.sin() + 0.020 * (2.0 * g).sin()).to_radians();

        4.0 * (l - 0.0057183 - lambda.to_degrees())
    }

    fn solar_declination(&self, julian_day: f64) -> f64 {
        let n = julian_day - 2451545.0;
        let l = (280.460 + 0.9856474 * n) % 360.0;
        let g = ((357.528 + 0.9856003 * n) % 360.0).to_radians();
        let lambda = (l + 1.915 * g.sin() + 0.020 * (2.0 * g).sin()).to_radians();

        let obliquity = (23.439 - 0.0000004 * n).to_radians();
        (obliquity.sin() * lambda.sin()).asin().to_degrees()
    }

    fn solar_time(&self, datetime: &DateTime<Utc>, equation_of_time: f64) -> f64 {
        let local_time = datetime.hour() as f64 + datetime.minute() as f64 / 60.0;
        local_time + equation_of_time / 60.0 + self.longitude / 15.0
    }
}
