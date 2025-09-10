import React, { useState, useEffect } from 'react';
import { CalendarIcon, ClockIcon } from '@heroicons/react/24/outline';
import { format, parseISO } from 'date-fns';

interface TimeControlsProps {
  onConfigChange: (config: any) => void;
  config: any;
  disabled?: boolean;
}

const TimeControls: React.FC<TimeControlsProps> = ({ onConfigChange, config, disabled = false }) => {
  const [startDate, setStartDate] = useState(format(new Date(), 'yyyy-MM-dd'));
  const [endDate, setEndDate] = useState(format(new Date(Date.now() + 30 * 24 * 60 * 60 * 1000), 'yyyy-MM-dd'));
  const [hourInterval, setHourInterval] = useState(1);

  useEffect(() => {
    const startISO = new Date(startDate).toISOString();
    const endISO = new Date(endDate).toISOString();
    
    onConfigChange({
      start_date: startISO,
      end_date: endISO,
      hour_interval: hourInterval,
    });
  }, [startDate, endDate, hourInterval, onConfigChange]);

  const calculateTimestamps = () => {
    const start = new Date(startDate).getTime();
    const end = new Date(endDate).getTime();
    const hours = (end - start) / (1000 * 60 * 60);
    return Math.ceil(hours / hourInterval);
  };

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold flex items-center">
        <CalendarIcon className="h-5 w-5 mr-2" />
        Time Configuration
      </h3>
      
      <div>
        <label className="block text-sm font-medium mb-2 text-gray-300">
          Start Date
        </label>
        <input
          type="date"
          value={startDate}
          onChange={(e) => setStartDate(e.target.value)}
          disabled={disabled}
          className="w-full px-3 py-2 bg-gray-700 rounded-md border border-gray-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2 text-gray-300">
          End Date
        </label>
        <input
          type="date"
          value={endDate}
          min={startDate}
          onChange={(e) => setEndDate(e.target.value)}
          disabled={disabled}
          className="w-full px-3 py-2 bg-gray-700 rounded-md border border-gray-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
        />
      </div>

      <div>
        <label className="block text-sm font-medium mb-2 text-gray-300">
          <ClockIcon className="inline h-4 w-4 mr-1" />
          Time Interval
        </label>
        <select
          value={hourInterval}
          onChange={(e) => setHourInterval(Number(e.target.value))}
          disabled={disabled}
          className="w-full px-3 py-2 bg-gray-700 rounded-md border border-gray-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <option value={0.5}>30 minutes</option>
          <option value={1}>1 hour</option>
          <option value={2}>2 hours</option>
          <option value={3}>3 hours</option>
          <option value={6}>6 hours</option>
          <option value={12}>12 hours</option>
          <option value={24}>24 hours</option>
        </select>
      </div>

      <div className="p-3 bg-blue-800 bg-opacity-20 rounded-md border border-blue-600">
        <div className="text-sm space-y-1">
          <p className="text-blue-300">
            <strong>Period:</strong> {Math.ceil((new Date(endDate).getTime() - new Date(startDate).getTime()) / (1000 * 60 * 60 * 24))} days
          </p>
          <p className="text-blue-300">
            <strong>Estimated timestamps:</strong> ~{calculateTimestamps()}
          </p>
          <p className="text-xs text-blue-400 mt-2">
            {calculateTimestamps() > 1000 && '⚠️ Large number of timestamps may take longer to process'}
          </p>
        </div>
      </div>
    </div>
  );
};

export default TimeControls;