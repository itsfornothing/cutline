import React from 'react';
import { X, Keyboard } from 'lucide-react';

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ShortcutsModal: React.FC<ShortcutsModalProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  const shortcutGroups = [
    {
      title: 'Timeline Navigation',
      shortcuts: [
        { key: 'Space', desc: 'Play / Pause playback' },
        { key: 'J / K / L', desc: 'Reverse / Pause / Fast-forward shuttle' },
        { key: '← / →', desc: 'Step backward / forward 1 frame' },
        { key: 'Shift + ← / →', desc: 'Step backward / forward 1 second' },
        { key: 'Home / End', desc: 'Jump to timeline start / end' },
      ],
    },
    {
      title: 'Editor & Validation',
      shortcuts: [
        { key: '⌘ + Enter', desc: 'Execute Dry-Run Quote (:validate)' },
        { key: '⌘ + S', desc: 'Save Template changes' },
        { key: '⌘ + K', desc: 'Open Command Palette' },
        { key: '?', desc: 'Toggle keyboard shortcut cheat sheet' },
        { key: 'Esc', desc: 'Close open overlay or modal' },
      ],
    },
    {
      title: 'Safe Zone & Overlays',
      shortcuts: [
        { key: 'Z', desc: 'Toggle platform safe zone overlay' },
        { key: 'M', desc: 'Add magnetic edit marker' },
      ],
    },
  ];

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-lg bg-panel-dark border border-white/12 rounded-lg shadow-2xl p-6 overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between pb-4 border-b border-white/8">
          <div className="flex items-center gap-2 text-sm font-semibold text-neutral-100">
            <Keyboard className="w-4 h-4 text-amber-safelight" />
            <span>Keyboard Shortcuts</span>
          </div>
          <button
            onClick={onClose}
            className="text-neutral-400 hover:text-neutral-200 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="mt-4 space-y-6 max-h-[70vh] overflow-y-auto pr-1">
          {shortcutGroups.map((group) => (
            <div key={group.title}>
              <h4 className="text-xs font-mono text-neutral-400 uppercase tracking-wider mb-2">
                {group.title}
              </h4>
              <div className="space-y-1.5">
                {group.shortcuts.map((sc) => (
                  <div
                    key={sc.key}
                    className="flex items-center justify-between py-1 px-2 rounded bg-white/4 text-xs"
                  >
                    <span className="text-neutral-300">{sc.desc}</span>
                    <kbd className="px-2 py-0.5 rounded bg-white/8 text-neutral-200 font-mono text-[11px] border border-white/10">
                      {sc.key}
                    </kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
