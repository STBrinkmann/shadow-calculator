import { useState, useCallback, useEffect } from 'react';
import MapView from './components/LeafletMapView'; // or SimpleMapView
import TimeControls from './components/TimeControls';
import DualModeUpload from './components/DualModeUpload';
import MetadataDisplay from './components/MetadataDisplay';
import ResultsPanel from './components/ResultsPanel';
import ProgressModal from './components/ProgressModal';
import PerformanceSettings from './components/PerformanceSettings';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { Config, ShadowQuality, UploadMode, ResultsMetadata } from './types';

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
  // Upload mode state
  const [uploadMode, setUploadMode] = useState<UploadMode>('calculate');
  const [resultsMetadata, setResultsMetadata] = useState<ResultsMetadata | null>(null);
  
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

  const handleModeChange = useCallback((mode: UploadMode) => {
    setUploadMode(mode);
    setError('');
    setHasResults(false);
    setShadowData([]);
    setTimestamps([]);
    setResultsMetadata(null);
    setRasterBounds(null);
    setAverageShadowRaster(null);
    setAllSummaryData(null);
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

  const handleResultsSelected = useCallback(async (filePath: string, metadata: ResultsMetadata) => {
    try {
      setError('');
      setResultsMetadata(metadata);
      
      // Load the results file
      const result = await invoke<string>('load_results_file', { filePath });
      console.log('Results loaded:', result);
      
      setHasResults(true);
      
      // Set raster bounds from metadata
      setRasterBounds({
        min_lon: metadata.bounds.min_lon,
        max_lon: metadata.bounds.max_lon,
        min_lat: metadata.bounds.min_lat,
        max_lat: metadata.bounds.max_lat,
      });
      
      // Get timestamps if any
      if (metadata.total_timestamps > 0) {
        const timestampsResult = await invoke<string[]>('get_timestamps');
        setTimestamps(timestampsResult);
        
        // Load first timestamp
        const shadowData = await invoke<number[][]>('get_shadow_at_time', { timeIndex: 0 });
        setShadowData(shadowData);
      }
      
      // Load average shadow raster and summary data
      const averageRaster = await invoke<RasterData>('get_average_shadow_raster');
      setAverageShadowRaster(averageRaster);
      
      const summaryData = await invoke<AllSummaryData>('get_all_summary_data');
      setAllSummaryData(summaryData);
      
    } catch (error) {
      console.error('Failed to load results:', error);
      setError(`Failed to load results: ${error}`);
    }
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
      <header className="bg-gray-800 px-6 py-4 shadow-lg border-b border-gray-700">
        <div className="flex justify-between items-center">
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
        </div>
        
        {/* Future Analysis Tools Area */}
        <div className="mt-3 p-2 bg-gray-900 bg-opacity-50 rounded-lg border border-gray-600">
          <div className="flex items-center text-gray-400">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
            <span className="text-sm">Analysis Tools</span>
            <span className="ml-auto text-xs bg-orange-600 px-2 py-1 rounded">Coming Soon</span>
          </div>
        </div>
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
        <aside className={`w-80 bg-gray-800 overflow-y-auto border-r border-gray-700 ${isCalculating ? 'pointer-events-none opacity-75' : ''}`}>
          {/* Header Section */}
          <div className="p-4 border-b border-gray-700">
            <h2 className="text-lg font-semibold text-gray-200 mb-2">Shadow Analysis Tools</h2>
            <p className="text-xs text-gray-400">Advanced terrain shadow modeling and analysis</p>
          </div>
          
          {/* Data Input Section */}
          <div className="p-4 border-b border-gray-700">
            <DualModeUpload
              mode={uploadMode}
              onModeChange={handleModeChange}
              onFilesSelected={handleFilesSelected}
              onResultsSelected={handleResultsSelected}
              disabled={isCalculating}
            />
            
            {/* Show metadata for uploaded results */}
            {uploadMode === 'upload' && resultsMetadata && (
              <div className="mt-4">
                <MetadataDisplay metadata={resultsMetadata} />
              </div>
            )}
            
            {/* Show raster bounds for calculate mode */}
            {uploadMode === 'calculate' && rasterBounds && (
              <div className="mt-3 p-3 bg-blue-900 bg-opacity-30 rounded-lg border border-blue-600">
                <div className="flex items-center mb-2">
                  <svg className="w-4 h-4 text-blue-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-1.447-.894L15 4m0 13V4m-6 3l6-3" />
                  </svg>
                  <h4 className="text-xs font-medium text-blue-300">Raster Extent</h4>
                </div>
                <div className="text-xs text-blue-200 space-y-1">
                  <p>Lon: {rasterBounds.min_lon.toFixed(4)}° to {rasterBounds.max_lon.toFixed(4)}°</p>
                  <p>Lat: {rasterBounds.min_lat.toFixed(4)}° to {rasterBounds.max_lat.toFixed(4)}°</p>
                </div>
              </div>
            )}
          </div>
          
          {/* Analysis Configuration - Only show for calculate mode */}
          {uploadMode === 'calculate' && (
            <div className="p-4 border-b border-gray-700">
              <h3 className="text-sm font-semibold text-gray-300 mb-3 flex items-center">
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Time Configuration
              </h3>
              <TimeControls 
                onConfigChange={handleTimeConfigChange}
                config={config}
                disabled={isCalculating}
              />
            </div>
          )}
          
          {/* Advanced Settings - Only show for calculate mode */}
          {uploadMode === 'calculate' && (
            <div className="p-4 border-b border-gray-700">
              <h3 className="text-sm font-semibold text-gray-300 mb-3 flex items-center">
                <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                </svg>
                Advanced Settings
              </h3>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-2 text-gray-300">
                    Analysis Buffer
                    <span className="text-xs text-gray-500 ml-2">(meters)</span>
                  </label>
                  <input
                    type="number"
                    value={config.buffer_meters}
                    onChange={(e) => setConfig(prev => ({ ...prev, buffer_meters: Number(e.target.value) }))}
                    disabled={isCalculating}
                    className="w-full px-3 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                    style={{ color: 'white' }}
                  />
                  <p className="text-xs text-gray-500 mt-1">Extra area around AOI for shadow casting</p>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-2 text-gray-300">Solar Position Precision</label>
                  <div className="relative">
                    <select
                      value={config.angle_precision}
                      onChange={(e) => setConfig(prev => ({ ...prev, angle_precision: Number(e.target.value) }))}
                      disabled={isCalculating}
                      className="w-full px-3 py-2 pr-8 rounded-lg border border-gray-600 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                      style={{ 
                        backgroundColor: '#374151', 
                        color: '#f9fafb',
                        WebkitAppearance: 'none',
                        MozAppearance: 'none',
                        appearance: 'none'
                      }}
                    >
                      <option value={0.01} style={{ backgroundColor: '#f9fafb', color: '#111827' }}>High (0.01°)</option>
                      <option value={0.1} style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Normal (0.1°)</option>
                      <option value={0.5} style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Fast (0.5°)</option>
                      <option value={1.0} style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Draft (1.0°)</option>
                    </select>
                    <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-400">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                      </svg>
                    </div>
                  </div>
                </div>
                
                <div>
                  <label className="block text-sm font-medium mb-2 text-gray-300">Rendering Quality</label>
                  <div className="relative">
                    <select
                      value={config.shadow_quality}
                      onChange={(e) => setConfig(prev => ({ ...prev, shadow_quality: e.target.value as ShadowQuality }))}
                      disabled={isCalculating}
                      className="w-full px-3 py-2 pr-8 rounded-lg border border-gray-600 focus:ring-2 focus:ring-blue-500 focus:border-transparent focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                      style={{ 
                        backgroundColor: '#374151', 
                        color: '#f9fafb',
                        WebkitAppearance: 'none',
                        MozAppearance: 'none',
                        appearance: 'none'
                      }}
                    >
                      <option value="Fast" style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Fast</option>
                      <option value="Normal" style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Normal</option>
                      <option value="High" style={{ backgroundColor: '#f9fafb', color: '#111827' }}>High</option>
                      <option value="Scientific" style={{ backgroundColor: '#f9fafb', color: '#111827' }}>Scientific</option>
                    </select>
                    <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-400">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                      </svg>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
          
          {/* Action Buttons */}
          <div className="p-4">
            {uploadMode === 'calculate' && (
              <div className="space-y-2">
                <button
                  onClick={handleCalculate}
                  disabled={isCalculating || !config.dtm_path || !config.dsm_path || config.aoi.length === 0}
                  className="w-full py-3 px-4 bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 disabled:from-gray-600 disabled:to-gray-700 text-white font-medium rounded-lg shadow-lg transition-all duration-200 transform hover:scale-[1.01] disabled:scale-100 disabled:shadow-none"
                >
                  {isCalculating ? (
                    <div className="flex items-center justify-center">
                      <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                      </svg>
                      Analyzing Shadows...
                    </div>
                  ) : (
                    'Run Shadow Analysis'
                  )}
                </button>
                
                {(!config.dtm_path || !config.dsm_path || config.aoi.length === 0) && (
                  <div className="text-xs text-gray-500 space-y-1">
                    {!config.dtm_path && <p>• Select DTM file</p>}
                    {!config.dsm_path && <p>• Select DSM file</p>}
                    {config.aoi.length === 0 && <p>• Draw area of interest on map</p>}
                  </div>
                )}
              </div>
            )}
            
            {uploadMode === 'upload' && !hasResults && (
              <div className="p-3 bg-gray-800 rounded-lg border border-gray-600 text-center">
                <div className="flex items-center justify-center text-gray-400 mb-2">
                  <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16l-4-4m0 0l4-4m-4 4h18" />
                  </svg>
                  <span className="text-sm">Select a results file above to begin</span>
                </div>
                <p className="text-xs text-gray-500">Upload a .tif file generated by Shadow Calculator</p>
              </div>
            )}
            
            {hasResults && (
              <div className="mt-4 space-y-2">
                <div className="text-xs font-medium text-gray-400 mb-2 flex items-center">
                  <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                  Export Results
                </div>
                <button
                  onClick={() => handleExport('geotiff')}
                  disabled={isCalculating}
                  className="w-full py-2 px-3 bg-emerald-600 hover:bg-emerald-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white text-sm rounded-lg transition-colors"
                >
                  Export as GeoTIFF
                </button>
                <button
                  onClick={() => handleExport('csv')}
                  disabled={isCalculating}
                  className="w-full py-2 px-3 bg-emerald-600 hover:bg-emerald-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white text-sm rounded-lg transition-colors"
                >
                  Export as CSV
                </button>
              </div>
            )}
          </div>
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