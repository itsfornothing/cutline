import React from 'react';
import { Subtitles, Sparkles, Sliders, Shield } from 'lucide-react';

interface CaptionsInspectorProps {
  style: any;
  onStyleChange: (updatedStyle: any) => void;
  safeZone: string;
  onSafeZoneChange: (zone: any) => void;
}

export const CaptionsInspector: React.FC<CaptionsInspectorProps> = ({
  style,
  onStyleChange,
  safeZone,
  onSafeZoneChange,
}) => {
  const words = [
    { text: 'Deterministic', start: '0.0s', end: '1.2s', confidence: 0.99 },
    { text: 'video', start: '1.2s', end: '1.8s', confidence: 0.98 },
    { text: 'rendering', start: '1.8s', end: '2.6s', confidence: 0.95 },
    { text: 'with', start: '2.6s', end: '3.0s', confidence: 0.99 },
    { text: 'Cutline', start: '3.0s', end: '4.2s', confidence: 0.97 },
  ];

  return (
    <div className="space-y-4 text-xs">
      {/* Caption Presets */}
      <div className="space-y-2">
        <label className="block font-mono text-[11px] text-neutral-400">STYLE PRESET</label>
        <div className="grid grid-cols-3 gap-1.5">
          {(['karaoke', 'pop_in', 'boxed'] as const).map((preset) => (
            <button
              key={preset}
              onClick={() => onStyleChange({ ...style, preset })}
              className={`py-1.5 px-2 rounded text-center font-mono text-[11px] border capitalize transition-colors ${
                (style.preset || 'karaoke') === preset
                  ? 'bg-amber-safelight/15 text-amber-safelight border-amber-safelight/40 font-bold'
                  : 'text-neutral-400 border-white/6 hover:bg-white/4'
              }`}
            >
              {preset.replace('_', ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Highlight Color & Font Size */}
      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <label className="block font-mono text-[11px] text-neutral-400">HIGHLIGHT COLOR</label>
          <div className="flex items-center gap-2 bg-canvas-dark border border-white/10 rounded px-2 py-1">
            <input
              type="color"
              value={style.highlight || '#FFB000'}
              onChange={(e) => onStyleChange({ ...style, highlight: e.target.value })}
              className="w-4 h-4 rounded border-0 cursor-pointer bg-transparent"
            />
            <span className="font-mono text-[11px] text-neutral-200">
              {style.highlight || '#FFB000'}
            </span>
          </div>
        </div>

        <div className="space-y-1">
          <label className="block font-mono text-[11px] text-neutral-400">FONT SIZE (PX)</label>
          <input
            type="number"
            value={style.font_size || 52}
            onChange={(e) => onStyleChange({ ...style, font_size: parseInt(e.target.value, 10) })}
            className="w-full bg-canvas-dark border border-white/10 rounded px-2 py-1 text-xs font-mono text-neutral-200 outline-hidden"
          />
        </div>
      </div>

      {/* Safe Zone Alignment */}
      <div className="space-y-1">
        <label className="block font-mono text-[11px] text-neutral-400 flex items-center justify-between">
          <span>SAFE-ZONE OVERLAY</span>
          <Shield className="w-3 h-3 text-red-400" />
        </label>
        <select
          value={safeZone}
          onChange={(e) => onSafeZoneChange(e.target.value)}
          className="w-full bg-canvas-dark border border-white/10 rounded px-2 py-1.5 text-xs font-mono text-neutral-200 outline-hidden"
        >
          <option value="none">Disabled (Full Bleed)</option>
          <option value="tiktok">TikTok (Top 108px / Bottom 320px)</option>
          <option value="reels">Instagram Reels (Top 120px / Bottom 280px)</option>
          <option value="shorts">YouTube Shorts (Top 96px / Bottom 240px)</option>
        </select>
      </div>

      {/* Word-Level Aligned Transcript */}
      <div className="pt-2 border-t border-white/8 space-y-2">
        <div className="flex items-center justify-between font-mono text-[11px] text-neutral-400">
          <span>ALIGNED WORDS ({words.length})</span>
          <span className="text-emerald-400">Forced Aligner</span>
        </div>

        <div className="space-y-1 max-h-48 overflow-y-auto pr-1">
          {words.map((w, i) => (
            <div
              key={i}
              className="p-1.5 rounded bg-white/4 border border-white/6 flex items-center justify-between text-xs font-mono"
            >
              <span className="text-neutral-100 font-medium">{w.text}</span>
              <div className="flex items-center gap-2 text-[10px] text-neutral-500">
                <span>
                  {w.start}–{w.end}
                </span>
                <span
                  className="px-1 py-0.2 rounded bg-emerald-500/10 text-emerald-400 font-mono"
                  title="Confidence score"
                >
                  {(w.confidence * 100).toFixed(0)}%
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
