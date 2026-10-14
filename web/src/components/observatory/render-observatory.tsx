import React, { useState } from 'react';
import {
  Activity,
  CheckCircle2,
  Clock,
  Cpu,
  Layers,
  ArrowRight,
  Copy,
  Check,
  RefreshCw,
  ExternalLink,
  ShieldCheck,
  FileText,
  Sliders,
} from 'lucide-react';
import { secondsToSMPTE } from '../../lib/timecode';

export interface StageTrace {
  stage: string;
  durationMs: number;
  workerId: string;
  throughputFps?: number;
  bytesProcessedMb?: number;
  status: 'succeeded' | 'running' | 'queued';
  details?: Record<string, any>;
}

export const RenderObservatory: React.FC = () => {
  const [copied, setCopied] = useState(false);
  const [hoverFrameIdx, setHoverFrameIdx] = useState<number>(4);

  const renderData = {
    id: '01927c34-8c88-7e23-81a1-8d0739c9f28c',
    status: 'succeeded',
    lane: 'priority',
    engineVersion: '0.1.0-wgpu (build #441)',
    tenantId: 'tenant_acme',
    createdAt: '2026-10-14 15:02:12 UTC',
    completedAt: '2026-10-14 15:02:15 UTC',
    queueWaitMs: 312,
    totalDurationMs: 2450,
    billedSeconds: 12.0,
    costUsd: 0.0354,
    outputs: [
      { id: 'yt', aspect: '16:9', resolution: '1920x1080', format: 'mp4', url: 'https://cdn.cutline.dev/renders/yt.mp4' },
      { id: 'tt', aspect: '9:16', resolution: '1080x1920', format: 'mp4', reframe: 'smart', url: 'https://cdn.cutline.dev/renders/tt.mp4' },
    ],
  };

  const stages: StageTrace[] = [
    { stage: 'fetching', durationMs: 420, workerId: 'worker-gpu-us-east-1', bytesProcessedMb: 48.2, status: 'succeeded' },
    { stage: 'aligning', durationMs: 210, workerId: 'worker-gpu-us-east-1', throughputFps: 180, status: 'succeeded', details: { wordsAligned: 24, language: 'en' } },
    { stage: 'compositing', durationMs: 840, workerId: 'worker-gpu-us-east-1', throughputFps: 142, status: 'succeeded', details: { streams: 2, smartReframed: 1 } },
    { stage: 'encoding', durationMs: 510, workerId: 'worker-gpu-us-east-1', throughputFps: 95, status: 'succeeded', details: { codec: 'h264_nvenc' } },
    { stage: 'muxing', durationMs: 140, workerId: 'worker-gpu-us-east-1', status: 'succeeded', details: { normalizeLufs: -14 } },
    { stage: 'signing', durationMs: 95, workerId: 'worker-gpu-us-east-1', status: 'succeeded', details: { c2pa: true, aiDisclosed: true, algo: 'es256' } },
    { stage: 'uploading', durationMs: 235, workerId: 'worker-gpu-us-east-1', bytesProcessedMb: 14.8, status: 'succeeded' },
  ];

  const totalStageMs = stages.reduce((acc, s) => acc + s.durationMs, 0);

  const copyCurl = () => {
    const curl = `curl -X GET https://api.cutline.dev/v1/renders/${renderData.id} \\\n  -H "Authorization: Bearer cl_live_sample"`;
    navigator.clipboard.writeText(curl);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex-1 flex flex-col h-[calc(100vh-3rem)] overflow-y-auto select-none bg-canvas-dark p-6 space-y-6">
      {/* Observatory Title & Metadata Header */}
      <div className="flex items-start justify-between border-b border-white/8 pb-4">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-lg font-bold font-mono text-neutral-100 flex items-center gap-2">
              <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" />
              <span>Render {renderData.id}</span>
            </h1>
            <span className="px-2 py-0.5 rounded font-mono text-xs bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase font-semibold">
              {renderData.status}
            </span>
            <span className="px-2 py-0.5 rounded font-mono text-xs bg-amber-safelight/10 text-amber-safelight border border-amber-safelight/20 uppercase">
              {renderData.lane} Lane
            </span>
          </div>
          <p className="text-xs font-mono text-neutral-500 mt-1">
            Tenant: {renderData.tenantId} · Engine: {renderData.engineVersion} · Created: {renderData.createdAt}
          </p>
        </div>

        {/* Top Actions */}
        <div className="flex items-center gap-2">
          <button
            onClick={copyCurl}
            className="px-3 py-1.5 rounded bg-white/6 hover:bg-white/10 text-neutral-200 border border-white/10 font-mono text-xs flex items-center gap-1.5 transition-colors"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5 text-neutral-400" />}
            <span>Copy as cURL</span>
          </button>

          <button className="px-3 py-1.5 rounded bg-amber-safelight hover:bg-amber-highlight text-neutral-950 font-bold text-xs flex items-center gap-1.5 transition-colors">
            <RefreshCw className="w-3.5 h-3.5" />
            <span>Re-run on Pinned Engine</span>
          </button>
        </div>
      </div>

      {/* Grid: Filmstrip Scrubber + Cost Ledger */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Filmstrip In-Place Thumbnail Scrubber (PRD §5.4 B) */}
        <div className="lg:col-span-2 bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider flex items-center gap-2">
              <Layers className="w-4 h-4 text-amber-safelight" />
              <span>Filmstrip Thumbnail Scrubber (Hover to Scrub)</span>
            </span>
            <span className="text-[11px] font-mono text-neutral-400">
              Frame {hoverFrameIdx + 1}/8 · SMPTE {secondsToSMPTE((hoverFrameIdx / 8) * 12, 30)}
            </span>
          </div>

          {/* Filmstrip Frame Previews */}
          <div className="grid grid-cols-8 gap-1.5 bg-canvas-dark p-2 rounded border border-white/6">
            {[...Array(8)].map((_, i) => (
              <div
                key={i}
                onMouseEnter={() => setHoverFrameIdx(i)}
                className={`relative aspect-9/16 rounded overflow-hidden cursor-crosshair border transition-all ${
                  hoverFrameIdx === i
                    ? 'border-amber-safelight shadow-md ring-1 ring-amber-safelight scale-105 z-10'
                    : 'border-white/10 opacity-70 hover:opacity-100'
                }`}
              >
                {/* Visual frame mock representation */}
                <div
                  className="w-full h-full flex flex-col justify-between p-1.5"
                  style={{
                    background: `linear-gradient(135deg, hsl(${30 + i * 15}, 20%, 15%), hsl(${40 + i * 20}, 30%, 10%))`,
                  }}
                >
                  <span className="text-[9px] font-mono text-neutral-400">F{i * 45}</span>
                  <div className="w-full h-1 bg-amber-safelight/40 rounded-full" />
                </div>
              </div>
            ))}
          </div>

          <div className="flex items-center justify-between text-xs font-mono text-neutral-400 pt-2">
            <span>Output: 1080×1920 (9:16) MP4</span>
            <a
              href="https://cdn.cutline.dev/renders/tt.mp4"
              target="_blank"
              rel="noreferrer"
              className="text-amber-safelight hover:underline flex items-center gap-1"
            >
              <span>Download Signed Video</span>
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>
        </div>

        {/* Cost Ledger (PRD §5.4 B) */}
        <div className="bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider flex items-center gap-2">
              <Clock className="w-4 h-4 text-emerald-400" />
              <span>Metered Cost Ledger</span>
            </span>
            <span className="text-[11px] font-mono text-emerald-400">Idempotent</span>
          </div>

          <div className="divide-y divide-white/6 text-xs font-mono">
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">Billed Duration (ceil 100ms)</span>
              <span className="text-neutral-200 font-semibold">{renderData.billedSeconds.toFixed(1)}s</span>
            </div>
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">Base GPU Compute ($0.08/min)</span>
              <span className="text-neutral-200 font-semibold">$0.0160</span>
            </div>
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">Fanout Stream Add-on</span>
              <span className="text-neutral-200 font-semibold">$0.0040</span>
            </div>
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">Caption Forced Alignment</span>
              <span className="text-neutral-200 font-semibold">$0.0020</span>
            </div>
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">C2PA Provenance Signature</span>
              <span className="text-neutral-200 font-semibold">$0.0050</span>
            </div>
            <div className="py-2 flex items-center justify-between">
              <span className="text-neutral-400">Priority Lane (1.5× multiplier)</span>
              <span className="text-neutral-200 font-semibold">1.50×</span>
            </div>
            <div className="pt-3 flex items-center justify-between text-sm font-bold border-t border-white/12">
              <span className="text-neutral-100">Invoice Total</span>
              <span className="text-amber-safelight font-mono text-base">${renderData.costUsd.toFixed(4)}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Stage Waterfall (Trace View) */}
      <div className="bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Activity className="w-4 h-4 text-amber-safelight" />
            <h2 className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
              Execution Stage Waterfall ({stages.length} Stages · {totalStageMs}ms Total)
            </h2>
          </div>
          <span className="text-[11px] font-mono text-neutral-400">
            Queue Wait: {renderData.queueWaitMs}ms
          </span>
        </div>

        {/* Visual Waterfall Bars */}
        <div className="space-y-2 pt-2">
          {stages.map((stg) => {
            const widthPct = Math.max(4, (stg.durationMs / totalStageMs) * 100);
            return (
              <div
                key={stg.stage}
                className="group p-2.5 rounded bg-canvas-dark border border-white/6 hover:border-white/16 transition-all text-xs font-mono"
              >
                <div className="flex items-center justify-between mb-1.5">
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-emerald-400" />
                    <span className="font-semibold text-neutral-100 uppercase">{stg.stage}</span>
                    <span className="text-[10px] text-neutral-500 font-mono">({stg.workerId})</span>
                  </div>

                  <div className="flex items-center gap-3 text-neutral-400 text-[11px]">
                    {stg.throughputFps && <span>{stg.throughputFps} fps</span>}
                    {stg.bytesProcessedMb && <span>{stg.bytesProcessedMb} MB</span>}
                    <span className="font-bold text-neutral-200">{stg.durationMs}ms</span>
                  </div>
                </div>

                {/* Relative Bar */}
                <div className="w-full h-1.5 bg-white/6 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-amber-safelight group-hover:bg-amber-highlight transition-all"
                    style={{ width: `${widthPct}%` }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
