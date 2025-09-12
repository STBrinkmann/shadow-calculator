import React, { useState, useEffect } from 'react';
import { SeasonalAnalysis, MonthlyShadowStats, SeasonStats } from '../types';

interface SeasonalDashboardProps {
  data: SeasonalAnalysis | null;
  onClose: () => void;
}

interface MonthCardProps {
  monthStats: MonthlyShadowStats;
  isSelected: boolean;
  onClick: () => void;
}

const MonthCard: React.FC<MonthCardProps> = ({ monthStats, isSelected, onClick }) => {
  const monthNames = [
    '', 'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
  ];

  // Calculate average values across all cells for display
  const calculateAverage = (data: number[][]) => {
    if (!data.length || !data[0].length) return 0;
    const sum = data.flat().reduce((acc, val) => acc + val, 0);
    return sum / (data.length * data[0].length);
  };

  const avgShadowPercent = calculateAverage(monthStats.avg_shadow_percentage);
  const avgSolarEfficiency = calculateAverage(monthStats.solar_efficiency_percentage);

  return (
    <div
      className={`border rounded-lg p-4 cursor-pointer transition-all duration-300 ${
        isSelected 
          ? 'border-blue-500 bg-blue-50 shadow-lg' 
          : 'border-gray-200 hover:border-gray-300 hover:shadow-md'
      }`}
      onClick={onClick}
    >
      <div className="text-center">
        <h3 className={`text-lg font-semibold mb-2 ${isSelected ? 'text-blue-900' : 'text-gray-800'}`}>
          {monthNames[monthStats.month]} {monthStats.year}
        </h3>
        <div className="space-y-2 text-sm">
          <div>
            <span className={isSelected ? 'text-blue-700' : 'text-gray-600'}>Shadow: </span>
            <span className={`font-medium ${isSelected ? 'text-blue-900' : 'text-gray-900'}`}>{avgShadowPercent.toFixed(1)}%</span>
          </div>
          <div>
            <span className={isSelected ? 'text-blue-700' : 'text-gray-600'}>Solar: </span>
            <span className="font-medium text-green-600">{avgSolarEfficiency.toFixed(1)}%</span>
          </div>
          <div className={`text-xs ${isSelected ? 'text-blue-600' : 'text-gray-500'}`}>
            {monthStats.days_in_analysis} days analyzed
          </div>
        </div>
      </div>
    </div>
  );
};

interface SeasonCardProps {
  seasonStats: SeasonStats;
  isSelected: boolean;
  onClick: () => void;
}

const SeasonCard: React.FC<SeasonCardProps> = ({ seasonStats, isSelected, onClick }) => {
  const calculateAverage = (data: number[][]) => {
    if (!data.length || !data[0].length) return 0;
    const sum = data.flat().reduce((acc, val) => acc + val, 0);
    return sum / (data.length * data[0].length);
  };

  const avgShadowPercent = calculateAverage(seasonStats.avg_shadow_percentage);
  const avgSolarEfficiency = calculateAverage(seasonStats.solar_efficiency_percentage);

  const getSeasonIcon = (season: string) => {
    switch (season.toLowerCase()) {
      case 'spring': return '🌸';
      case 'summer': return '☀️';
      case 'autumn':
      case 'fall': return '🍂';
      case 'winter': return '❄️';
      default: return '📅';
    }
  };

  const getSeasonColor = (season: string) => {
    switch (season.toLowerCase()) {
      case 'spring': return 'text-green-600';
      case 'summer': return 'text-yellow-600';
      case 'autumn':
      case 'fall': return 'text-orange-600';
      case 'winter': return 'text-blue-600';
      default: return 'text-gray-600';
    }
  };

  return (
    <div
      className={`border rounded-lg p-4 cursor-pointer transition-all duration-300 ${
        isSelected 
          ? 'border-blue-500 bg-blue-50 shadow-lg' 
          : 'border-gray-200 hover:border-gray-300 hover:shadow-md'
      }`}
      onClick={onClick}
    >
      <div className="text-center">
        <div className="text-3xl mb-2">{getSeasonIcon(seasonStats.season_name)}</div>
        <h3 className={`text-xl font-semibold mb-3 ${getSeasonColor(seasonStats.season_name)}`}>
          {seasonStats.season_name}
        </h3>
        <div className="space-y-2 text-sm">
          <div>
            <span className="text-gray-600">Avg Shadow: </span>
            <span className="font-medium">{avgShadowPercent.toFixed(1)}%</span>
          </div>
          <div>
            <span className="text-gray-600">Avg Solar: </span>
            <span className="font-medium text-green-600">{avgSolarEfficiency.toFixed(1)}%</span>
          </div>
          <div className="text-xs text-gray-500">
            {seasonStats.total_days} total days
          </div>
        </div>
      </div>
    </div>
  );
};

