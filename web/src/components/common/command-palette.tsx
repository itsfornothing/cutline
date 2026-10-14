import React, { useState, useEffect } from 'react';
import {
  Search,
  Video,
  FileCheck,
  CreditCard,
  BookOpen,
  Sun,
  Moon,
  Command,
  X,
  PlusCircle,
  Play,
} from 'lucide-react';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  onNavigate: (tab: string) => void;
  onNewRender?: () => void;
  onToggleTheme: () => void;
  isDark: boolean;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  onNavigate,
  onNewRender,
  onToggleTheme,
  isDark,
}) => {
  const [query, setQuery] = useState('');

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen) onClose();
        else setQuery('');
      } else if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const actions = [
    {
      id: 'studio',
      label: 'Open Template Studio',
      category: 'Navigation',
      icon: <Video className="w-4 h-4 text-amber-safelight" />,
      action: () => {
        onNavigate('studio');
        onClose();
      },
    },
    {
      id: 'observatory',
      label: 'Open Render Observatory',
      category: 'Navigation',
      icon: <Play className="w-4 h-4 text-emerald-400" />,
      action: () => {
        onNavigate('observatory');
        onClose();
      },
    },
    {
      id: 'ci',
      label: 'Open Template CI Golden Diff',
      category: 'Navigation',
      icon: <FileCheck className="w-4 h-4 text-blue-400" />,
      action: () => {
        onNavigate('ci');
        onClose();
      },
    },
    {
      id: 'usage',
      label: 'View Usage & Metered Billing',
      category: 'Navigation',
      icon: <CreditCard className="w-4 h-4 text-amber-400" />,
      action: () => {
        onNavigate('usage');
        onClose();
      },
    },
    {
      id: 'docs',
      label: 'Explore API Docs & Spec',
      category: 'Navigation',
      icon: <BookOpen className="w-4 h-4 text-neutral-400" />,
      action: () => {
        onNavigate('docs');
        onClose();
      },
    },
    {
      id: 'theme',
      label: isDark ? 'Switch to Warm Paper Light Theme' : 'Switch to Film Lab Dark Theme',
      category: 'Preferences',
      icon: isDark ? <Sun className="w-4 h-4 text-amber-safelight" /> : <Moon className="w-4 h-4" />,
      action: () => {
        onToggleTheme();
        onClose();
      },
    },
    {
      id: 'new_render',
      label: 'Submit New Render (POST /v1/renders)',
      category: 'Actions',
      icon: <PlusCircle className="w-4 h-4 text-amber-safelight" />,
      action: () => {
        if (onNewRender) onNewRender();
        onClose();
      },
    },
  ];

  const filtered = actions.filter((a) =>
    a.label.toLowerCase().includes(query.toLowerCase()) ||
    a.category.toLowerCase().includes(query.toLowerCase())
  );

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-xs flex items-start justify-center pt-24 px-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl bg-panel-dark border border-white/12 rounded-lg shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center px-4 py-3 border-b border-white/10 gap-3">
          <Search className="w-4 h-4 text-neutral-400" />
          <input
            type="text"
            placeholder="Type a command or jump to surface... (⌘K)"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            className="flex-1 bg-transparent border-0 outline-hidden text-sm text-neutral-100 placeholder:text-neutral-500 font-mono"
            autoFocus
          />
          <kbd className="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono bg-white/6 text-neutral-400 border border-white/10 rounded">
            ESC
          </kbd>
        </div>

        <div className="max-h-80 overflow-y-auto py-2">
          {filtered.length === 0 ? (
            <div className="p-4 text-center text-xs text-neutral-500 font-mono">
              No matching commands
            </div>
          ) : (
            filtered.map((item, idx) => (
              <button
                key={item.id}
                onClick={item.action}
                className="w-full px-4 py-2.5 flex items-center justify-between text-left text-xs hover:bg-white/6 transition-colors duration-fast group"
              >
                <div className="flex items-center gap-3">
                  {item.icon}
                  <span className="text-neutral-200 group-hover:text-amber-safelight font-medium">
                    {item.label}
                  </span>
                </div>
                <span className="text-[11px] font-mono text-neutral-500">{item.category}</span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
