import { useState, useCallback, useEffect } from 'react';
import MapView from './components/LeafletMapView'; // or SimpleMapView
import TimeControls from './components/TimeControls';
import FileUpload from './components/FileUpload';
import ResultsPanel from './components/ResultsPanel';
import ProgressModal from './components/ProgressModal';
import PerformanceSettings from './components/PerformanceSettings';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Config, ShadowQuality } from './types';

interface RasterBounds {
  min_lon: number;
  max_lon: number;
  min_lat: number;
  max_lat: number;
}

interface ProgressUpdate {
  progress: number;
  current_step: string;
  total_steps?: number;
  current_step_number?: number;
}

interface RasterData {
  data: number[][];
  bounds: RasterBounds;
  transform: number[];
}

interface AllSummaryData {
  total_shadow_hours: number[][];
  avg_shadow_percentage: number[][];
  max_consecutive_shadow: number[][];
  morning_shadow_hours: number[][];
  afternoon_shadow_hours: number[][];
  bounds: RasterBounds;
  transform: number[];
}

function App() {
  const [config, setConfig] = useState<Config>({
    dtm_path: '',
    dsm_path: '',
    aoi: [],
    start_date: new Date().toISOString(),
    end_date: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
    hour_interval: 1,
    buffer_meters: 50,
    angle_precision: 0.1,
    shadow_quality: 'Normal' as ShadowQuality,
  });
  
  const [isCalculating, setIsCalculating] = useState(false);
  const [hasResults, setHasResults] = useState(false);
  const [currentTimeIndex, setCurrentTimeIndex] = useState(0);
  const [shadowData, setShadowData] = useState<number[][]>([]);
  const [timestamps, setTimestamps] = useState<string[]>([]);
  const [error, setError] = useState<string>('');
  const [rasterBounds, setRasterBounds] = useState<RasterBounds | null>(null);
  const [averageShadowRaster, setAverageShadowRaster] = useState<RasterData | null>(null);
  const [allSummaryData, setAllSummaryData] = useState<AllSummaryData | null>(null);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  // Progress state
  const [progressData, setProgressData] = useState<ProgressUpdate>({
    progress: 0,
    current_step: 'Initializing...',
    total_steps: undefined,
    current_step_number: undefined,
  });

  // Listen for progress updates from the backend
  useEffect(() => {
    const unlisten = listen<ProgressUpdate>('progress-update', (event) => {
      setProgressData(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const handleFilesSelected = useCallback((dtmPath: string, dsmPath: string) => {
    setConfig(prev => ({ ...prev, dtm_path: dtmPath, dsm_path: dsmPath }));
    setError('');
    
    // Load rasters and get bounds
    invoke<RasterBounds>('load_rasters', { dtmPath, dsmPath })
      .then(bounds => {
        console.log('Raster bounds received:', bounds);
        setRasterBounds(bounds);
        setError('');
      })
      .catch(error => {
        console.error(error);
        setError(String(error));
      });
  }, []);

  const handleAOIDrawn = useCallback((coordinates: number[][]) => {
    setConfig(prev => ({ ...prev, aoi: coordinates }));
  }, []);

  const handleTimeConfigChange = useCallback((timeConfig: any) => {
    setConfig(prev => ({ ...prev, ...timeConfig }));
  }, []);

  const handleCalculate = useCallback(async () => {
    if (!config.dtm_path || !config.dsm_path || config.aoi.length === 0) {
      setError('Please provide all required inputs');
      return;
    }

    setIsCalculating(true);
    setError('');
    
    // Reset progress data
    setProgressData({
      progress: 0,
      current_step: 'Starting calculation...',
      total_steps: undefined,
      current_step_number: undefined,
    });

    try {
      const result = await invoke('calculate_shadows', { config });
      console.log(result);
      setHasResults(true);
      
      // Get timestamps
      const timestampsResult = await invoke<string[]>('get_timestamps');
      setTimestamps(timestampsResult);
      
      // Load first timestamp
      const shadowData = await invoke<number[][]>('get_shadow_at_time', { timeIndex: 0 });
      setShadowData(shadowData);
      
      // Load average shadow raster (Band 2)
      const averageRaster = await invoke<RasterData>('get_average_shadow_raster');
      setAverageShadowRaster(averageRaster);
      
      // Load all summary data for tooltips and popups
      const summaryData = await invoke<AllSummaryData>('get_all_summary_data');
      setAllSummaryData(summaryData);
    } catch (error) {
      console.error('Calculation failed:', error);
      setError(`Calculation failed: ${error}`);
    } finally {
      setIsCalculating(false);
    }
  }, [config]);

  const handleTimeChange = useCallback(async (index: number) => {
    setCurrentTimeIndex(index);
    try {
      const shadowData = await invoke<number[][]>('get_shadow_at_time', { timeIndex: index });
      setShadowData(shadowData);
    } catch (error) {
      console.error('Failed to load shadow data:', error);
      setError(`Failed to load shadow data: ${error}`);
    }
  }, []);

  const handleExport = useCallback(async (format: string) => {
    // No longer specify a path - let the backend handle it
    try {
      const result = await invoke('export_results', { 
        outputPath: '', // Empty path, backend will generate
        format 
      });
      console.log(result);
      
      // Show success message with path
      const message = String(result);
      alert(`✅ Export successful!\n\n${message}`);
    } catch (error) {
      console.error('Export failed:', error);
      setError(`Export failed: ${error}`);
    }
  }, []);

  return (
    <div className="h-screen flex flex-col bg-gray-900 text-white">
      <header className="bg-gray-800 px-6 py-4 shadow-lg border-b border-gray-700 flex justify-between items-center">
        <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
          Shadow Calculator
        </h1>
        <button
          onClick={() => setIsSettingsOpen(true)}
          className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded-md transition-colors"
          title="Performance Settings"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
        </button>
      </header>
      
      {error && (
        <div className="bg-red-600 px-4 py-2 text-white flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError('')} className="text-white hover:text-gray-200">
            ✕
          </button>
        </div>
      )}
      
      <div className="flex-1 flex overflow-hidden">
        <aside className={`w-80 bg-gray-800 p-4 overflow-y-auto border-r border-gray-700 ${isCalculating ? 'pointer-events-none opacity-75' : ''}`}>
          <FileUpload onFilesSelected={handleFilesSelected} />
          
          {rasterBounds && (
            <div className="mt-4 p-3 bg-blue-800 bg-opacity-20 rounded-md border border-blue-600">
              <p className="text-xs text-blue-300">
                <strong>Raster extent:</strong><br />
                Lon: {rasterBounds.min_lon.toFixed(4)}° to {rasterBounds.max_lon.toFixed(4)}°<br />
                Lat: {rasterBounds.min_lat.toFixed(4)}° to {rasterBounds.max_lat.toFixed(4)}°
              </p>
            </div>
          )}
          
          <div className="mt-6">
            <TimeControls 
              onConfigChange={handleTimeConfigChange}
              config={config}
              disabled={isCalculating}
            />
          </div>
          
          <div className="mt-6 space-y-4">
            <h3 className="text-lg font-semibold">Advanced Settings</h3>
            
            <div>
              <label className="block text-sm font-medium mb-2">
                Buffer (meters)
                <span className="text-xs text-gray-400 ml-2">Extra area for shadows</span>
              </label>
              <input
                type="number"
                value={config.buffer_meters}
                onChange={(e) => setConfig(prev => ({ ...prev, buffer_meters: Number(e.target.value) }))}
                disabled={isCalculating}
                className="w-full px-3 py-2 bg-gray-700 rounded-md focus:ring-2 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
              />
              <p className="text-xs text-gray-400 mt-1">
                Will be converted to degrees based on latitude
              </p>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Angle Precision (degrees)</label>
              <select
                value={config.angle_precision}
                onChange={(e) => setConfig(prev => ({ ...prev, angle_precision: Number(e.target.value) }))}
                disabled={isCalculating}
                className="w-full px-3 py-2 bg-gray-700 rounded-md focus:ring-2 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <option value={0.01}>0.01°</option>
                <option value={0.1}>0.1°</option>
                <option value={0.5}>0.5°</option>
                <option value={1.0}>1.0°</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">Shadow Quality</label>
              <select
                value={config.shadow_quality}
                onChange={(e) => setConfig(prev => ({ ...prev, shadow_quality: e.target.value as ShadowQuality }))}
                disabled={isCalculating}
                className="w-full px-3 py-2 bg-gray-700 rounded-md focus:ring-2 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <option value="Fast">Fast (Binary)</option>
                <option value="Normal">Normal (2x2 edges)</option>
                <option value="High">High (4x4 edges)</option>
                <option value="Scientific">Scientific (8x8 full)</option>
              </select>
            </div>
          </div>
          
          <button
            onClick={handleCalculate}
            disabled={isCalculating || !config.dtm_path || !config.dsm_path || config.aoi.length === 0}
            className="mt-6 w-full py-3 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 disabled:from-gray-600 disabled:to-gray-600 rounded-md font-medium transition-all duration-200 transform hover:scale-[1.02] disabled:scale-100"
          >
            {isCalculating ? 'Calculating...' : 'Calculate Shadows'}
          </button>
          
          {hasResults && (
            <div className="mt-4 space-y-2">
              <button
                onClick={() => handleExport('geotiff')}
                disabled={isCalculating}
                className="w-full py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-md transition-colors"
              >
                Export as GeoTIFF
              </button>
              <button
                onClick={() => handleExport('csv')}
                disabled={isCalculating}
                className="w-full py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed rounded-md transition-colors"
              >
                Export as CSV
              </button>
            </div>
          )}
        </aside>
        
        <main className={`flex-1 relative ${isCalculating ? 'pointer-events-none' : ''}`}>
          <MapView 
            onAOIDrawn={handleAOIDrawn}
            rasterBounds={rasterBounds}
            averageShadowRaster={averageShadowRaster}
            allSummaryData={allSummaryData}
          />
          
          {hasResults && (
            <ResultsPanel
              currentTimeIndex={currentTimeIndex}
              onTimeChange={handleTimeChange}
              shadowData={shadowData}
              timestamps={timestamps}
            />
          )}
        </main>
      </div>
      
      {/* Progress Modal */}
      <ProgressModal
        isVisible={isCalculating}
        progress={progressData.progress}
        currentStep={progressData.current_step}
        totalSteps={progressData.total_steps}
        currentStepNumber={progressData.current_step_number}
      />

      {/* Performance Settings Modal */}
      <PerformanceSettings
        config={config}
        onConfigChange={setConfig}
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />
    </div>
  );
}

export default App;