import React, { useState } from 'react';
import {
  Play,
  Zap,
  Sliders,
  DollarSign,
  ShieldCheck,
  Cpu,
  ArrowRight,
  ExternalLink,
  Layers,
  Check,
} from 'lucide-react';
import { PreviewCanvas } from '../studio/preview-canvas';
import { SAMPLE_TEMPLATES } from '../../lib/sample-templates';
import { TimelineData } from '../../lib/canvas-renderer';

interface MarketingViewProps {
  onOpenStudio: () => void;
}

export const MarketingView: React.FC<MarketingViewProps> = ({ onOpenStudio }) => {
  const [selectedTemplate, setSelectedTemplate] = useState(SAMPLE_TEMPLATES[0]);
  const [timeline, setTimeline] = useState<TimelineData>(SAMPLE_TEMPLATES[0].timeline);
  const [currentTime, setCurrentTime] = useState(1.5);
  const [isPlaying, setIsPlaying] = useState(false);
  const [monthlyMinutes, setMonthlyMinutes] = useState(1500);

  // Pricing calculation vs Self-Hosted Headless Chrome (Remotion / Puppeteer)
  const cutlineRatePerMin = monthlyMinutes > 5000 ? 0.04 : monthlyMinutes > 1000 ? 0.05 : 0.08;
  const cutlineTotalUsd = monthlyMinutes * cutlineRatePerMin;

  // Self-hosting 8x c6i.2xlarge EC2 instances + EBS + egress + engineering maintenance (~$650/mo minimum)
  const selfHostCompute = 480; // 24/7 dedicated servers
  const selfHostEngMaintenance = 250; // On-call dev ops maintenance
  const selfHostTotalUsd = Math.round(selfHostCompute + selfHostEngMaintenance + monthlyMinutes * 0.02);

  return (
    <div className="flex-1 flex flex-col h-[calc(100vh-3rem)] overflow-y-auto select-none bg-canvas-dark text-neutral-100">
      {/* Hero Section: Live Editable Demo (PRD §5.5) */}
      <section className="p-8 max-w-6xl mx-auto w-full space-y-6">
        <div className="space-y-3">
          <div className="inline-flex items-center gap-2 px-2.5 py-1 rounded bg-amber-safelight/10 text-amber-safelight border border-amber-safelight/30 font-mono text-xs">
            <span className="w-2 h-2 rounded-full bg-amber-safelight" />
            <span>DETERMINISTIC VIDEO RENDERING API</span>
          </div>

          <h1 className="text-3xl md:text-5xl font-bold tracking-tight text-neutral-100">
            Send JSON. Get platform-ready video.
          </h1>
          <p className="text-sm md:text-base text-neutral-400 max-w-2xl font-mono leading-relaxed">
            Native pure-Rust compositor with word-level captions, smart multi-format reframing, and C2PA provenance credentials built in. Sub-second dry runs, exactly-once billing.
          </p>
        </div>

        {/* Live Interactive Hero Box */}
        <div className="bg-panel-dark border border-white/12 rounded-lg shadow-2xl overflow-hidden grid grid-cols-1 md:grid-cols-2">
          {/* Left: JSON Editor & Template Switcher */}
          <div className="p-5 border-b md:border-b-0 md:border-r border-white/8 flex flex-col justify-between space-y-4">
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
                  Select Real Template
                </span>
                <span className="text-[11px] font-mono text-neutral-500">Live 2-Way Sync</span>
              </div>

              <div className="flex items-center gap-1.5">
                {SAMPLE_TEMPLATES.map((t) => (
                  <button
                    key={t.id}
                    onClick={() => {
                      setSelectedTemplate(t);
                      setTimeline(t.timeline);
                      setCurrentTime(1.0);
                    }}
                    className={`flex-1 py-1 px-2 rounded text-xs font-mono border truncate transition-colors ${
                      selectedTemplate.id === t.id
                        ? 'bg-amber-safelight/15 text-amber-safelight border-amber-safelight/40 font-bold'
                        : 'text-neutral-400 border-white/6 hover:bg-white/4'
                    }`}
                  >
                    {t.aspect}
                  </button>
                ))}
              </div>

              <div className="bg-canvas-dark rounded p-3 font-mono text-[11px] text-neutral-300 max-h-56 overflow-y-auto border border-white/6">
                <pre>{JSON.stringify(timeline, null, 2)}</pre>
              </div>
            </div>

            {/* Instant Real Cost Quote Badge (PRD §5.5) */}
            <div className="p-3 bg-white/4 rounded border border-white/6 flex items-center justify-between font-mono text-xs">
              <div className="flex items-center gap-2">
                <DollarSign className="w-4 h-4 text-amber-safelight" />
                <span className="text-neutral-300">
                  {selectedTemplate.duration}s · 1080p · captions
                </span>
              </div>
              <span className="font-bold text-amber-safelight text-sm">$0.0160</span>
            </div>
          </div>

          {/* Right: Real Rendered Canvas Preview */}
          <div className="p-5 flex flex-col items-center justify-center bg-black/40">
            <PreviewCanvas
              timeline={timeline}
              currentTime={currentTime}
              duration={selectedTemplate.duration}
              isPlaying={isPlaying}
              onTimeChange={setCurrentTime}
              onTogglePlay={() => setIsPlaying(!isPlaying)}
              safeZone="tiktok"
              onSafeZoneChange={() => {}}
            />
          </div>
        </div>

        {/* CTAs */}
        <div className="flex items-center gap-4 pt-2">
          <button
            onClick={onOpenStudio}
            className="px-5 py-2.5 rounded bg-amber-safelight hover:bg-amber-highlight text-neutral-950 font-bold text-sm flex items-center gap-2 transition-colors shadow-sm"
          >
            <span>Launch Template Studio</span>
            <ArrowRight className="w-4 h-4" />
          </button>

          <span className="text-xs font-mono text-neutral-500">
            No signup required · 10 free minutes/mo
          </span>
        </div>
      </section>

      {/* Pricing Calculator Section: Side-by-Side vs Self-Hosting (PRD §5.5) */}
      <section className="p-8 border-t border-white/8 max-w-6xl mx-auto w-full space-y-6">
        <div className="space-y-1">
          <h2 className="text-2xl font-bold font-mono text-neutral-100">
            Transparent, Metered Pricing
          </h2>
          <p className="text-xs font-mono text-neutral-400">
            Drag the slider to compare effective monthly render cost vs self-hosting headless browsers.
          </p>
        </div>

        {/* Minutes Slider */}
        <div className="bg-panel-dark border border-white/8 rounded-lg p-6 space-y-4">
          <div className="flex items-center justify-between font-mono">
            <span className="text-xs text-neutral-400 uppercase tracking-wider">MONTHLY VOLUME</span>
            <span className="text-lg font-bold text-amber-safelight">
              {monthlyMinutes.toLocaleString()} video minutes / month
            </span>
          </div>

          <input
            type="range"
            min="100"
            max="15000"
            step="100"
            value={monthlyMinutes}
            onChange={(e) => setMonthlyMinutes(parseInt(e.target.value, 10))}
            className="w-full accent-amber-safelight cursor-pointer"
          />

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4">
            {/* Cutline Managed Plan */}
            <div className="p-5 rounded bg-canvas-dark border border-amber-safelight/40 space-y-3 font-mono">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-neutral-200 uppercase">CUTLINE API</span>
                <span className="px-2 py-0.5 rounded text-[10px] bg-amber-safelight/20 text-amber-safelight font-bold">
                  ${cutlineRatePerMin.toFixed(2)}/min
                </span>
              </div>
              <div className="text-3xl font-bold text-neutral-100">${cutlineTotalUsd.toFixed(2)}</div>
              <p className="text-[11px] text-neutral-400">
                Fully managed NVENC GPU nodes, instant zero cold-start queue, word-level captions, fanout reframing, and C2PA signing included.
              </p>
            </div>

            {/* Self-Hosted Browser Renderer Estimate */}
            <div className="p-5 rounded bg-canvas-dark border border-white/8 space-y-3 font-mono opacity-80">
              <div className="flex items-center justify-between">
                <span className="text-xs font-bold text-neutral-400 uppercase">
                  SELF-HOSTED CHROME (REMOTION/PUPPETEER)
                </span>
                <span className="text-[10px] text-neutral-500">Estimated</span>
              </div>
              <div className="text-3xl font-bold text-neutral-300">${selfHostTotalUsd.toFixed(2)}</div>
              <p className="text-[11px] text-neutral-500">
                Assumes 8× c6i.2xlarge cloud VMs, high RAM spikes, crash restarts, and dev-ops maintenance overhead.
              </p>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};
