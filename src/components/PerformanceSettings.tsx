import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { CpuInfo, Config } from '../types';

interface PerformanceSettingsProps {
  config: Config;
  onConfigChange: (config: Config) => void;
  isOpen: boolean;
  onClose: () => void;
}

export default function PerformanceSettings({ 
  config, 
  onConfigChange, 
  isOpen, 
  onClose 
}: PerformanceSettingsProps) {
  const [cpuInfo, setCpuInfo] = useState<CpuInfo | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      fetchCpuInfo();
    }
  }, [isOpen]);

  const fetchCpuInfo = async () => {
    setLoading(true);
    try {
      const info = await invoke<CpuInfo>('get_cpu_info');
      setCpuInfo(info);
    } catch (error) {
      console.error('Failed to get CPU info:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCpuCoresChange = (cores: number | null) => {
    onConfigChange({
      ...config,
      cpu_cores: cores === null ? undefined : cores
    });
  };

  if (!isOpen) return null;
  
  console.log('PerformanceSettings rendering, isOpen:', isOpen, 'cpuInfo:', cpuInfo);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center" style={{ zIndex: 10001 }}>
      <div className="bg-white rounded-lg shadow-lg p-6 max-w-md w-full mx-4" style={{ zIndex: 10002 }}>
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold">Performance Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
          >
            ×
          </button>
        </div>

        <div className="space-y-4">
          <div>
            <h3 className="font-semibold mb-2">CPU Configuration</h3>
            {loading ? (
              <p className="text-gray-500">Loading CPU information...</p>
            ) : cpuInfo ? (
              <div className="space-y-3">
                <div className="text-sm text-gray-600">
                  Available CPU cores: <span className="font-semibold">{cpuInfo.total_cores}</span>
                </div>
                
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    CPU Cores to Use
                  </label>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-2">
                      <input
                        type="radio"
                        id="auto-detect"
                        name="cpu-setting"
                        checked={config.cpu_cores === undefined}
                        onChange={() => handleCpuCoresChange(null)}
                        className="text-blue-600"
                      />
                      <label htmlFor="auto-detect" className="text-sm">
                        Auto-detect (use all {cpuInfo.total_cores} cores)
                      </label>
                    </div>
                    
                    <div className="flex items-center space-x-2">
                      <input
                        type="radio"
                        id="manual-setting"
                        name="cpu-setting"
                        checked={config.cpu_cores !== undefined}
                        onChange={() => handleCpuCoresChange(Math.max(1, Math.floor(cpuInfo.total_cores / 2)))}
                        className="text-blue-600"
                      />
                      <label htmlFor="manual-setting" className="text-sm">
                        Manual setting
                      </label>
                    </div>

                    {config.cpu_cores !== undefined && (
                      <div className="ml-6 space-y-2">
                        <div className="flex items-center space-x-2">
                          <span className="text-sm text-gray-600">Cores:</span>
                          <input
                            type="range"
                            min="1"
                            max={cpuInfo.total_cores}
                            value={config.cpu_cores || 1}
                            onChange={(e) => handleCpuCoresChange(parseInt(e.target.value))}
                            className="flex-1"
                          />
                          <span className="font-semibold text-sm min-w-[2ch]">
                            {config.cpu_cores}
                          </span>
                        </div>
                        <div className="text-xs text-gray-500">
                          Using {config.cpu_cores} of {cpuInfo.total_cores} available cores
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <p className="text-red-500 text-sm">Failed to load CPU information</p>
            )}
          </div>

          <div className="border-t pt-4">
            <h4 className="font-medium text-gray-700 mb-2">Performance Tips</h4>
            <ul className="text-xs text-gray-600 space-y-1">
              <li>• Auto-detect uses all available cores for maximum performance</li>
              <li>• Manual setting allows you to reserve cores for other applications</li>
              <li>• More cores = faster processing but higher CPU usage</li>
              <li>• Changes apply to the next calculation</li>
            </ul>
          </div>
        </div>

        <div className="flex justify-end space-x-2 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-600 hover:text-gray-800"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}