interface SeasonalDashboardInlineProps {
  data: SeasonalAnalysis | null;
}

const SeasonalDashboardInline: React.FC<SeasonalDashboardInlineProps> = ({ data }) => {
  const [selectedMonth, setSelectedMonth] = useState<number | null>(null);
  const [selectedSeason, setSelectedSeason] = useState<string | null>(null);
  const [currentView, setCurrentView] = useState<'months' | 'seasons'>('months');
  const [isAnimating, setIsAnimating] = useState(false);

  useEffect(() => {
    // Auto-select first month when data is loaded
    if (data?.monthly_stats && data.monthly_stats.length > 0 && selectedMonth === null) {
      setSelectedMonth(0);
    }
  }, [data, selectedMonth]);

  if (!data) {
    return (
      <div className="flex-1 flex items-center justify-center bg-gray-50">
        <div className="text-center p-8 max-w-md">
          <div className="w-20 h-20 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-6">
            <svg className="w-10 h-10 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
          <h3 className="text-xl font-semibold text-gray-800 mb-2">No Seasonal Data Available</h3>
          <p className="text-gray-600 mb-6">
            Seasonal analysis requires shadow calculation data. Please calculate shadows for your area or upload existing results to view seasonal patterns.
          </p>
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 text-sm text-blue-800">
            <p className="font-medium mb-1">💡 To get started:</p>
            <ul className="text-left space-y-1">
              <li>• Upload DTM and DSM files</li>
              <li>• Draw your area of interest on the map</li>
              <li>• Set your analysis time period</li>
              <li>• Click "Calculate Shadows"</li>
            </ul>
          </div>
        </div>
      </div>
    );
  }

  const handleViewChange = (view: 'months' | 'seasons') => {
    if (view === currentView) return;
    
    setIsAnimating(true);
    setTimeout(() => {
      setCurrentView(view);
      setSelectedMonth(null);
      setSelectedSeason(null);
      setIsAnimating(false);
    }, 150);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString();
  };

  const calculateAverage = (data: number[][]) => {
    if (!data.length || !data[0].length) return 0;
    const sum = data.flat().reduce((acc, val) => acc + val, 0);
    return sum / (data.length * data[0].length);
  };

  return (
    <div className="flex-1 flex flex-col bg-white">
      {/* Header */}
      <div className="border-b p-6 bg-white">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-2xl font-bold text-gray-800">Seasonal Analysis Dashboard</h1>
            <p className="text-sm text-gray-600 mt-1">
              Analysis Period: {formatDate(data.analysis_period[0])} - {formatDate(data.analysis_period[1])}
            </p>
          </div>
        </div>
        
        {/* View Toggle */}
        <div className="flex space-x-2 mt-4">
          <button
            onClick={() => handleViewChange('months')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              currentView === 'months'
                ? 'bg-blue-100 text-blue-700 border-blue-200'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Monthly View ({data.monthly_stats.length} months)
          </button>
          <button
            onClick={() => handleViewChange('seasons')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              currentView === 'seasons'
                ? 'bg-blue-100 text-blue-700 border-blue-200'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            Seasonal View ({data.seasonal_summaries.length} seasons)
          </button>
        </div>
      </div>

      {/* Content */}
      <div className={`flex-1 overflow-y-auto p-6 transition-opacity duration-300 ${isAnimating ? 'opacity-50' : 'opacity-100'}`}>
        {currentView === 'months' && (
          <div>
            <h2 className="text-xl font-semibold mb-4">Monthly Shadow Analysis</h2>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
              {data.monthly_stats.map((monthStats, index) => (
                <MonthCard
                  key={`${monthStats.year}-${monthStats.month}`}
                  monthStats={monthStats}
                  isSelected={selectedMonth === index}
                  onClick={() => setSelectedMonth(index)}
                />
              ))}
            </div>
            
            {/* Month Details */}
            {selectedMonth !== null && data.monthly_stats[selectedMonth] && (
              <div className="mt-6 p-6 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg border border-blue-200 shadow-sm">
                <h3 className="text-lg font-semibold mb-4 text-gray-800 flex items-center">
                  <span className="mr-2">📊</span>
                  Detailed Statistics for {new Date(data.monthly_stats[selectedMonth].year, data.monthly_stats[selectedMonth].month - 1).toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}
                </h3>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                  <div className="bg-white p-4 rounded-lg shadow-sm">
                    <div className="text-2xl mb-2">📅</div>
                    <div className="text-sm text-gray-600 mb-1">Days Analyzed</div>
                    <div className="text-xl font-bold text-gray-900">{data.monthly_stats[selectedMonth].days_in_analysis}</div>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm">
                    <div className="text-2xl mb-2">🌑</div>
                    <div className="text-sm text-gray-600 mb-1">Avg Shadow</div>
                    <div className="text-xl font-bold text-gray-900">
                      {calculateAverage(data.monthly_stats[selectedMonth].avg_shadow_percentage).toFixed(1)}%
                    </div>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm">
                    <div className="text-2xl mb-2">☀️</div>
                    <div className="text-sm text-gray-600 mb-1">Solar Efficiency</div>
                    <div className="text-xl font-bold text-green-600">
                      {calculateAverage(data.monthly_stats[selectedMonth].solar_efficiency_percentage).toFixed(1)}%
                    </div>
                  </div>
                  <div className="bg-white p-4 rounded-lg shadow-sm">
                    <div className="text-2xl mb-2">⏱️</div>
                    <div className="text-sm text-gray-600 mb-1">Max Consecutive Shadow</div>
                    <div className="text-xl font-bold text-gray-900">
                      {calculateAverage(data.monthly_stats[selectedMonth].max_consecutive_shadow).toFixed(1)} hrs
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}

        {currentView === 'seasons' && (
          <div>
            <h2 className="text-xl font-semibold mb-4">Seasonal Shadow Analysis</h2>
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
              {data.seasonal_summaries.map((seasonStats) => (
                <SeasonCard
                  key={seasonStats.season_name}
                  seasonStats={seasonStats}
                  isSelected={selectedSeason === seasonStats.season_name}
                  onClick={() => setSelectedSeason(seasonStats.season_name)}
                />
              ))}
            </div>
            
            {/* Season Details */}
            {selectedSeason && (
              <div className="mt-6 p-6 bg-gradient-to-r from-amber-50 to-yellow-50 rounded-lg border border-amber-200 shadow-sm">
                <h3 className="text-lg font-semibold mb-4 text-gray-800 flex items-center">
                  <span className="mr-2">📈</span>
                  Detailed Statistics for {selectedSeason}
                </h3>
                {(() => {
                  const season = data.seasonal_summaries.find(s => s.season_name === selectedSeason);
                  if (!season) return null;
                  
                  return (
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                      <div className="bg-white p-4 rounded-lg shadow-sm">
                        <div className="text-2xl mb-2">📅</div>
                        <div className="text-sm text-gray-600 mb-1">Total Days</div>
                        <div className="text-xl font-bold text-gray-900">{season.total_days}</div>
                      </div>
                      <div className="bg-white p-4 rounded-lg shadow-sm">
                        <div className="text-2xl mb-2">🗓️</div>
                        <div className="text-sm text-gray-600 mb-1">Months Included</div>
                        <div className="text-sm font-bold text-gray-900">{season.months.map(m => new Date(0, m-1).toLocaleDateString('en', {month: 'short'})).join(', ')}</div>
                      </div>
                      <div className="bg-white p-4 rounded-lg shadow-sm">
                        <div className="text-2xl mb-2">🌑</div>
                        <div className="text-sm text-gray-600 mb-1">Avg Shadow</div>
                        <div className="text-xl font-bold text-gray-900">
                          {calculateAverage(season.avg_shadow_percentage).toFixed(1)}%
                        </div>
                      </div>
                      <div className="bg-white p-4 rounded-lg shadow-sm">
                        <div className="text-2xl mb-2">☀️</div>
                        <div className="text-sm text-gray-600 mb-1">Solar Efficiency</div>
                        <div className="text-xl font-bold text-green-600">
                          {calculateAverage(season.solar_efficiency_percentage).toFixed(1)}%
                        </div>
                      </div>
                    </div>
                  );
                })()}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="border-t p-4 bg-gray-50 text-sm text-gray-600">
        <p>
          <strong>Tip:</strong> Click on any month or season card to view detailed statistics. 
          Switch between monthly and seasonal views using the tabs above.
        </p>
      </div>
    </div>
  );
};

export const SeasonalDashboard: React.FC<SeasonalDashboardProps> = ({ data, onClose }) => {
  const [selectedMonth, setSelectedMonth] = useState<number | null>(null);
  const [selectedSeason, setSelectedSeason] = useState<string | null>(null);
  const [currentView, setCurrentView] = useState<'months' | 'seasons'>('months');
  const [isAnimating, setIsAnimating] = useState(false);

  const calculateAverage = (data: number[][]) => {
    if (!data.length || !data[0].length) return 0;
    const sum = data.flat().reduce((acc, val) => acc + val, 0);
    return sum / (data.length * data[0].length);
  };

  useEffect(() => {
    // Auto-select first month when data is loaded
    if (data?.monthly_stats && data.monthly_stats.length > 0 && selectedMonth === null) {
      setSelectedMonth(0);
    }
  }, [data, selectedMonth]);

  if (!data) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white p-6 rounded-lg shadow-lg">
          <p>Loading seasonal analysis...</p>
        </div>
      </div>
    );
  }

  const handleViewChange = (view: 'months' | 'seasons') => {
    if (view === currentView) return;
    
    setIsAnimating(true);
    setTimeout(() => {
      setCurrentView(view);
      setSelectedMonth(null);
      setSelectedSeason(null);
      setIsAnimating(false);
    }, 150);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[9999] p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="sticky top-0 bg-white border-b p-6 z-10">
          <div className="flex justify-between items-center">
            <div>
              <h1 className="text-2xl font-bold text-gray-800">Seasonal Analysis Dashboard</h1>
              <p className="text-sm text-gray-600 mt-1">
                Analysis Period: {formatDate(data.analysis_period[0])} - {formatDate(data.analysis_period[1])}
              </p>
            </div>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 text-2xl font-bold"
            >
              ×
            </button>
          </div>
          
          {/* View Toggle */}
          <div className="flex space-x-2 mt-4">
            <button
              onClick={() => handleViewChange('months')}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                currentView === 'months'
                  ? 'bg-blue-100 text-blue-700 border-blue-200'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Monthly View ({data.monthly_stats.length} months)
            </button>
            <button
              onClick={() => handleViewChange('seasons')}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                currentView === 'seasons'
                  ? 'bg-blue-100 text-blue-700 border-blue-200'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              Seasonal View ({data.seasonal_summaries.length} seasons)
            </button>
          </div>
        </div>

        {/* Content */}
        <div className={`p-6 transition-opacity duration-300 ${isAnimating ? 'opacity-50' : 'opacity-100'}`}>
          {currentView === 'months' && (
            <div>
              <h2 className="text-xl font-semibold mb-4">Monthly Shadow Analysis</h2>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
                {data.monthly_stats.map((monthStats, index) => (
                  <MonthCard
                    key={`${monthStats.year}-${monthStats.month}`}
                    monthStats={monthStats}
                    isSelected={selectedMonth === index}
                    onClick={() => setSelectedMonth(index)}
                  />
                ))}
              </div>
              
              {/* Month Details */}
              {selectedMonth !== null && data.monthly_stats[selectedMonth] && (
                <div className="mt-6 p-6 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg border border-blue-200 shadow-sm">
                  <h3 className="text-lg font-semibold mb-4 text-gray-800 flex items-center">
                    <span className="mr-2">📊</span>
                    Detailed Statistics for {new Date(data.monthly_stats[selectedMonth].year, data.monthly_stats[selectedMonth].month - 1).toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}
                  </h3>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                    <div className="bg-white p-4 rounded-lg shadow-sm">
                      <div className="text-2xl mb-2">📅</div>
                      <div className="text-sm text-gray-600 mb-1">Days Analyzed</div>
                      <div className="text-xl font-bold text-gray-900">{data.monthly_stats[selectedMonth].days_in_analysis}</div>
                    </div>
                    {/* Add more detailed statistics here */}
                  </div>
                </div>
              )}
            </div>
          )}

          {currentView === 'seasons' && (
            <div>
              <h2 className="text-xl font-semibold mb-4">Seasonal Shadow Analysis</h2>
              <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
                {data.seasonal_summaries.map((seasonStats) => (
                  <SeasonCard
                    key={seasonStats.season_name}
                    seasonStats={seasonStats}
                    isSelected={selectedSeason === seasonStats.season_name}
                    onClick={() => setSelectedSeason(seasonStats.season_name)}
                  />
                ))}
              </div>
              
              {/* Season Details */}
              {selectedSeason && (
                <div className="mt-6 p-6 bg-gradient-to-r from-amber-50 to-yellow-50 rounded-lg border border-amber-200 shadow-sm">
                  <h3 className="text-lg font-semibold mb-4 text-gray-800 flex items-center">
                    <span className="mr-2">📈</span>
                    Detailed Statistics for {selectedSeason}
                  </h3>
                  {(() => {
                    const season = data.seasonal_summaries.find(s => s.season_name === selectedSeason);
                    if (!season) return null;
                    
                    return (
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
                        <div className="bg-white p-4 rounded-lg shadow-sm">
                          <div className="text-2xl mb-2">📅</div>
                          <div className="text-sm text-gray-600 mb-1">Total Days</div>
                          <div className="text-xl font-bold text-gray-900">{season.total_days}</div>
                        </div>
                        <div className="bg-white p-4 rounded-lg shadow-sm">
                          <div className="text-2xl mb-2">🗓️</div>
                          <div className="text-sm text-gray-600 mb-1">Months Included</div>
                          <div className="text-sm font-bold text-gray-900">{season.months.map(m => new Date(0, m-1).toLocaleDateString('en', {month: 'short'})).join(', ')}</div>
                        </div>
                        <div className="bg-white p-4 rounded-lg shadow-sm">
                          <div className="text-2xl mb-2">🌑</div>
                          <div className="text-sm text-gray-600 mb-1">Avg Shadow</div>
                          <div className="text-xl font-bold text-gray-900">
                            {calculateAverage(season.avg_shadow_percentage).toFixed(1)}%
                          </div>
                        </div>
                        <div className="bg-white p-4 rounded-lg shadow-sm">
                          <div className="text-2xl mb-2">☀️</div>
                          <div className="text-sm text-gray-600 mb-1">Solar Efficiency</div>
                          <div className="text-xl font-bold text-green-600">
                            {calculateAverage(season.solar_efficiency_percentage).toFixed(1)}%
                          </div>
                        </div>
                      </div>
                    );
                  })()}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="border-t p-4 bg-gray-50 text-sm text-gray-600">
          <p>
            <strong>Tip:</strong> Click on any month or season card to view detailed statistics. 
            Switch between monthly and seasonal views using the tabs above.
          </p>
        </div>
      </div>
    </div>
  );
};

export { SeasonalDashboardInline };
export default SeasonalDashboard;