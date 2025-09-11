export type ShadowQuality = 'Fast' | 'Normal' | 'High' | 'Scientific';

export interface Config {
  dtm_path: string;
  dsm_path: string;
  aoi: number[][];
  start_date: string;
  end_date: string;
  hour_interval: number;
  buffer_meters: number;
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

export type UploadMode = 'calculate' | 'upload';