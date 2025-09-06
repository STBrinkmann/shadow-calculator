import React, { useCallback } from 'react';
import { open } from '@tauri-apps/api/dialog';
import { DocumentArrowUpIcon, CheckCircleIcon } from '@heroicons/react/24/outline';

interface FileUploadProps {
  onFilesSelected: (dtmPath: string, dsmPath: string) => void;
}

const FileUpload: React.FC<FileUploadProps> = ({ onFilesSelected }) => {
  const [dtmPath, setDtmPath] = React.useState<string>('');
  const [dsmPath, setDsmPath] = React.useState<string>('');

  const selectFile = useCallback(async (type: 'dtm' | 'dsm') => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Raster files',
          extensions: ['tif', 'tiff', 'img', 'hgt', 'asc']
        }]
      });

      if (selected && typeof selected === 'string') {
        if (type === 'dtm') {
          setDtmPath(selected);
          if (dsmPath) onFilesSelected(selected, dsmPath);
        } else {
          setDsmPath(selected);
          if (dtmPath) onFilesSelected(dtmPath, selected);
        }
      }
    } catch (error) {
      console.error('Error selecting file:', error);
    }
  }, [dtmPath, dsmPath, onFilesSelected]);

  const getFileName = (path: string) => {
    if (!path) return '';
    const parts = path.split(/[/\\]/);
    return parts[parts.length - 1];
  };

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center">
        <DocumentArrowUpIcon className="h-5 w-5 mr-2" />
        Input Data
      </h3>
      
      <div className="space-y-3">
        <div>
          <label className="block text-sm font-medium mb-2 text-gray-300">
            Digital Terrain Model (DTM)
          </label>
          <button
            onClick={() => selectFile('dtm')}
            className={`w-full px-4 py-3 rounded-md flex items-center justify-between transition-all duration-200 ${
              dtmPath 
                ? 'bg-green-900 bg-opacity-30 hover:bg-opacity-40 border border-green-600' 
                : 'bg-gray-700 hover:bg-gray-600 border border-gray-600'
            }`}
          >
            <span className="flex items-center">
              {dtmPath ? (
                <>
                  <CheckCircleIcon className="h-5 w-5 mr-2 text-green-400" />
                  <span className="text-sm truncate">{getFileName(dtmPath)}</span>
                </>
              ) : (
                <>
                  <DocumentArrowUpIcon className="h-5 w-5 mr-2" />
                  <span className="text-sm">Select DTM file</span>
                </>
              )}
            </span>
          </button>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2 text-gray-300">
            Digital Surface Model (DSM)
          </label>
          <button
            onClick={() => selectFile('dsm')}
            className={`w-full px-4 py-3 rounded-md flex items-center justify-between transition-all duration-200 ${
              dsmPath 
                ? 'bg-green-900 bg-opacity-30 hover:bg-opacity-40 border border-green-600' 
                : 'bg-gray-700 hover:bg-gray-600 border border-gray-600'
            }`}
          >
            <span className="flex items-center">
              {dsmPath ? (
                <>
                  <CheckCircleIcon className="h-5 w-5 mr-2 text-green-400" />
                  <span className="text-sm truncate">{getFileName(dsmPath)}</span>
                </>
              ) : (
                <>
                  <DocumentArrowUpIcon className="h-5 w-5 mr-2" />
                  <span className="text-sm">Select DSM file</span>
                </>
              )}
            </span>
          </button>
        </div>
      </div>

      {dtmPath && dsmPath && (
        <div className="p-3 bg-green-800 bg-opacity-20 rounded-md border border-green-600">
          <p className="text-sm text-green-400 flex items-center">
            <CheckCircleIcon className="h-4 w-4 mr-2" />
            Both rasters loaded successfully
          </p>
        </div>
      )}

      <div className="text-xs text-gray-400 space-y-1">
        <p>• DTM: Ground elevation (terrain only)</p>
        <p>• DSM: Surface elevation (includes buildings/trees)</p>
        <p>• Both files must have the same resolution</p>
      </div>
    </div>
  );
};

export default FileUpload;