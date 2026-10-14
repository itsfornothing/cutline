import React from 'react';
import {
  Video,
  Activity,
  GitCompare,
  BarChart3,
  BookOpen,
  Command,
  HelpCircle,
  Sun,
  Moon,
  Sparkles,
} from 'lucide-react';

interface HeaderProps {
  activeTab: string;
  onSelectTab: (tab: string) => void;
  onOpenCommandPalette: () => void;
  onOpenShortcuts: () => void;
  isDark: boolean;
  onToggleTheme: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  activeTab,
  onSelectTab,
  onOpenCommandPalette,
  onOpenShortcuts,
  isDark,
  onToggleTheme,
}) => {
  const tabs = [
    { id: 'studio', label: 'Template Studio', icon: Video },
    { id: 'observatory', label: 'Render Observatory', icon: Activity },
    { id: 'ci', label: 'Template CI', icon: GitCompare },
    { id: 'usage', label: 'Usage & Metering', icon: BarChart3 },
    { id: 'docs', label: 'API Reference', icon: BookOpen },
  ];

  return (
    <header className="h-12 border-b border-white/8 bg-panel-dark px-4 flex items-center justify-between select-none">
      {/* Brand & Engine Version */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 font-mono font-bold tracking-tight text-sm text-neutral-100">
          <span className="w-2.5 h-2.5 rounded-full bg-amber-safelight animate-pulse" />
          <span>CUTLINE</span>
        </div>
        <span className="text-[11px] font-mono px-1.5 py-0.5 rounded bg-white/6 text-neutral-400 border border-white/6">
          v0.1.0-wgpu
        </span>
      </div>

      {/* Surface Navigation Tabs */}
      <nav className="flex items-center gap-1">
        {tabs.map((tab) => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.id;
          return (
            <button
              key={tab.id}
              onClick={() => onSelectTab(tab.id)}
              className={`px-3 py-1.5 rounded text-xs font-medium flex items-center gap-2 transition-all duration-fast ${
                isActive
                  ? 'bg-amber-safelight/10 text-amber-safelight border border-amber-safelight/30 font-semibold'
                  : 'text-neutral-400 hover:text-neutral-200 hover:bg-white/4'
              }`}
            >
              <Icon className="w-3.5 h-3.5" />
              <span>{tab.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Utility Actions */}
      <div className="flex items-center gap-2">
        <button
          onClick={onOpenCommandPalette}
          className="px-2 py-1 bg-white/4 hover:bg-white/8 text-neutral-400 hover:text-neutral-200 border border-white/8 rounded text-xs font-mono flex items-center gap-1.5 transition-colors duration-fast"
          title="Command Palette (⌘K)"
        >
          <Command className="w-3 h-3 text-amber-safelight" />
          <span className="text-[11px]">⌘K</span>
        </button>

        <button
          onClick={onOpenShortcuts}
          className="p-1.5 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded transition-colors duration-fast"
          title="Keyboard Shortcuts (?)"
        >
          <HelpCircle className="w-4 h-4" />
        </button>

        <button
          onClick={onToggleTheme}
          className="p-1.5 text-neutral-400 hover:text-neutral-200 hover:bg-white/6 rounded transition-colors duration-fast"
          title="Toggle Theme"
        >
          {isDark ? <Sun className="w-4 h-4 text-amber-safelight" /> : <Moon className="w-4 h-4" />}
        </button>
      </div>
    </header>
  );
};
