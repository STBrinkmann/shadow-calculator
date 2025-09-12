export type ShadowQuality = 'Fast' | 'Normal' | 'High' | 'Scientific';

export interface Config {
  dtm_path: string;
  dsm_path: string;
  aoi: number[][];
  start_date: string;
  end_date: string;
  hour_interval: number;
  angle_precision: number;
  shadow_quality: ShadowQuality;
  cpu_cores?: number; // Optional number of CPU cores to use
}

export interface TimeConfig {
  start_date: string;
  end_date: string;
  hour_interval: number;
}

export interface CpuInfo {
  total_cores: number;
  logical_cores: number;
}

export interface ResultsMetadata {
  start_date: string;
  end_date: string;
  hour_interval: number;
  total_timestamps: number;
  summary_layers: string[];
  bounds: {
    min_lon: number;
    max_lon: number;
    min_lat: number;
    max_lat: number;
  };
}

export interface MonthlyShadowStats {
  month: number;
  year: number;
  total_shadow_hours: number[][];
  avg_shadow_percentage: number[][];
  max_consecutive_shadow: number[][];
  solar_efficiency_percentage: number[][];
  days_in_analysis: number;
}

export interface SeasonStats {
  season_name: string;
  months: number[];
  total_shadow_hours: number[][];
  avg_shadow_percentage: number[][];
  max_consecutive_shadow: number[][];
  solar_efficiency_percentage: number[][];
  total_days: number;
}

export interface SeasonalAnalysis {
  monthly_stats: MonthlyShadowStats[];
  seasonal_summaries: SeasonStats[];
  analysis_period: [string, string]; // ISO 8601 datetime strings
}

export type UploadMode = 'calculate' | 'upload';