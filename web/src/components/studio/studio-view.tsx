import React, { useState, useEffect } from 'react';
import {
  Code,
  Sliders,
  Subtitles,
  DollarSign,
  Play,
  RotateCcw,
  Sparkles,
  ChevronDown,
} from 'lucide-react';
import { PreviewCanvas } from './preview-canvas';
import { TimelineRuler } from './timeline-ruler';
import { VariablesRail } from './variables-rail';
import { CaptionsInspector } from './captions-inspector';
import { JsonEditorTab } from './json-editor-tab';
import { CostQuoteModal } from './cost-quote-modal';
import { SAMPLE_TEMPLATES, SampleTemplate } from '../../lib/sample-templates';
import { TimelineData } from '../../lib/canvas-renderer';

interface StudioViewProps {
  onTriggerRender?: (timeline: any) => void;
}

export const StudioView: React.FC<StudioViewProps> = ({ onTriggerRender }) => {
  const [selectedTemplate, setSelectedTemplate] = useState<SampleTemplate>(SAMPLE_TEMPLATES[0]);
  const [timeline, setTimeline] = useState<TimelineData>(SAMPLE_TEMPLATES[0].timeline);
  const [variables, setVariables] = useState<Record<string, string>>({
    title: 'Why I Built Cutline',
    speaker: 'Founder',
  });
  const [currentTime, setCurrentTime] = useState<number>(1.5);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const [inspectorTab, setInspectorTab] = useState<'captions' | 'json' | 'properties'>('captions');
  const [safeZone, setSafeZone] = useState<'none' | 'tiktok' | 'reels' | 'shorts'>('tiktok');
  const [isQuoteOpen, setIsQuoteOpen] = useState<boolean>(false);

  // Template switch handler
  const handleTemplateChange = (tpl: SampleTemplate) => {
    setSelectedTemplate(tpl);
    setTimeline(tpl.timeline);
    setVariables(tpl.timeline.variables || {});
    setCurrentTime(1.0);
    setIsPlaying(false);
  };

  const handleVariableChange = (key: string, val: string) => {
    setVariables((prev) => ({ ...prev, [key]: val }));
    // Update timeline JSON
    setTimeline((prev) => ({
      ...prev,
      variables: {
        ...(prev.variables || {}),
        [key]: val,
      },
    }));
  };

  // Keyboard shortcut listener for space and dry run quote
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }
      if (e.code === 'Space') {
        e.preventDefault();
        setIsPlaying((p) => !p);
      } else if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault();
        setIsQuoteOpen(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const captionTrack = timeline.tracks.find((t) => t.kind === 'captions');
  const captionStyle = captionTrack?.style || {};

  const handleCaptionStyleChange = (updatedStyle: any) => {
    setTimeline((prev) => ({
      ...prev,
      tracks: prev.tracks.map((t) => (t.kind === 'captions' ? { ...t, style: updatedStyle } : t)),
    }));
  };

  return (
    <div className="flex-1 flex flex-col h-[calc(100vh-3rem)] overflow-hidden select-none bg-canvas-dark">
      {/* Studio Top Control Strip */}
      <div className="h-10 border-b border-white/8 px-4 flex items-center justify-between bg-panel-dark text-xs">
        {/* Template Selector Dropdown */}
        <div className="flex items-center gap-2">
          <span className="text-[11px] font-mono text-neutral-500 uppercase">Template:</span>
          <div className="relative">
            <select
              value={selectedTemplate.id}
              onChange={(e) => {
                const found = SAMPLE_TEMPLATES.find((t) => t.id === e.target.value);
                if (found) handleTemplateChange(found);
              }}
              className="appearance-none bg-white/4 border border-white/10 rounded px-2.5 py-1 pr-7 text-xs font-mono text-neutral-100 outline-hidden hover:border-white/20 cursor-pointer"
            >
              {SAMPLE_TEMPLATES.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name} ({t.aspect})
                </option>
              ))}
            </select>
            <ChevronDown className="w-3.5 h-3.5 text-neutral-400 absolute right-2 top-2 pointer-events-none" />
          </div>
        </div>

        {/* Action Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setIsQuoteOpen(true)}
            className="px-3 py-1 rounded bg-white/6 hover:bg-white/10 text-neutral-200 border border-white/10 font-mono text-xs flex items-center gap-1.5 transition-colors"
            title="Dry-run quote & validation (⌘Enter)"
          >
            <DollarSign className="w-3.5 h-3.5 text-amber-safelight" />
            <span>Dry-Run Quote</span>
          </button>

          <button
            onClick={() => {
              if (onTriggerRender) onTriggerRender(timeline);
            }}
            className="px-3 py-1 rounded bg-amber-safelight hover:bg-amber-highlight text-neutral-950 font-bold text-xs flex items-center gap-1.5 transition-colors"
          >
            <Play className="w-3.5 h-3.5 fill-current" />
            <span>Render MP4</span>
          </button>
        </div>
      </div>

      {/* Main 4-Region Body */}
      <div className="flex-1 flex overflow-hidden">
        {/* Region 1: Left Variables & Asset Rail */}
        <VariablesRail
          timeline={timeline}
          variables={variables}
          onVariableChange={handleVariableChange}
        />

        {/* Region 2: Center Preview Canvas */}
        <PreviewCanvas
          timeline={timeline}
          currentTime={currentTime}
          duration={selectedTemplate.duration}
          isPlaying={isPlaying}
          onTimeChange={setCurrentTime}
          onTogglePlay={() => setIsPlaying(!isPlaying)}
          safeZone={safeZone}
          onSafeZoneChange={setSafeZone}
        />

        {/* Region 4: Right Inspector & JSON Schema Panel */}
        <div className="w-80 bg-panel-dark border-l border-white/8 flex flex-col overflow-hidden">
          {/* Tab Bar */}
          <div className="h-9 border-b border-white/8 flex items-center px-2 gap-1 bg-panel-dark/50 text-xs">
            <button
              onClick={() => setInspectorTab('captions')}
              className={`flex-1 py-1 rounded text-xs font-medium flex items-center justify-center gap-1 transition-colors ${
                inspectorTab === 'captions'
                  ? 'bg-white/8 text-neutral-100 font-semibold'
                  : 'text-neutral-400 hover:text-neutral-200'
              }`}
            >
              <Subtitles className="w-3.5 h-3.5 text-amber-safelight" />
              <span>Captions</span>
            </button>
            <button
              onClick={() => setInspectorTab('json')}
              className={`flex-1 py-1 rounded text-xs font-medium flex items-center justify-center gap-1 transition-colors ${
                inspectorTab === 'json'
                  ? 'bg-white/8 text-neutral-100 font-semibold'
                  : 'text-neutral-400 hover:text-neutral-200'
              }`}
            >
              <Code className="w-3.5 h-3.5 text-blue-400" />
              <span>JSON Schema</span>
            </button>
          </div>

          {/* Inspector Content */}
          <div className="flex-1 overflow-y-auto p-4">
            {inspectorTab === 'captions' ? (
              <CaptionsInspector
                style={captionStyle}
                onStyleChange={handleCaptionStyleChange}
                safeZone={safeZone}
                onSafeZoneChange={setSafeZone}
              />
            ) : (
              <JsonEditorTab timeline={timeline} onChange={setTimeline} />
            )}
          </div>
        </div>
      </div>

      {/* Region 3: Bottom Virtualized SMPTE Timeline */}
      <TimelineRuler
        timeline={timeline}
        currentTime={currentTime}
        duration={selectedTemplate.duration}
        onTimeChange={setCurrentTime}
        selectedClipId={selectedClipId}
        onSelectClip={setSelectedClipId}
      />

      {/* Dry Run Cost Quote Modal */}
      <CostQuoteModal
        isOpen={isQuoteOpen}
        onClose={() => setIsQuoteOpen(false)}
        timeline={timeline}
        onConfirmRender={() => {
          if (onTriggerRender) onTriggerRender(timeline);
        }}
      />
    </div>
  );
};
