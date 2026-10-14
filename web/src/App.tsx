import React, { useState, useEffect } from 'react';
import { Header } from './components/common/header';
import { CommandPalette } from './components/common/command-palette';
import { ShortcutsModal } from './components/common/shortcuts-modal';
import { StudioView } from './components/studio/studio-view';
import { RenderObservatory } from './components/observatory/render-observatory';
import { TemplateCiView } from './components/ci/template-ci-view';
import { UsageView } from './components/usage/usage-view';
import { ApiDocsView } from './components/docs/api-docs-view';
import { MarketingView } from './components/marketing/marketing-view';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<string>('studio');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState<boolean>(false);
  const [isShortcutsOpen, setIsShortcutsOpen] = useState<boolean>(false);
  const [isDark, setIsDark] = useState<boolean>(true);

  // Keyboard shortcut listener for ? (help) and ⌘K
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }
      if (e.key === '?' && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        setIsShortcutsOpen((prev) => !prev);
      } else if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleToggleTheme = () => {
    setIsDark((prev) => {
      const next = !prev;
      if (next) {
        document.documentElement.classList.add('dark');
        document.documentElement.classList.remove('light');
      } else {
        document.documentElement.classList.remove('dark');
        document.documentElement.classList.add('light');
      }
      return next;
    });
  };

  return (
    <div className={`min-h-screen flex flex-col font-sans ${isDark ? 'dark bg-canvas-dark text-neutral-100' : 'light bg-canvas-light text-neutral-900'}`}>
      {/* Top Application Header */}
      <Header
        activeTab={activeTab}
        onSelectTab={setActiveTab}
        onOpenCommandPalette={() => setIsCommandPaletteOpen(true)}
        onOpenShortcuts={() => setIsShortcutsOpen(true)}
        isDark={isDark}
        onToggleTheme={handleToggleTheme}
      />

      {/* Main Surface View */}
      <main className="flex-1 flex overflow-hidden">
        {activeTab === 'studio' && <StudioView />}
        {activeTab === 'observatory' && <RenderObservatory />}
        {activeTab === 'ci' && <TemplateCiView />}
        {activeTab === 'usage' && <UsageView />}
        {activeTab === 'docs' && <ApiDocsView />}
        {activeTab === 'marketing' && <MarketingView onOpenStudio={() => setActiveTab('studio')} />}
      </main>

      {/* Command Palette Modal (⌘K) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        onNavigate={setActiveTab}
        onToggleTheme={handleToggleTheme}
        isDark={isDark}
      />

      {/* Shortcuts Modal (?) */}
      <ShortcutsModal
        isOpen={isShortcutsOpen}
        onClose={() => setIsShortcutsOpen(false)}
      />
    </div>
  );
};
export default App;
