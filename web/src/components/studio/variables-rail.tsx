import React, { useState } from 'react';
import {
  Variable,
  FileSpreadsheet,
  FolderOpen,
  Plus,
  RefreshCw,
  ExternalLink,
} from 'lucide-react';
import { TimelineData } from '../../lib/canvas-renderer';

interface VariablesRailProps {
  timeline: TimelineData;
  variables: Record<string, string>;
  onVariableChange: (key: string, val: string) => void;
  onSelectCsvRow?: (rowIdx: number) => void;
}

export const VariablesRail: React.FC<VariablesRailProps> = ({
  timeline,
  variables,
  onVariableChange,
  onSelectCsvRow,
}) => {
  const [activeTab, setActiveTab] = useState<'vars' | 'assets'>('vars');
  const [selectedCsvRow, setSelectedCsvRow] = useState<number>(1);

  // Mock CSV Rows for "preview with row N of CSV" selector
  const csvRows = [
    { row: 1, name: 'Alex Vance', discount: '20% OFF', code: 'LAUNCH20' },
    { row: 2, name: 'Jordan Chen', discount: '35% OFF', code: 'SUMMER35' },
    { row: 3, name: 'Taylor Swift', discount: '50% VIP', code: 'VIPPASS' },
  ];

  const handleCsvSelect = (rowIdx: number) => {
    setSelectedCsvRow(rowIdx);
    const row = csvRows[rowIdx - 1];
    if (row && onSelectCsvRow) {
      onSelectCsvRow(rowIdx);
      if ('title' in variables) onVariableChange('title', `Exclusive for ${row.name}`);
      if ('speaker' in variables) onVariableChange('speaker', row.name);
    }
  };

  return (
    <aside className="w-64 bg-panel-dark flex flex-col border-r border-white/8 select-none text-xs">
      {/* Tab Switcher */}
      <div className="h-9 border-b border-white/8 flex items-center px-2 gap-1 bg-panel-dark/50">
        <button
          onClick={() => setActiveTab('vars')}
          className={`flex-1 py-1 rounded text-xs font-medium flex items-center justify-center gap-1.5 transition-colors ${
            activeTab === 'vars'
              ? 'bg-white/8 text-neutral-100 font-semibold'
              : 'text-neutral-400 hover:text-neutral-200'
          }`}
        >
          <Variable className="w-3.5 h-3.5 text-amber-safelight" />
          <span>Variables</span>
        </button>
        <button
          onClick={() => setActiveTab('assets')}
          className={`flex-1 py-1 rounded text-xs font-medium flex items-center justify-center gap-1.5 transition-colors ${
            activeTab === 'assets'
              ? 'bg-white/8 text-neutral-100 font-semibold'
              : 'text-neutral-400 hover:text-neutral-200'
          }`}
        >
          <FolderOpen className="w-3.5 h-3.5 text-blue-400" />
          <span>Assets ({Object.keys(timeline.assets || {}).length})</span>
        </button>
      </div>

      {/* Variables View */}
      {activeTab === 'vars' ? (
        <div className="flex-1 overflow-y-auto p-3 space-y-4">
          {/* CSV Batch Row Selector */}
          <div className="p-2.5 rounded bg-white/4 border border-white/6 space-y-2">
            <div className="flex items-center justify-between text-neutral-300">
              <span className="font-mono text-[11px] flex items-center gap-1">
                <FileSpreadsheet className="w-3.5 h-3.5 text-emerald-400" />
                <span>CSV PREVIEW</span>
              </span>
              <span className="text-[10px] font-mono text-neutral-500">3 Rows</span>
            </div>
            <div className="flex items-center gap-1">
              {[1, 2, 3].map((r) => (
                <button
                  key={r}
                  onClick={() => handleCsvSelect(r)}
                  className={`flex-1 py-1 rounded text-center font-mono text-[11px] border transition-colors ${
                    selectedCsvRow === r
                      ? 'bg-amber-safelight/15 text-amber-safelight border-amber-safelight/40 font-bold'
                      : 'text-neutral-400 border-white/6 hover:bg-white/4'
                  }`}
                >
                  Row {r}
                </button>
              ))}
            </div>
          </div>

          {/* Variables Form */}
          <div className="space-y-3">
            <div className="flex items-center justify-between text-neutral-400 font-mono text-[11px]">
              <span>TEMPLATE VARIABLES</span>
              <span className="text-[10px] text-neutral-600">Schema Typed</span>
            </div>

            {Object.keys(variables).length === 0 ? (
              <div className="text-center py-6 text-neutral-500 font-mono text-[11px]">
                No variables defined in timeline
              </div>
            ) : (
              Object.entries(variables).map(([key, val]) => (
                <div key={key} className="space-y-1">
                  <label className="block text-[11px] font-mono text-neutral-400">{`{{${key}}}`}</label>
                  <input
                    type="text"
                    value={val}
                    onChange={(e) => onVariableChange(key, e.target.value)}
                    className="w-full bg-canvas-dark border border-white/10 rounded px-2 py-1.5 text-xs text-neutral-100 font-mono outline-hidden focus:border-amber-safelight"
                  />
                </div>
              ))
            )}
          </div>
        </div>
      ) : (
        /* Assets View */
        <div className="flex-1 overflow-y-auto p-3 space-y-2">
          <div className="flex items-center justify-between text-neutral-400 font-mono text-[11px] mb-2">
            <span>INGESTED ASSETS</span>
            <span className="text-[10px] text-neutral-600">S3 / BYO Bucket</span>
          </div>

          {Object.entries(timeline.assets || {}).map(([id, asset]) => (
            <div
              key={id}
              className="p-2 rounded bg-white/4 border border-white/6 space-y-1 text-xs font-mono"
            >
              <div className="flex items-center justify-between">
                <span className="font-semibold text-neutral-200">{id}</span>
                <span className="text-[10px] uppercase px-1 py-0.2 rounded bg-white/6 text-neutral-400">
                  {asset.type}
                </span>
              </div>
              <div className="text-[11px] text-neutral-400 truncate">{asset.src}</div>
              {asset.duration && (
                <div className="text-[10px] text-neutral-500">
                  Duration: {asset.duration.toFixed(1)}s
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </aside>
  );
};
