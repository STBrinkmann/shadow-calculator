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
}

export interface TimeConfig {
  start_date: string;
  end_date: string;
  hour_interval: number;
}