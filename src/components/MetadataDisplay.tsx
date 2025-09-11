import React from 'react';
import { format } from 'date-fns';
import { ResultsMetadata } from '../types';

interface MetadataDisplayProps {
  metadata: ResultsMetadata;
}

const MetadataDisplay: React.FC<MetadataDisplayProps> = ({ metadata }) => {
  const formatDate = (dateStr: string) => {
    try {
      return format(new Date(dateStr), 'MMM dd, yyyy HH:mm');
    } catch {
      return dateStr;
    }
  };

  const formatInterval = (hours: number) => {
    if (hours >= 24) {
      return `${hours / 24} day${hours / 24 !== 1 ? 's' : ''}`;
    } else if (hours >= 1) {
      return `${hours} hour${hours !== 1 ? 's' : ''}`;
    } else {
      return `${Math.round(hours * 60)} minutes`;
    }
  };

  return (
    <div className="p-3 bg-purple-900 bg-opacity-30 rounded-lg border border-purple-600">
      <div className="flex items-center mb-2">
        <svg className="w-4 h-4 text-purple-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
        <h4 className="text-xs font-medium text-purple-300">Results Metadata</h4>
      </div>
      
      <div className="space-y-2 text-xs text-purple-200">
        <div className="grid grid-cols-2 gap-2">
          <div>
            <span className="text-purple-400">Period:</span>
            <div className="font-mono">{formatDate(metadata.start_date)}</div>
            <div className="font-mono">to {formatDate(metadata.end_date)}</div>
          </div>
          
          <div>
            <span className="text-purple-400">Frequency:</span>
            <div>{formatInterval(metadata.hour_interval)}</div>
            <span className="text-purple-400">Timestamps:</span>
            <div>{metadata.total_timestamps.toLocaleString()}</div>
          </div>
        </div>
        
        <div>
          <span className="text-purple-400">Extent:</span>
          <div className="font-mono text-xs">
            Lon: {metadata.bounds.min_lon.toFixed(4)}° to {metadata.bounds.max_lon.toFixed(4)}°
          </div>
          <div className="font-mono text-xs">
            Lat: {metadata.bounds.min_lat.toFixed(4)}° to {metadata.bounds.max_lat.toFixed(4)}°
          </div>
        </div>
        
        <div>
          <span className="text-purple-400">Summary Layers:</span>
          <div className="mt-1 space-y-1">
            {metadata.summary_layers.map((layer, index) => (
              <div key={index} className="text-xs">
                • {layer}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default MetadataDisplay;