import React, { useState } from 'react';
import {
  BarChart3,
  Download,
  KeyRound,
  TrendingDown,
  AlertCircle,
  CreditCard,
  DollarSign,
  ArrowUpRight,
} from 'lucide-react';

export const UsageView: React.FC = () => {
  const [downloadToast, setDownloadToast] = useState(false);

  const handleExportCsv = (tableName: string) => {
    setDownloadToast(true);
    setTimeout(() => setDownloadToast(false), 2500);
  };

  const templatesData = [
    { name: 'faceless_tiktok', count: 1840, billedSec: 22080, cost: '$29.44', sparkline: [12, 18, 24, 30, 45, 50, 68] },
    { name: 'real_estate_tour', count: 620, billedSec: 6200, cost: '$8.26', sparkline: [8, 10, 15, 14, 22, 20, 28] },
    { name: 'fanout_promo_3x', count: 410, billedSec: 6150, cost: '$12.30', sparkline: [5, 8, 12, 16, 18, 25, 30] },
    { name: 'ecom_product_card', count: 890, billedSec: 4450, cost: '$5.93', sparkline: [15, 20, 25, 22, 30, 35, 42] },
  ];

  const apiKeysData = [
    { keyId: 'key_prod_us_east', name: 'US Production Cluster', tier: 'Scale', usageSec: 28400, spend: '$37.86' },
    { keyId: 'key_staging_ci', name: 'GitHub Actions Golden CI', tier: 'Starter', usageSec: 2480, spend: '$3.30' },
    { keyId: 'key_dev_local', name: 'Developer Local Testing', tier: 'Free', usageSec: 800, spend: '$0.00' },
  ];

  return (
    <div className="flex-1 flex flex-col h-[calc(100vh-3rem)] overflow-y-auto select-none bg-canvas-dark p-6 space-y-6">
      {/* Title & Budget Burn-Down Header */}
      <div className="flex items-start justify-between border-b border-white/8 pb-4">
        <div>
          <h1 className="text-lg font-bold font-mono text-neutral-100 flex items-center gap-2">
            <CreditCard className="w-5 h-5 text-amber-safelight" />
            <span>Usage & Metered Billing Ledger</span>
          </h1>
          <p className="text-xs font-mono text-neutral-500 mt-1">
            Tenant: tenant_acme (Growth Tier) · Cycle: October 2026 · Currency: USD
          </p>
        </div>

        <button
          onClick={() => handleExportCsv('all_usage')}
          className="px-3 py-1.5 rounded bg-white/6 hover:bg-white/10 text-neutral-200 border border-white/10 font-mono text-xs flex items-center gap-1.5 transition-colors"
        >
          <Download className="w-3.5 h-3.5" />
          <span>Export Full Ledger CSV</span>
        </button>
      </div>

      {downloadToast && (
        <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono rounded">
          ✓ Export complete: usage_events_2026_10.csv generated.
        </div>
      )}

      {/* KPI Cards & Budget Burn-Down */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div className="bg-panel-dark border border-white/8 rounded-lg p-4 space-y-1">
          <span className="text-[11px] font-mono text-neutral-500 uppercase">MONTHLY SPEND</span>
          <div className="text-2xl font-bold font-mono text-neutral-100">$55.93</div>
          <span className="text-[10px] font-mono text-emerald-400">Budget: $150.00 (37% used)</span>
        </div>

        <div className="bg-panel-dark border border-white/8 rounded-lg p-4 space-y-1">
          <span className="text-[11px] font-mono text-neutral-500 uppercase">TOTAL BILLED TIME</span>
          <div className="text-2xl font-bold font-mono text-amber-safelight">38,880s</div>
          <span className="text-[10px] font-mono text-neutral-400">648.0 render minutes</span>
        </div>

        <div className="bg-panel-dark border border-white/8 rounded-lg p-4 space-y-1">
          <span className="text-[11px] font-mono text-neutral-500 uppercase">SUCCESS RATE</span>
          <div className="text-2xl font-bold font-mono text-emerald-400">99.92%</div>
          <span className="text-[10px] font-mono text-neutral-400">3,760 total jobs</span>
        </div>

        <div className="bg-panel-dark border border-white/8 rounded-lg p-4 space-y-1">
          <span className="text-[11px] font-mono text-neutral-500 uppercase">AVERAGE JOB TIME</span>
          <div className="text-2xl font-bold font-mono text-neutral-200">1.84s</div>
          <span className="text-[10px] font-mono text-neutral-400">1.9× local baseline speed</span>

        </div>
      </div>

      {/* Small-Multiples Sparklines per Template (PRD §5.4 D) */}
      <div className="bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <BarChart3 className="w-4 h-4 text-amber-safelight" />
            <h2 className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
              Small-Multiples: Activity & Spend per Template
            </h2>
          </div>
          <button
            onClick={() => handleExportCsv('templates')}
            className="text-[11px] font-mono text-neutral-400 hover:text-neutral-200"
          >
            Export CSV
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {templatesData.map((tpl) => (
            <div
              key={tpl.name}
              className="p-3.5 rounded bg-canvas-dark border border-white/6 space-y-2 font-mono text-xs"
            >
              <div className="flex items-center justify-between font-semibold text-neutral-200">
                <span className="truncate">{tpl.name}</span>
                <span className="text-amber-safelight">{tpl.cost}</span>
              </div>
              <div className="flex items-center justify-between text-[11px] text-neutral-500">
                <span>{tpl.count} videos</span>
                <span>{tpl.billedSec}s</span>
              </div>

              {/* Sparkline Graphic Bars */}
              <div className="flex items-end gap-1 h-8 pt-1">
                {tpl.sparkline.map((val, idx) => (
                  <div
                    key={idx}
                    className="flex-1 bg-amber-safelight/40 hover:bg-amber-safelight rounded-t-xs transition-colors"
                    style={{ height: `${(val / 70) * 100}%` }}
                    title={`Day ${idx + 1}: ${val} renders`}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Per-API-Key Attribution Table */}
      <div className="bg-panel-dark border border-white/8 rounded-lg p-5 space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <KeyRound className="w-4 h-4 text-emerald-400" />
            <h2 className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider">
              Per-API-Key Usage Attribution
            </h2>
          </div>
          <button
            onClick={() => handleExportCsv('apikeys')}
            className="text-[11px] font-mono text-neutral-400 hover:text-neutral-200"
          >
            Export CSV
          </button>
        </div>

        <div className="border border-white/8 rounded overflow-hidden">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-canvas-dark text-neutral-400 border-b border-white/8 text-[11px]">
              <tr>
                <th className="py-2.5 px-3">KEY ID</th>
                <th className="py-2.5 px-3">NAME</th>
                <th className="py-2.5 px-3">TIER</th>
                <th className="py-2.5 px-3 text-right">BILLED TIME</th>
                <th className="py-2.5 px-3 text-right">SPEND (USD)</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-white/6 text-neutral-200">
              {apiKeysData.map((row) => (
                <tr key={row.keyId} className="hover:bg-white/2 transition-colors">
                  <td className="py-2.5 px-3 text-amber-safelight font-semibold">{row.keyId}</td>
                  <td className="py-2.5 px-3">{row.name}</td>
                  <td className="py-2.5 px-3">
                    <span className="px-1.5 py-0.5 rounded bg-white/6 text-neutral-400 text-[10px]">
                      {row.tier}
                    </span>
                  </td>
                  <td className="py-2.5 px-3 text-right">{row.usageSec}s</td>
                  <td className="py-2.5 px-3 text-right font-bold text-neutral-100">{row.spend}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
