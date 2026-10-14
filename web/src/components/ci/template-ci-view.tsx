import React, { useState } from 'react';
import {
  GitCompare,
  CheckCircle2,
  XCircle,
  Split,
  Eye,
  SlidersHorizontal,
  Flame,
  Check,
  X,
  FileCheck,
} from 'lucide-react';
import { secondsToSMPTE } from '../../lib/timecode';

interface GoldenFrame {
  timestamp: number;
  expectedHash: string;
  actualHash: string;
  ssim: number;
  passed: boolean;
  maxDelta: number;
}

export const TemplateCiView: React.FC = () => {
  const [sliderPos, setSliderPos] = useState<number>(50); // percentage (0-100)
  const [showHeatmap, setShowHeatmap] = useState<boolean>(false);
  const [selectedFrameIdx, setSelectedFrameIdx] = useState<number>(0);
  const [approvalStatus, setApprovalStatus] = useState<'pending' | 'approved' | 'rejected'>('approved');

  const frames: GoldenFrame[] = [
    { timestamp: 0.5, expectedHash: 'e3b0c44298fc1c14', actualHash: 'e3b0c44298fc1c14', ssim: 1.0, passed: true, maxDelta: 0 },
    { timestamp: 2.0, expectedHash: '9f86d081884c7d65', actualHash: '9f86d081884c7d65', ssim: 1.0, passed: true, maxDelta: 0 },
    { timestamp: 4.5, expectedHash: '4b227777d4dd1fc6', actualHash: '4b227777d4dd1fc6', ssim: 0.998, passed: true, maxDelta: 2 },
  ];

  const currentFrame = frames[selectedFrameIdx];

  return (
    <div className="flex-1 flex flex-col h-[calc(100vh-3rem)] overflow-y-auto select-none bg-canvas-dark p-6 space-y-6">
      {/* CI Run Summary Banner */}
      <div className="flex items-start justify-between border-b border-white/8 pb-4">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-lg font-bold font-mono text-neutral-100 flex items-center gap-2">
              <FileCheck className="w-5 h-5 text-amber-safelight" />
              <span>Template CI: Golden Frame Regression Diff</span>
            </h1>
            <span className="px-2 py-0.5 rounded font-mono text-xs bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-bold uppercase">
              PASSING (SSIM ≥ 0.98)
            </span>
          </div>
          <p className="text-xs font-mono text-neutral-500 mt-1">
            Template: tpl_faceless_tiktok (git sha: f161ba5) · Ran: 2026-10-14 15:00:00 UTC
          </p>
        </div>

        {/* PR-style Approve / Reject Actions */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setApprovalStatus('rejected')}
            className={`px-3 py-1.5 rounded text-xs font-mono border flex items-center gap-1.5 transition-colors ${
              approvalStatus === 'rejected'
                ? 'bg-red-500/20 text-red-400 border-red-500/30 font-bold'
                : 'text-neutral-400 hover:text-neutral-200 border-white/10 hover:bg-white/4'
            }`}
          >
            <X className="w-3.5 h-3.5" />
            <span>Reject Changes</span>
          </button>
          <button
            onClick={() => setApprovalStatus('approved')}
            className={`px-4 py-1.5 rounded text-xs font-mono border font-bold flex items-center gap-1.5 transition-colors ${
              approvalStatus === 'approved'
                ? 'bg-emerald-500 text-neutral-950 border-emerald-400 font-bold shadow-xs'
                : 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
            }`}
          >
            <Check className="w-3.5 h-3.5" />
            <span>Approve & Update Goldens</span>
          </button>
        </div>
      </div>

      {/* Main Diff Area: Interactive Draggable Swipe Divider */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-2 bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Split className="w-4 h-4 text-amber-safelight" />
              <span className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
                Interactive Visual Diff (Swipe Divider)
              </span>
            </div>

            {/* Toggle Heatmap Overlay */}
            <button
              onClick={() => setShowHeatmap(!showHeatmap)}
              className={`px-2 py-1 rounded text-xs font-mono border flex items-center gap-1.5 transition-colors ${
                showHeatmap
                  ? 'bg-amber-safelight/15 text-amber-safelight border-amber-safelight/40 font-bold'
                  : 'text-neutral-400 border-white/8 hover:bg-white/4'
              }`}
            >
              <Flame className="w-3.5 h-3.5" />
              <span>Heatmap Overlay ({showHeatmap ? 'ON' : 'OFF'})</span>
            </button>
          </div>

          {/* Swipe Container Viewport */}
          <div className="relative aspect-16/9 bg-black rounded overflow-hidden border border-white/12 select-none">
            {/* Expected Image (Baseline) */}
            <div
              className="absolute inset-0 flex items-center justify-center font-mono text-sm"
              style={{
                background: 'linear-gradient(135deg, #111827 0%, #1f2937 100%)',
              }}
            >
              <div className="text-center space-y-1">
                <span className="px-2 py-1 rounded bg-black/60 text-xs text-neutral-300 font-mono border border-white/10">
                  BASELINE (GOLDEN)
                </span>
                <p className="text-[11px] text-neutral-500">Hash: {currentFrame.expectedHash}</p>
              </div>
            </div>

            {/* Actual Image (Candidate) with Clip Path */}
            <div
              className="absolute inset-0 flex items-center justify-center font-mono text-sm border-r-2 border-amber-safelight"
              style={{
                clipPath: `polygon(0 0, ${sliderPos}% 0, ${sliderPos}% 100%, 0 100%)`,
                background: showHeatmap
                  ? 'radial-gradient(circle, rgba(239,68,68,0.25) 0%, rgba(17,24,39,1) 80%)'
                  : 'linear-gradient(135deg, #18181b 0%, #27272a 100%)',
              }}
            >
              <div className="text-center space-y-1">
                <span className="px-2 py-1 rounded bg-black/60 text-xs text-amber-safelight font-mono border border-amber-safelight/30">
                  CANDIDATE RENDER
                </span>
                <p className="text-[11px] text-neutral-500">Hash: {currentFrame.actualHash}</p>
              </div>
            </div>

            {/* Draggable Divider Handle */}
            <input
              type="range"
              min="0"
              max="100"
              value={sliderPos}
              onChange={(e) => setSliderPos(parseInt(e.target.value, 10))}
              className="absolute inset-x-0 bottom-3 w-[95%] mx-auto opacity-70 hover:opacity-100 cursor-ew-resize accent-amber-safelight"
            />
          </div>

          <div className="flex items-center justify-between text-xs font-mono text-neutral-400">
            <span>Swipe Slider Position: {sliderPos}%</span>
            <span>Current Frame Timestamp: {secondsToSMPTE(currentFrame.timestamp, 30)}</span>
          </div>
        </div>

        {/* Pinned Golden Frames Sortable Table */}
        <div className="bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
              Pinned Golden Set ({frames.length})
            </span>
            <span className="text-[11px] font-mono text-neutral-500">Threshold: 0.980</span>
          </div>

          <div className="space-y-2">
            {frames.map((frame, idx) => (
              <div
                key={frame.timestamp}
                onClick={() => setSelectedFrameIdx(idx)}
                className={`p-3 rounded border cursor-pointer font-mono text-xs transition-all ${
                  selectedFrameIdx === idx
                    ? 'border-amber-safelight bg-amber-safelight/10'
                    : 'border-white/8 bg-canvas-dark hover:border-white/16'
                }`}
              >
                <div className="flex items-center justify-between mb-1.5">
                  <span className="font-semibold text-neutral-200">
                    Timestamp: {frame.timestamp.toFixed(1)}s ({secondsToSMPTE(frame.timestamp, 30)})
                  </span>
                  <span
                    className={`px-1.5 py-0.5 rounded text-[10px] font-bold ${
                      frame.passed
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : 'bg-red-500/10 text-red-400 border border-red-500/20'
                    }`}
                  >
                    SSIM {frame.ssim.toFixed(3)}
                  </span>
                </div>
                <div className="text-[10px] text-neutral-500 truncate">
                  Expected: {frame.expectedHash}
                </div>
                <div className="text-[10px] text-neutral-500 truncate">
                  Actual: {frame.actualHash}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
