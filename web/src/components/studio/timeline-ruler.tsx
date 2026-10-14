import React, { useRef, useState, useEffect } from 'react';
import {
  Film,
  Music,
  Subtitles,
  Type,
  Magnet,
  Maximize2,
  ZoomIn,
  ZoomOut,
  Scissors,
} from 'lucide-react';
import { secondsToSMPTE } from '../../lib/timecode';
import { TimelineData } from '../../lib/canvas-renderer';

interface TimelineRulerProps {
  timeline: TimelineData;
  currentTime: number;
  duration: number;
  onTimeChange: (time: number) => void;
  selectedClipId: string | null;
  onSelectClip: (id: string | null) => void;
}

export const TimelineRuler: React.FC<TimelineRulerProps> = ({
  timeline,
  currentTime,
  duration,
  onTimeChange,
  selectedClipId,
  onSelectClip,
}) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const [zoom, setZoom] = useState(1.0); // pixels per second multiplier
  const [magneticSnapping, setMagneticSnapping] = useState(true);
  const [isDraggingPlayhead, setIsDraggingPlayhead] = useState(false);

  const basePps = 80; // base pixels per second
  const pps = basePps * zoom;
  const totalWidth = Math.max(1000, duration * pps + 100);

  // Collect magnetic snap points (clip starts, ends, caption word boundaries)
  const snapPoints: number[] = [0, duration];
  for (const track of timeline.tracks) {
    for (const clip of track.clips) {
      const start = typeof clip.start === 'number' ? clip.start : 0;
      const dur = typeof clip.duration === 'number' ? clip.duration : 5;
      snapPoints.push(start);
      snapPoints.push(start + dur);
    }
  }

  const handlePointerDown = (e: React.PointerEvent) => {
    setIsDraggingPlayhead(true);
    updatePlayheadFromPointer(e);
  };

  const updatePlayheadFromPointer = (e: React.PointerEvent | PointerEvent) => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const scrollLeft = containerRef.current.scrollLeft;
    const clientX = e.clientX - rect.left + scrollLeft - 180; // 180px track label width

    let rawTime = Math.max(0, Math.min(duration, clientX / pps));

    // Magnetic snapping (snap if within 8px)
    if (magneticSnapping) {
      for (const snap of snapPoints) {
        const snapPixel = snap * pps;
        if (Math.abs(clientX - snapPixel) < 8) {
          rawTime = snap;
          break;
        }
      }
    }

    onTimeChange(rawTime);
  };

  useEffect(() => {
    const handlePointerMove = (e: PointerEvent) => {
      if (isDraggingPlayhead) {
        updatePlayheadFromPointer(e);
      }
    };
    const handlePointerUp = () => {
      setIsDraggingPlayhead(false);
    };

    if (isDraggingPlayhead) {
      window.addEventListener('pointermove', handlePointerMove);
      window.addEventListener('pointerup', handlePointerUp);
    }
    return () => {
      window.removeEventListener('pointermove', handlePointerMove);
      window.removeEventListener('pointerup', handlePointerUp);
    };
  }, [isDraggingPlayhead, pps, duration, magneticSnapping]);

  // Generate SMPTE ruler tick marks
  const ticks = [];
  const tickStep = zoom < 0.8 ? 2 : zoom > 2 ? 0.5 : 1.0; // seconds between ticks
  for (let t = 0; t <= duration + 1; t += tickStep) {
    ticks.push(t);
  }

  return (
    <div className="h-64 border-t border-white/8 bg-panel-dark flex flex-col select-none">
      {/* Timeline Controls & Toggles */}
      <div className="h-8 border-b border-white/8 px-4 flex items-center justify-between text-xs bg-canvas-dark/40">
        <div className="flex items-center gap-3">
          <button
            onClick={() => setMagneticSnapping(!magneticSnapping)}
            className={`flex items-center gap-1.5 px-2 py-0.5 rounded text-[11px] font-mono border transition-colors ${
              magneticSnapping
                ? 'bg-amber-safelight/10 text-amber-safelight border-amber-safelight/30 font-medium'
                : 'text-neutral-500 border-white/6 hover:text-neutral-300'
            }`}
          >
            <Magnet className="w-3 h-3" />
            <span>MAGNETIC SNAP</span>
          </button>

          <span className="text-[11px] font-mono text-neutral-500">
            {timeline.tracks.length} Tracks · {duration.toFixed(1)}s Runtime
          </span>
        </div>

        {/* Zoom Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setZoom((z) => Math.max(0.4, z - 0.2))}
            className="p-1 text-neutral-400 hover:text-neutral-200"
            title="Zoom Out"
          >
            <ZoomOut className="w-3.5 h-3.5" />
          </button>
          <span className="text-[10px] font-mono text-neutral-400 w-12 text-center">
            {Math.round(zoom * 100)}%
          </span>
          <button
            onClick={() => setZoom((z) => Math.min(3.0, z + 0.2))}
            className="p-1 text-neutral-400 hover:text-neutral-200"
            title="Zoom In"
          >
            <ZoomIn className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Main Ruler & Tracks Viewport */}
      <div ref={containerRef} className="flex-1 overflow-x-auto overflow-y-auto relative">
        <div style={{ width: `${totalWidth}px` }} className="min-w-full">
          {/* SMPTE Ruler Header */}
          <div
            className="h-7 border-b border-white/8 flex items-center bg-panel-dark sticky top-0 z-20 cursor-pointer"
            onPointerDown={handlePointerDown}
          >
            {/* Label spacer */}
            <div className="w-44 shrink-0 px-3 font-mono text-[11px] text-neutral-500 uppercase border-r border-white/8 bg-panel-dark">
              TRACK
            </div>

            {/* Ruler Ticks */}
            <div className="relative flex-1 h-full">
              {ticks.map((t) => {
                const x = t * pps;
                return (
                  <div
                    key={t}
                    className="absolute top-0 bottom-0 border-l border-white/12 flex flex-col justify-between pl-1"
                    style={{ left: `${x}px` }}
                  >
                    <span className="text-[9px] font-mono text-neutral-500 leading-none pt-1">
                      {secondsToSMPTE(t, 30)}
                    </span>
                    <div className="h-1.5 w-px bg-white/20" />
                  </div>
                );
              })}
            </div>
          </div>

          {/* Track Rows */}
          <div className="relative py-1 space-y-1">
            {timeline.tracks.map((track, trackIdx) => {
              const TrackIcon =
                track.kind === 'video'
                  ? Film
                  : track.kind === 'audio'
                  ? Music
                  : track.kind === 'captions'
                  ? Subtitles
                  : Type;

              return (
                <div key={track.id} className="h-11 flex items-center group relative">
                  {/* Track Label */}
                  <div className="w-44 shrink-0 px-3 flex items-center justify-between border-r border-white/8 bg-panel-dark/90 h-full z-10">
                    <div className="flex items-center gap-2">
                      <TrackIcon className="w-3.5 h-3.5 text-neutral-400" />
                      <span className="text-xs font-mono text-neutral-200">{track.id}</span>
                    </div>
                    <span className="text-[10px] font-mono text-neutral-500 uppercase">
                      {track.kind}
                    </span>
                  </div>

                  {/* Track Clips Canvas */}
                  <div className="flex-1 relative h-full bg-white/[0.015]">
                    {track.clips.map((clip, clipIdx) => {
                      const start = typeof clip.start === 'number' ? clip.start : 0;
                      const clipDur = typeof clip.duration === 'number' ? clip.duration : 5;
                      const clipWidth = clipDur * pps;
                      const clipX = start * pps;
                      const clipKey = `${track.id}-${clipIdx}`;
                      const isSelected = selectedClipId === clipKey;

                      return (
                        <div
                          key={clipKey}
                          onClick={() => onSelectClip(clipKey)}
                          style={{
                            left: `${clipX}px`,
                            width: `${clipWidth}px`,
                          }}
                          className={`absolute top-1 bottom-1 rounded border px-2 flex items-center justify-between overflow-hidden cursor-pointer transition-all duration-fast ${
                            isSelected
                              ? 'border-amber-safelight bg-amber-safelight/15 shadow-sm'
                              : track.kind === 'captions'
                              ? 'border-amber-500/30 bg-amber-500/10 text-amber-200'
                              : track.kind === 'audio'
                              ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-200'
                              : 'border-white/16 bg-white/6 text-neutral-200'
                          }`}
                        >
                          {/* Clip Info */}
                          <div className="truncate text-[11px] font-mono">
                            {clip.asset || clip.text?.text || track.kind}
                          </div>
                          <span className="text-[10px] font-mono opacity-60">
                            {clipDur.toFixed(1)}s
                          </span>

                          {/* Decorative Waveform for Audio */}
                          {track.kind === 'audio' && (
                            <div className="absolute inset-x-0 bottom-0 h-2.5 opacity-30 flex items-end gap-0.5 px-1 overflow-hidden pointer-events-none">
                              {[...Array(24)].map((_, i) => (
                                <div
                                  key={i}
                                  className="w-1 bg-emerald-400 rounded-t-xs"
                                  style={{
                                    height: `${Math.sin(i * 1.5) * 60 + 40}%`,
                                  }}
                                />
                              ))}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              );
            })}

            {/* Draggable Playhead Cursor Line */}
            <div
              className="absolute top-0 bottom-0 z-30 pointer-events-none flex flex-col items-center"
              style={{
                left: `${currentTime * pps + 176}px`, // 176px offset accounting for label
              }}
            >
              <div className="w-3 h-3 bg-amber-safelight rotate-45 -mt-1 shadow-md" />
              <div className="w-0.5 flex-1 bg-amber-safelight shadow-[0_0_8px_rgba(251,191,36,0.6)]" />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
