import React, { useRef, useEffect, useState, useCallback } from 'react';
import {
  Play,
  Pause,
  SkipBack,
  SkipForward,
  ChevronLeft,
  ChevronRight,
  ShieldAlert,
  CheckCircle2,
  Maximize2,
  Sliders,
} from 'lucide-react';
import {
  renderTimelineToCanvas,
  calculateSSIMParity,
  TimelineData,
} from '../../lib/canvas-renderer';
import { secondsToSMPTE } from '../../lib/timecode';

interface PreviewCanvasProps {
  timeline: TimelineData;
  currentTime: number;
  duration: number;
  isPlaying: boolean;
  onTimeChange: (time: number) => void;
  onTogglePlay: () => void;
  safeZone: 'none' | 'tiktok' | 'reels' | 'shorts';
  onSafeZoneChange: (zone: 'none' | 'tiktok' | 'reels' | 'shorts') => void;
}

export const PreviewCanvas: React.FC<PreviewCanvasProps> = ({
  timeline,
  currentTime,
  duration,
  isPlaying,
  onTimeChange,
  onTogglePlay,
  safeZone,
  onSafeZoneChange,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [showSafeZoneOverlay, setShowSafeZoneOverlay] = useState(true);
  const [aspect, setAspect] = useState<'9:16' | '16:9' | '1:1'>('9:16');

  // Preview Parity SSIM metrics
  const parity = calculateSSIMParity(currentTime);

  // Determine canvas aspect dimensions
  const dimensions = {
    '9:16': { width: 360, height: 640 },
    '16:9': { width: 640, height: 360 },
    '1:1': { width: 440, height: 440 },
  }[aspect];

  // Render frame
  const renderFrame = useCallback(() => {
    if (!canvasRef.current) return;
    renderTimelineToCanvas(canvasRef.current, timeline, {
      width: dimensions.width,
      height: dimensions.height,
      time: currentTime,
      fps: 30,
      safeZone,
      showSafeZoneOverlay,
    });
  }, [timeline, currentTime, dimensions, safeZone, showSafeZoneOverlay]);

  useEffect(() => {
    renderFrame();
  }, [renderFrame]);

  // Decoupled playback loop via requestAnimationFrame
  useEffect(() => {
    if (!isPlaying) return;

    let lastTimestamp = performance.now();
    let animationFrameId: number;

    const loop = (now: number) => {
      const delta = (now - lastTimestamp) / 1000;
      lastTimestamp = now;

      const nextTime = currentTime + delta;
      if (nextTime >= duration) {
        onTimeChange(0);
      } else {
        onTimeChange(nextTime);
      }

      animationFrameId = requestAnimationFrame(loop);
    };

    animationFrameId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(animationFrameId);
  }, [isPlaying, currentTime, duration, onTimeChange]);

  const stepFrame = (frames: number) => {
    const fps = 30;
    const newTime = Math.max(0, Math.min(duration, currentTime + frames / fps));
    onTimeChange(newTime);
  };

  const stepSecond = (seconds: number) => {
    const newTime = Math.max(0, Math.min(duration, currentTime + seconds));
    onTimeChange(newTime);
  };

  return (
    <div className="flex-1 flex flex-col bg-canvas-dark border-x border-white/8 overflow-hidden select-none">
      {/* Top Preview Status Bar */}
      <div className="h-9 border-b border-white/8 px-4 flex items-center justify-between bg-panel-dark/50 text-xs">
        {/* Parity Badge */}
        <div className="flex items-center gap-2">
          <div
            className={`flex items-center gap-1.5 px-2 py-0.5 rounded font-mono text-[11px] font-medium border ${
              parity.parityPassed
                ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                : 'bg-amber-500/10 text-amber-400 border-amber-500/20'
            }`}
          >
            <CheckCircle2 className="w-3 h-3" />
            <span>PREVIEW PARITY SSIM {parity.ssim.toFixed(3)}</span>
          </div>
          <span className="text-[11px] font-mono text-neutral-500">
            Δmax: {parity.maxPixelDelta}px
          </span>
        </div>

        {/* Aspect Ratio & Safe Zone Selector */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1 font-mono text-[11px] text-neutral-400">
            <span className="text-neutral-500">Aspect:</span>
            {(['9:16', '16:9', '1:1'] as const).map((asp) => (
              <button
                key={asp}
                onClick={() => setAspect(asp)}
                className={`px-1.5 py-0.5 rounded ${
                  aspect === asp
                    ? 'bg-white/10 text-neutral-100 font-semibold'
                    : 'text-neutral-400 hover:text-neutral-200'
                }`}
              >
                {asp}
              </button>
            ))}
          </div>

          <div className="h-3 w-px bg-white/10" />

          {/* Safe Zone Switcher */}
          <div className="flex items-center gap-1">
            <ShieldAlert className="w-3.5 h-3.5 text-neutral-400" />
            <select
              value={safeZone}
              onChange={(e) => onSafeZoneChange(e.target.value as any)}
              className="bg-panel-dark border border-white/10 rounded px-1.5 py-0.5 text-[11px] font-mono text-neutral-300 outline-hidden focus:border-amber-safelight"
            >
              <option value="none">No Safe Zone</option>
              <option value="tiktok">TikTok Overlay</option>
              <option value="reels">Instagram Reels</option>
              <option value="shorts">YouTube Shorts</option>
            </select>
            <button
              onClick={() => setShowSafeZoneOverlay(!showSafeZoneOverlay)}
              className={`px-1.5 py-0.5 rounded text-[10px] font-mono border ${
                showSafeZoneOverlay
                  ? 'bg-red-500/10 text-red-400 border-red-500/20'
                  : 'text-neutral-500 border-white/6'
              }`}
            >
              {showSafeZoneOverlay ? 'VISIBLE' : 'HIDDEN'}
            </button>
          </div>
        </div>
      </div>

      {/* Canvas Viewport */}
      <div className="flex-1 flex items-center justify-center p-6 bg-[#08080a] relative overflow-hidden">
        <div className="relative shadow-2xl rounded-sm overflow-hidden border border-white/12">
          <canvas
            ref={canvasRef}
            className="block"
            style={{
              width: dimensions.width,
              height: dimensions.height,
            }}
          />
        </div>
      </div>

      {/* Transport Control Bar */}
      <div className="h-11 border-t border-white/8 px-4 flex items-center justify-between bg-panel-dark">
        {/* Playhead Controls */}
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => onTimeChange(0)}
            className="p-1 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded"
            title="Jump to Start (Home)"
          >
            <SkipBack className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => stepSecond(-1)}
            className="p-1 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded"
            title="Step Back 1s (Shift+←)"
          >
            <ChevronLeft className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => stepFrame(-1)}
            className="px-1.5 py-0.5 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded text-[11px] font-mono"
            title="Step Back 1 Frame (←)"
          >
            -1F
          </button>

          <button
            onClick={onTogglePlay}
            className="p-1.5 bg-amber-safelight text-neutral-950 font-bold rounded hover:bg-amber-highlight transition-colors mx-1"
            title="Play / Pause (Space)"
          >
            {isPlaying ? <Pause className="w-4 h-4 fill-current" /> : <Play className="w-4 h-4 fill-current" />}
          </button>

          <button
            onClick={() => stepFrame(1)}
            className="px-1.5 py-0.5 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded text-[11px] font-mono"
            title="Step Forward 1 Frame (→)"
          >
            +1F
          </button>
          <button
            onClick={() => stepSecond(1)}
            className="p-1 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded"
            title="Step Forward 1s (Shift+→)"
          >
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => onTimeChange(duration)}
            className="p-1 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded"
            title="Jump to End (End)"
          >
            <SkipForward className="w-3.5 h-3.5" />
          </button>
        </div>

        {/* SMPTE Timecode & Frame Display */}
        <div className="flex items-center gap-3 font-mono">
          <div className="flex items-center gap-1.5 bg-black/40 border border-white/8 px-2.5 py-1 rounded">
            <span className="text-[11px] text-neutral-500 uppercase">SMPTE</span>
            <span className="text-xs font-bold text-amber-safelight tracking-wider smpte-timecode">
              {secondsToSMPTE(currentTime, 30)}
            </span>
          </div>

          <span className="text-[11px] text-neutral-500">
            / {secondsToSMPTE(duration, 30)} ({Math.round(currentTime * 30)}f)
          </span>
        </div>
      </div>
    </div>
  );
};
