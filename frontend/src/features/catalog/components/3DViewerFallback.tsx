import React from 'react';

interface ViewerProps {
  modelUrl?: string;
  productName?: string;
}

export const ThreeDViewerFallback: React.FC<ViewerProps> = ({ productName }) => {
  return (
    <div className="w-full aspect-video bg-gradient-to-br from-slate-950 via-slate-900 to-indigo-950/40 border border-slate-800 rounded-2xl flex flex-col items-center justify-center p-8 relative overflow-hidden group">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(99,102,241,0.15)_0,transparent_70%)]" />
      <div className="w-32 h-32 rounded-3xl border border-indigo-500/30 bg-slate-900/80 shadow-2xl flex items-center justify-center animate-pulse relative z-10">
        <span className="text-4xl">🧊</span>
      </div>
      <p className="mt-4 text-sm font-semibold text-slate-300 relative z-10">
        3D Interactive Model Preview
      </p>
      <p className="text-xs text-slate-500 mt-1 relative z-10">
        {productName || 'WebGL Fallback Context'}
      </p>
    </div>
  );
};

export default ThreeDViewerFallback;
