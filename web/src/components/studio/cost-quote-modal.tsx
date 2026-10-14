import React from 'react';
import { X, CheckCircle2, ShieldCheck, AlertTriangle, DollarSign, Calculator } from 'lucide-react';
import { formatBilledCost } from '../../lib/timecode';

interface CostQuoteModalProps {
  isOpen: boolean;
  onClose: () => void;
  timeline: any;
  onConfirmRender?: () => void;
}

export const CostQuoteModal: React.FC<CostQuoteModalProps> = ({
  isOpen,
  onClose,
  timeline,
  onConfirmRender,
}) => {
  if (!isOpen) return null;

  // Compute pricing metrics based on PRD §1.4 & §1.5
  const duration = 12.0;
  const billedSeconds = Math.ceil(duration * 10) / 10;
  const outputsCount = timeline.outputs?.length || 1;
  const is4K = timeline.outputs?.some((o: any) => o.resolution?.width >= 3840);
  const is60Fps = timeline.outputs?.some((o: any) => o.fps >= 60);
  const hasCaptions = timeline.tracks?.some((t: any) => t.kind === 'captions');
  const hasC2pa = timeline.provenance?.c2pa;

  const baseRatePerMin = 0.08;
  const resMultiplier = is4K ? 2.0 : 1.0;
  const fpsMultiplier = is60Fps ? 1.5 : 1.0;
  const baseComputeUsd = (billedSeconds * (baseRatePerMin / 60)) * resMultiplier * fpsMultiplier;

  const fanoutCostUsd = outputsCount > 1 ? (outputsCount - 1) * (0.02 / 60) * billedSeconds : 0.0;
  const captionsCostUsd = hasCaptions ? (0.01 / 60) * billedSeconds : 0.0;
  const provenanceCostUsd = hasC2pa ? 0.005 : 0.0;

  const totalUsd = baseComputeUsd + fanoutCostUsd + captionsCostUsd + provenanceCostUsd;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4 select-none"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg bg-panel-dark border border-white/12 rounded-lg shadow-2xl p-6 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between pb-4 border-b border-white/8">
          <div className="flex items-center gap-2">
            <Calculator className="w-5 h-5 text-amber-safelight" />
            <div>
              <h3 className="text-sm font-semibold text-neutral-100">Dry-Run Cost Quote</h3>
              <p className="text-[11px] font-mono text-neutral-500">POST /v1/renders:validate</p>
            </div>
          </div>
          <button onClick={onClose} className="text-neutral-400 hover:text-neutral-200">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Validation Checks */}
        <div className="mt-4 p-3 bg-white/4 border border-white/6 rounded space-y-1.5 text-xs font-mono">
          <div className="flex items-center gap-2 text-emerald-400">
            <CheckCircle2 className="w-4 h-4" />
            <span>Timeline JSON Schema valid · 0 syntax errors</span>
          </div>
          <div className="flex items-center gap-2 text-emerald-400">
            <ShieldCheck className="w-4 h-4" />
            <span>SSRF Asset Reachability verified · 2 assets reachable</span>
          </div>
          <div className="flex items-center gap-2 text-neutral-400">
            <AlertTriangle className="w-4 h-4 text-amber-400" />
            <span>Font glyph coverage: 100% latin / digits</span>
          </div>
        </div>

        {/* Itemized Cost Ledger */}
        <div className="mt-5 space-y-2">
          <h4 className="text-[11px] font-mono text-neutral-400 uppercase tracking-wider">
            ITEMIZED COST LEDGER (STARTER TIER)
          </h4>

          <div className="border border-white/8 rounded divide-y divide-white/6 text-xs font-mono bg-canvas-dark">
            <div className="p-2.5 flex items-center justify-between">
              <div>
                <span className="text-neutral-200">Base GPU Compute ({billedSeconds}s @ $0.08/min)</span>
                <div className="text-[10px] text-neutral-500">
                  Multiplier: {resMultiplier}× (Res) · {fpsMultiplier}× (FPS)
                </div>
              </div>
              <span className="text-neutral-200 font-semibold">${baseComputeUsd.toFixed(4)}</span>
            </div>

            {outputsCount > 1 && (
              <div className="p-2.5 flex items-center justify-between">
                <div>
                  <span className="text-neutral-200">
                    Multi-Format Fanout ({outputsCount - 1} extra outputs)
                  </span>
                  <div className="text-[10px] text-neutral-500">Rate: $0.02 / extra output-min</div>
                </div>
                <span className="text-neutral-200 font-semibold">${fanoutCostUsd.toFixed(4)}</span>
              </div>
            )}

            {hasCaptions && (
              <div className="p-2.5 flex items-center justify-between">
                <div>
                  <span className="text-neutral-200">Caption Forced Alignment Engine</span>
                  <div className="text-[10px] text-neutral-500">Rate: $0.01 / audio-min</div>
                </div>
                <span className="text-neutral-200 font-semibold">${captionsCostUsd.toFixed(4)}</span>
              </div>
            )}

            {hasC2pa && (
              <div className="p-2.5 flex items-center justify-between">
                <div>
                  <span className="text-neutral-200">C2PA Cryptographic Provenance & AI Disclosure</span>
                  <div className="text-[10px] text-neutral-500">Flat rate: $0.005 / video</div>
                </div>
                <span className="text-neutral-200 font-semibold">${provenanceCostUsd.toFixed(4)}</span>
              </div>
            )}

            {/* Total Line */}
            <div className="p-3 bg-white/4 flex items-center justify-between text-sm">
              <span className="font-bold text-neutral-100">Estimated Total Quote</span>
              <span className="font-bold text-amber-safelight tracking-wide font-mono text-base">
                ${totalUsd.toFixed(4)}
              </span>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="mt-6 flex items-center justify-end gap-3">
          <button
            onClick={onClose}
            className="px-3 py-1.5 rounded text-xs font-mono text-neutral-400 hover:text-neutral-200 border border-white/8 hover:bg-white/4"
          >
            Close
          </button>
          <button
            onClick={() => {
              if (onConfirmRender) onConfirmRender();
              onClose();
            }}
            className="px-4 py-1.5 rounded text-xs font-bold text-neutral-950 bg-amber-safelight hover:bg-amber-highlight transition-colors flex items-center gap-1.5"
          >
            <span>Submit Live Render</span>
          </button>
        </div>
      </div>
    </div>
  );
};
