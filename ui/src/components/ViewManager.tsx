// View manager that handles Dashboard and individual tab views

import { useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { TerminalView } from './Terminal';
import { SecretsManager } from './SecretsManager';
import { ServerDetail } from './ServerDetail';
import { SftpView } from './SftpView';
import { KubernetesView } from './KubernetesView';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';
import { useConnectionManager } from '../hooks/useConnectionManager';

function TileContent({ tabId }: { tabId: string }) {
  const { tabs } = useAppStore();
  const tab = tabs.find((t) => t.id === tabId);

  if (!tab) {
    return <div className="flex items-center justify-center h-full text-slate-500">Tab not found</div>;
  }

  switch (tab.type) {
    case 'terminal':
      return tab.sessionId ? (
        <TerminalView sessionId={tab.sessionId} />
      ) : (
        <div className="flex items-center justify-center h-full text-slate-500">No session</div>
      );
    case 'settings':
      if (tab.id === 'secrets') {
        return <SecretsManager />;
      }
      if (tab.serverId) {
        return <ServerDetail serverId={tab.serverId} />;
      }
      return <SettingsPanel />;
    case 'sftp':
      return <SftpView />;
    case 'k8s':
      return <KubernetesView />;
    default:
      return <div className="flex items-center justify-center h-full text-slate-500">Unknown tab type</div>;
  }
}

export function SettingsPanel() {
  const { theme, setTheme, fontSize, setFontSize, fontFamily, setFontFamily } = useAppStore();
  const [activeSection, setActiveSection] = useState<'general' | 'about'>('general');

  return (
    <div className="p-8 max-w-4xl mx-auto h-full overflow-y-auto">
      <h2 className="text-2xl font-bold mb-6 text-white text-left">Settings</h2>

      <div className="flex gap-4 mb-8 border-b border-white/5">
        {['general', 'about'].map((section) => (
          <button
            key={section}
            className={`pb-2 px-4 capitalize transition-colors ${activeSection === section ? 'text-primary border-b-2 border-primary' : 'text-slate-500 hover:text-slate-300'}`}
            onClick={() => setActiveSection(section as any)}
          >
            {section}
          </button>
        ))}
      </div>

      {activeSection === 'general' && (
        <div className="space-y-8 text-left">
          <section className="bg-background-card p-6 rounded-xl border border-white/5">
            <h3 className="text-sm font-semibold text-slate-500 uppercase tracking-wider mb-4">Appearance</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <label className="text-slate-300">Theme</label>
                <select
                  className="bg-background-dark border border-white/10 rounded px-3 py-1.5 text-sm text-white"
                  value={theme}
                  onChange={(e) => setTheme(e.target.value as 'dark' | 'light')}
                >
                  <option value="dark">Dark</option>
                  <option value="light">Light (Coming Soon)</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-slate-300">Font Size</label>
                <input
                  type="number"
                  className="bg-background-dark border border-white/10 rounded px-3 py-1.5 text-sm w-20 text-white"
                  value={fontSize}
                  onChange={(e) => setFontSize(parseInt(e.target.value))}
                  min={10}
                  max={24}
                />
              </div>
              <div className="flex items-center justify-between">
                <label className="text-slate-300">Font Family</label>
                <select
                  className="bg-background-dark border border-white/10 rounded px-3 py-1.5 text-sm text-white"
                  value={fontFamily}
                  onChange={(e) => setFontFamily(e.target.value)}
                >
                  <option value='Consolas, "Courier New", monospace'>Consolas</option>
                  <option value='"JetBrains Mono", monospace'>JetBrains Mono</option>
                  <option value='"Fira Code", monospace'>Fira Code</option>
                </select>
              </div>
            </div>
          </section>
        </div>
      )}

      {activeSection === 'about' && (
        <div className="bg-background-card p-6 rounded-xl border border-white/5 text-left">
           <h3 className="text-lg font-bold mb-2 text-white">Tacoshell v0.1.0</h3>
           <p className="text-slate-400">Unified Infrastructure Management GUI. Built with Rust + Tauri + React.</p>
        </div>
      )}
    </div>
  );
}

export function ViewManager() {
  const { tabs, activeTabId, servers, addTab, setActiveTab } = useAppStore();
  const {
    showAddDialog,
    setShowAddDialog,
    connectingServer,
    setConnectingServer,
    isConnecting,
    connectError,
    loadServers,
    handleConnect,
  } = useConnectionManager();

  if (!activeTabId) {
    return (
      <main className="flex-1 flex flex-col h-full overflow-hidden bg-background-dark relative">
        {/* Top Header */}
        <header className="h-16 px-6 flex items-center justify-between border-b border-white/5 bg-background-dark/80 backdrop-blur-md sticky top-0 z-20 shrink-0">
          {/* Search */}
          <div className="flex-1 max-w-2xl relative group">
            <span className="material-icons-round absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 group-focus-within:text-primary transition-colors">search</span>
            <input
              className="w-full bg-background-card border border-white/10 text-sm rounded-lg pl-10 pr-4 py-2 text-slate-300 placeholder-slate-500 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all"
              placeholder="Search hosts, IPs, or tags... (Cmd+K)"
              type="text"
            />
            <div className="absolute right-3 top-1/2 -translate-y-1/2 flex gap-1">
              <kbd className="hidden sm:inline-block px-1.5 py-0.5 text-xs text-slate-500 bg-white/5 border border-white/10 rounded font-mono">⌘K</kbd>
            </div>
          </div>
          {/* Actions */}
          <div className="flex items-center gap-3 ml-6">
            <button className="p-2 text-slate-400 hover:text-white hover:bg-white/5 rounded-lg transition-colors relative">
              <span className="material-icons-round">notifications</span>
            </button>
            <div className="h-6 w-px bg-white/10"></div>
            <button
              onClick={() => setShowAddDialog(true)}
              className="bg-primary hover:bg-primary-hover text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 shadow-lg shadow-primary/20 transition-all active:scale-95"
            >
              <span className="material-icons-round text-lg">add</span>
              New Connection
            </button>
          </div>
        </header>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
            <div className="flex items-center gap-1 bg-background-card p-1 rounded-lg border border-white/5 inline-flex w-fit">
              <button className="px-3 py-1.5 text-sm font-medium rounded text-white bg-white/10 shadow-sm">All Hosts</button>
              <button className="px-3 py-1.5 text-sm font-medium rounded text-slate-400 hover:text-white hover:bg-white/5 transition-colors">Recent</button>
              <button className="px-3 py-1.5 text-sm font-medium rounded text-slate-400 hover:text-white hover:bg-white/5 transition-colors">Favorites</button>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-sm text-slate-500">Sort by:</span>
              <select className="bg-transparent text-sm text-slate-300 border-none focus:ring-0 cursor-pointer p-0 font-medium outline-none">
                <option>Status</option>
                <option>Name</option>
                <option>Date Added</option>
              </select>
              <div className="flex bg-background-card rounded border border-white/5 ml-2">
                <button className="p-1.5 text-white bg-white/10 rounded-l"><span className="material-icons-round text-lg">grid_view</span></button>
                <button className="p-1.5 text-slate-500 hover:text-white rounded-r"><span className="material-icons-round text-lg">view_list</span></button>
              </div>
            </div>
          </div>

          {/* Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {servers.map((server) => (
              <div
                key={server.id}
                onClick={() => setConnectingServer(server)}
                className="group bg-background-card rounded-xl p-4 border border-white/5 hover:border-primary/50 transition-all duration-300 relative cursor-pointer"
              >
                <div className="flex justify-between items-start mb-3">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center border border-primary/20">
                      <span className="material-icons-round text-primary">terminal</span>
                    </div>
                    <div className="text-left">
                      <h3 className="font-semibold text-white group-hover:text-primary transition-colors">{server.name}</h3>
                      <p className="text-xs text-slate-500 flex items-center gap-1">
                        <span className="w-1.5 h-1.5 rounded-full bg-slate-500"></span>
                        Offline
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={(e) => {
                        e.stopPropagation();
                        const tabId = `server-${server.id}`;
                        addTab({ id: tabId, type: 'settings', title: `Edit: ${server.name}`, serverId: server.id });
                        setActiveTab(tabId);
                    }}
                    className="text-slate-500 hover:text-white p-1 rounded hover:bg-white/10 opacity-0 group-hover:opacity-100 transition-all"
                  >
                    <span className="material-icons-round text-xl">settings</span>
                  </button>
                </div>
                <div className="bg-black/30 rounded px-3 py-2 mb-3 font-mono text-xs text-slate-400 flex items-center justify-between">
                  <span>{server.host}</span>
                  <span className="text-slate-600">{server.port}</span>
                </div>
                <div className="flex items-center gap-2 mt-2">
                  <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-slate-700/30 text-slate-400 border border-slate-700/50 uppercase">{server.username}</span>
                </div>
              </div>
            ))}

            {/* Add New Card */}
            <button
              onClick={() => setShowAddDialog(true)}
              className="group bg-background-card/40 rounded-xl p-4 border border-dashed border-white/10 hover:border-primary hover:bg-primary/5 transition-all duration-300 flex flex-col items-center justify-center min-h-[160px] cursor-pointer"
            >
              <div className="w-12 h-12 rounded-full bg-white/5 group-hover:bg-primary/20 flex items-center justify-center mb-3 transition-colors">
                <span className="material-icons-round text-slate-400 group-hover:text-primary text-2xl">add</span>
              </div>
              <span className="text-sm font-medium text-slate-400 group-hover:text-white transition-colors">Add New Host</span>
            </button>
          </div>

          {/* Bottom Section / Info */}
          <div className="mt-8 mb-6 rounded-lg bg-gradient-to-r from-primary/20 to-transparent p-6 border border-primary/20 flex items-start sm:items-center justify-between gap-4">
            <div className="flex items-start gap-4 text-left">
              <div className="w-10 h-10 rounded-full bg-primary/20 flex items-center justify-center shrink-0">
                <span className="material-icons-round text-primary">tips_and_updates</span>
              </div>
              <div>
                <h4 className="text-white font-semibold">Pro Tip: Port Forwarding</h4>
                <p className="text-sm text-slate-400 mt-1 max-w-xl">You can now set up local port forwarding rules directly from the host context menu. Right-click any active connection to start.</p>
              </div>
            </div>
            <button className="text-sm font-medium text-white bg-primary hover:bg-primary-hover px-4 py-2 rounded-lg transition-colors whitespace-nowrap">Try it now</button>
          </div>
        </div>

        {/* Status Bar */}
        <footer className="h-8 bg-background-dark border-t border-white/5 flex items-center px-4 justify-between text-[11px] text-slate-500 select-none shrink-0">
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1.5 hover:text-slate-300 cursor-pointer transition-colors">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
              System Operational
            </span>
          </div>
          <div className="flex items-center gap-4">
            <span className="hover:text-slate-300 cursor-pointer transition-colors">v0.1.0 (Stable)</span>
            <span className="hover:text-slate-300 cursor-pointer transition-colors flex items-center gap-1">
              <span className="material-icons-round text-[12px]">wifi</span> 14ms
            </span>
          </div>
        </footer>

        {showAddDialog && (
          <AddServerDialog
            onClose={() => setShowAddDialog(false)}
            onAdded={loadServers}
          />
        )}

        {connectingServer && (
          <ConnectDialog
            server={connectingServer}
            onConnect={handleConnect}
            onCancel={() => setConnectingServer(null)}
            loading={isConnecting}
            error={connectError}
          />
        )}
      </main>
    );
  }

  return (
    <div className="flex-1 overflow-hidden bg-background-dark">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className="h-full w-full"
          style={{
            display: tab.id === activeTabId ? 'block' : 'none',
          }}
        >
          <TileContent tabId={tab.id} />
        </div>
      ))}
    </div>
  );
}
