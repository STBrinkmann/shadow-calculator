import React, { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { UploadMode, ResultsMetadata } from '../types';

interface DualModeUploadProps {
  mode: UploadMode;
  onModeChange: (mode: UploadMode) => void;
  onFilesSelected?: (dtmPath: string, dsmPath: string) => void;
  onResultsSelected?: (filePath: string, metadata: ResultsMetadata) => void;
  disabled?: boolean;
}

const DualModeUpload: React.FC<DualModeUploadProps> = ({
  mode,
  onModeChange,
  onFilesSelected,
  onResultsSelected,
  disabled = false,
}) => {
  const [dtmFile, setDtmFile] = useState<string>('');
  const [dsmFile, setDsmFile] = useState<string>('');
  const [resultsFile, setResultsFile] = useState<string>('');
  const [validating, setValidating] = useState(false);
  const [error, setError] = useState<string>('');

  const selectDTMFile = useCallback(async () => {
    try {
      const selected = await open({
        title: 'Select DTM File',
        filters: [
          { name: 'Raster Files', extensions: ['tif', 'tiff'] },
        ],
      });
      
      if (selected && typeof selected === 'string') {
        setDtmFile(selected);
        setError('');
        
        // Auto-trigger loading if both files are selected
        if (dsmFile && onFilesSelected) {
          onFilesSelected(selected, dsmFile);
        }
      }
    } catch (err) {
      console.error('Error selecting DTM file:', err);
      setError('Failed to select DTM file');
    }
  }, [dsmFile, onFilesSelected]);

  const selectDSMFile = useCallback(async () => {
    try {
      const selected = await open({
        title: 'Select DSM File',
        filters: [
          { name: 'Raster Files', extensions: ['tif', 'tiff'] },
        ],
      });
      
      if (selected && typeof selected === 'string') {
        setDsmFile(selected);
        setError('');
        
        // Auto-trigger loading if both files are selected
        if (dtmFile && onFilesSelected) {
          onFilesSelected(dtmFile, selected);
        }
      }
    } catch (err) {
      console.error('Error selecting DSM file:', err);
      setError('Failed to select DSM file');
    }
  }, [dtmFile, onFilesSelected]);

  const selectResultsFile = useCallback(async () => {
    try {
      setValidating(true);
      setError('');
      
      const selected = await open({
        title: 'Select Results File',
        filters: [
          { name: 'GeoTIFF Files', extensions: ['tif', 'tiff'] },
        ],
      });
      
      if (selected && typeof selected === 'string') {
        // Validate the results file
        const metadata = await invoke<ResultsMetadata>('validate_results_file', {
          filePath: selected
        });
        
        setResultsFile(selected);
        
        if (onResultsSelected) {
          onResultsSelected(selected, metadata);
        }
      }
    } catch (err) {
      console.error('Error selecting results file:', err);
      setError(String(err));
    } finally {
      setValidating(false);
    }
  }, [onResultsSelected]);

  const getFileName = (path: string) => {
    return path.split(/[\\/]/).pop() || path;
  };

  return (
    <div className="space-y-4">
      {/* Mode Selection */}
      <div className="space-y-3">
        <h3 className="text-sm font-semibold text-gray-300 mb-3 flex items-center">
          <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          Data Input Mode
        </h3>
        
        <div className="space-y-2">
          <div className="flex items-center space-x-2">
            <input
              type="radio"
              id="calculate-mode"
              name="upload-mode"
              checked={mode === 'calculate'}
              onChange={() => onModeChange('calculate')}
              disabled={disabled}
              className="text-blue-600 focus:ring-blue-500"
            />
            <label htmlFor="calculate-mode" className="text-sm text-gray-300">
              Calculate Shadows (Upload DSM/DTM)
            </label>
          </div>
          
          <div className="flex items-center space-x-2">
            <input
              type="radio"
              id="upload-mode"
              name="upload-mode"
              checked={mode === 'upload'}
              onChange={() => onModeChange('upload')}
              disabled={disabled}
              className="text-blue-600 focus:ring-blue-500"
            />
            <label htmlFor="upload-mode" className="text-sm text-gray-300">
              Load Results (Upload .tif results file)
            </label>
          </div>
        </div>
      </div>

      {/* File Selection */}
      {mode === 'calculate' ? (
        <div className="space-y-3">
          <div>
            <label className="block text-xs font-medium text-gray-400 mb-2">
              Digital Terrain Model (DTM)
            </label>
            <div className="flex space-x-2">
              <button
                onClick={selectDTMFile}
                disabled={disabled}
                className="flex-1 px-3 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded-lg border border-gray-600 transition-colors"
              >
                {dtmFile ? 'Change DTM...' : 'Select DTM...'}
              </button>
            </div>
            {dtmFile && (
              <div className="mt-1 text-xs text-blue-300 truncate">
                {getFileName(dtmFile)}
              </div>
            )}
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-400 mb-2">
              Digital Surface Model (DSM)
            </label>
            <div className="flex space-x-2">
              <button
                onClick={selectDSMFile}
                disabled={disabled}
                className="flex-1 px-3 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded-lg border border-gray-600 transition-colors"
              >
                {dsmFile ? 'Change DSM...' : 'Select DSM...'}
              </button>
            </div>
            {dsmFile && (
              <div className="mt-1 text-xs text-blue-300 truncate">
                {getFileName(dsmFile)}
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <div>
            <label className="block text-xs font-medium text-gray-400 mb-2">
              Results File (.tif)
            </label>
            <div className="flex space-x-2">
              <button
                onClick={selectResultsFile}
                disabled={disabled || validating}
                className="flex-1 px-3 py-2 bg-gray-700 hover:bg-gray-600 disabled:bg-gray-800 disabled:cursor-not-allowed text-white text-sm rounded-lg border border-gray-600 transition-colors"
              >
                {validating ? (
                  <div className="flex items-center justify-center">
                    <svg className="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Validating...
                  </div>
                ) : resultsFile ? 'Change Results...' : 'Select Results...'}
              </button>
            </div>
            {resultsFile && (
              <div className="mt-1 text-xs text-blue-300 truncate">
                {getFileName(resultsFile)}
              </div>
            )}
          </div>
        </div>
      )}

      {error && (
        <div className="p-2 bg-red-900 bg-opacity-50 border border-red-600 rounded-lg">
          <p className="text-xs text-red-300">{error}</p>
        </div>
      )}
      
      {/* Status */}
      {mode === 'calculate' && dtmFile && dsmFile && (
        <div className="p-2 bg-green-900 bg-opacity-30 border border-green-600 rounded-lg">
          <p className="text-xs text-green-300">✓ Ready for shadow calculation</p>
        </div>
      )}
      
      {mode === 'upload' && resultsFile && (
        <div className="p-2 bg-green-900 bg-opacity-30 border border-green-600 rounded-lg">
          <p className="text-xs text-green-300">✓ Results loaded and ready for visualization</p>
        </div>
      )}
    </div>
  );
};

export default DualModeUpload;