import React from 'react';
import { XMarkIcon } from '@heroicons/react/24/outline';

interface ProgressModalProps {
  isVisible: boolean;
  progress: number;
  currentStep: string;
  totalSteps?: number;
  currentStepNumber?: number;
  onCancel?: () => void;
  canCancel?: boolean;
}

const ProgressModal: React.FC<ProgressModalProps> = ({
  isVisible,
  progress,
  currentStep,
  totalSteps,
  currentStepNumber,
  onCancel,
  canCancel = false
}) => {
  if (!isVisible) return null;

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 bg-black bg-opacity-60 backdrop-blur-sm flex items-center justify-center" style={{ zIndex: 9999 }}>
        {/* Modal */}
        <div className="bg-gray-800 rounded-lg shadow-2xl border border-gray-600 w-96 mx-4" style={{ zIndex: 10000 }}>
          {/* Header */}
          <div className="flex items-center justify-between p-6 border-b border-gray-600">
            <div className="flex items-center space-x-3">
              {/* Animated spinner */}
              <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
              <h3 className="text-lg font-semibold text-white">
                Calculating Shadows
              </h3>
            </div>
            {canCancel && onCancel && (
              <button
                onClick={onCancel}
                className="text-gray-400 hover:text-white transition-colors"
                aria-label="Cancel calculation"
              >
                <XMarkIcon className="w-5 h-5" />
              </button>
            )}
          </div>

          {/* Content */}
          <div className="p-6 space-y-4">
            {/* Progress Bar */}
            <div className="space-y-3">
              <div className="flex justify-between text-sm">
                <span className="text-gray-300">Progress</span>
                <span className="text-blue-400 font-mono">{Math.round(progress)}%</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-3">
                <div 
                  className="bg-gradient-to-r from-blue-500 to-purple-500 h-3 rounded-full transition-all duration-300 ease-out"
                  style={{ width: `${Math.max(0, Math.min(100, progress))}%` }}
                ></div>
              </div>
            </div>

            {/* Simple status message */}
            <div className="text-center">
              <p className="text-sm text-gray-300 mb-2">{currentStep}</p>
              <p className="text-xs text-gray-400">
                Please wait while shadows are calculated...
                <br />
                This may take several minutes for large areas.
              </p>
            </div>
          </div>
        </div>
      </div>
    </>
  );
};

export default ProgressModal;