import React, { useState, useEffect } from 'react';
import { PlayIcon, PauseIcon, ForwardIcon, BackwardIcon } from '@heroicons/react/24/solid';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { format, parseISO } from 'date-fns';

interface ResultsPanelProps {
  currentTimeIndex: number;
  onTimeChange: (index: number) => void;
  shadowData: number[][];
  timestamps: string[];
}

const ResultsPanel: React.FC<ResultsPanelProps> = ({ 
  currentTimeIndex, 
  onTimeChange, 
  shadowData,
  timestamps
}) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [playbackSpeed, setPlaybackSpeed] = useState(500); // ms between frames
  const [stats, setStats] = useState({
    avgShadow: 0,
    maxShadow: 0,
    minShadow: 0,
    shadowedCells: 0,
    totalCells: 0,
  });

  const maxTimeIndex = timestamps.length || 1;

  useEffect(() => {
    if (!shadowData || shadowData.length === 0) return;

    // Calculate statistics
    let sum = 0;
    let count = 0;
    let shadowedCount = 0;
    let max = 0;
    let min = 1;

    for (const row of shadowData) {
      for (const value of row) {
        sum += value;
        count++;
        if (value > 0.5) shadowedCount++;
        max = Math.max(max, value);
        min = Math.min(min, value);
      }
    }

    setStats({
      avgShadow: count > 0 ? sum / count : 0,
      maxShadow: max,
      minShadow: min,
      shadowedCells: shadowedCount,
      totalCells: count,
    });
  }, [shadowData]);

  useEffect(() => {
    if (!isPlaying) return;

    const interval = setInterval(() => {
      onTimeChange((currentTimeIndex + 1) % maxTimeIndex);
    }, playbackSpeed);

    return () => clearInterval(interval);
  }, [isPlaying, currentTimeIndex, maxTimeIndex, onTimeChange, playbackSpeed]);

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleStepBackward = () => {
    setIsPlaying(false);
    onTimeChange(Math.max(0, currentTimeIndex - 1));
  };

  const handleStepForward = () => {
    setIsPlaying(false);
    onTimeChange(Math.min(maxTimeIndex - 1, currentTimeIndex + 1));
  };

  const getCurrentTimestamp = () => {
    if (timestamps && timestamps[currentTimeIndex]) {
      try {
        return format(parseISO(timestamps[currentTimeIndex]), 'MMM dd, yyyy HH:mm');
      } catch {
        return timestamps[currentTimeIndex];
      }
    }
    return '';
  };

  // Sample data for the time series chart (would be real data in production)
  const chartData = Array.from({ length: Math.min(24, maxTimeIndex) }, (_, i) => ({
    hour: i,
    shadow: Math.sin(i / 24 * Math.PI) * 0.5 + 0.2,
  }));

  return (
    <div className="absolute bottom-0 left-0 right-0 bg-gray-800 bg-opacity-95 backdrop-blur-sm p-4 border-t border-gray-700">
      <div className="max-w-7xl mx-auto">
        {/* Current timestamp display */}
        <div className="text-center mb-3">
          <p className="text-lg font-semibold text-blue-400">{getCurrentTimestamp()}</p>
        </div>
        
        {/* Playback Controls */}
        <div className="flex items-center space-x-4 mb-4">
          <button
            onClick={handleStepBackward}
            className="p-2 bg-gray-700 hover:bg-gray-600 rounded-full transition-colors"
            title="Previous"
          >
            <BackwardIcon className="h-5 w-5" />
          </button>
          
          <button
            onClick={handlePlayPause}
            className="p-3 bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 rounded-full transition-all duration-200 transform hover:scale-105"
            title={isPlaying ? 'Pause' : 'Play'}
          >
            {isPlaying ? (
              <PauseIcon className="h-6 w-6" />
            ) : (
              <PlayIcon className="h-6 w-6" />
            )}
          </button>
          
          <button
            onClick={handleStepForward}
            className="p-2 bg-gray-700 hover:bg-gray-600 rounded-full transition-colors"
            title="Next"
          >
            <ForwardIcon className="h-5 w-5" />
          </button>

          <input
            type="range"
            min={0}
            max={maxTimeIndex - 1}
            value={currentTimeIndex}
            onChange={(e) => {
              setIsPlaying(false);
              onTimeChange(Number(e.target.value));
            }}
            className="flex-1 h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
            style={{
              background: `linear-gradient(to right, #3b82f6 0%, #3b82f6 ${(currentTimeIndex / (maxTimeIndex - 1)) * 100}%, #374151 ${(currentTimeIndex / (maxTimeIndex - 1)) * 100}%, #374151 100%)`
            }}
          />

          <span className="text-sm font-mono min-w-[100px] text-right">
            {currentTimeIndex + 1} / {maxTimeIndex}
          </span>

          <select
            value={playbackSpeed}
            onChange={(e) => setPlaybackSpeed(Number(e.target.value))}
            className="px-2 py-1 bg-gray-700 rounded text-sm"
            title="Playback speed"
          >
            <option value={1000}>0.5x</option>
            <option value={500}>1x</option>
            <option value={250}>2x</option>
            <option value={100}>5x</option>
          </select>
        </div>

        {/* Statistics and Chart */}
        <div className="grid grid-cols-3 gap-4">
          <div className="space-y-2">
            <h4 className="text-sm font-semibold text-gray-300">Current Statistics</h4>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="bg-gray-700 rounded p-2">
                <div className="text-gray-400">Avg Shadow</div>
                <div className="text-lg font-mono text-blue-400">{(stats.avgShadow * 100).toFixed(1)}%</div>
              </div>
              <div className="bg-gray-700 rounded p-2">
                <div className="text-gray-400">Shadowed</div>
                <div className="text-lg font-mono text-purple-400">{stats.shadowedCells}/{stats.totalCells}</div>
              </div>
            </div>
          </div>

          <div className="col-span-2 h-32">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis 
                  dataKey="hour" 
                  stroke="#9CA3AF"
                  tick={{ fontSize: 10 }}
                />
                <YAxis 
                  stroke="#9CA3AF"
                  tick={{ fontSize: 10 }}
                  domain={[0, 1]}
                />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: '#1F2937', 
                    border: '1px solid #374151',
                    borderRadius: '4px'
                  }}
                  labelStyle={{ color: '#9CA3AF' }}
                />
                <Line 
                  type="monotone" 
                  dataKey="shadow" 
                  stroke="#3B82F6" 
                  strokeWidth={2}
                  dot={false}
                  animationDuration={300}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ResultsPanel;