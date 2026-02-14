// Split pane layout refactored as Dashboard / Main Content area

import { useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { TerminalView } from './Terminal';
import { SecretsManager } from './SecretsManager';
import { ServerDetail } from './ServerDetail';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';
import { connectSsh } from '../hooks/useTauri';
import type { Tab, Server as ServerType } from '../types';

interface SplitPaneLayoutProps {
  tabs: Tab[];
}

function TileContent({ tabId }: { tabId: string }) {
  const { tabs } = useAppStore();
  const tab = tabs.find((t) => t.id === tabId);

  if (!tab) {
    return <div className="tile-empty">Tab not found</div>;
  }

  switch (tab.type) {
    case 'terminal':
      return tab.sessionId ? (
        <TerminalView sessionId={tab.sessionId} />
      ) : (
        <div className="tile-empty">No session</div>
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
      return <SftpView tab={tab} />;
    case 'k8s':
      return <K8sView tab={tab} />;
    default:
      return <div className="tile-empty">Unknown tab type</div>;
  }
}

export function SettingsPanel() {
  const { theme, setTheme, fontSize, setFontSize } = useAppStore();
  const [activeSection, setActiveSection] = useState<'general' | 'secrets' | 'about'>('general');

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">⚙️ Settings</h2>

      <div className="flex gap-4 mb-8 border-b border-border-color">
        {['general', 'secrets', 'about'].map((section) => (
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
        <div className="space-y-8">
          <section className="bg-background-card p-6 rounded-xl border border-white/5">
            <h3 className="text-sm font-semibold text-slate-500 uppercase tracking-wider mb-4">Appearance</h3>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <label className="text-slate-300">Theme</label>
                <select className="bg-background-dark border border-white/10 rounded px-3 py-1.5 text-sm" value={theme} onChange={(e) => setTheme(e.target.value as 'dark' | 'light')}>
                  <option value="dark">Dark</option>
                  <option value="light">Light</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <label className="text-slate-300">Font Size</label>
                <input
                  type="number"
                  className="bg-background-dark border border-white/10 rounded px-3 py-1.5 text-sm w-20"
                  value={fontSize}
                  onChange={(e) => setFontSize(parseInt(e.target.value))}
                  min={10}
                  max={24}
                />
              </div>
            </div>
          </section>
        </div>
      )}

      {activeSection === 'secrets' && <SecretsManager />}

      {activeSection === 'about' && (
        <div className="bg-background-card p-6 rounded-xl border border-white/5">
           <h3 className="text-lg font-bold mb-2">Tacoshell v0.1.0</h3>
           <p className="text-slate-400">Unified Infrastructure Management GUI. Built with Rust + Tauri + React.</p>
        </div>
      )}
    </div>
  );
}

function SftpView({ tab }: { tab: Tab }) {
  console.log('SFTP View for', tab.title);
  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden bg-background-dark relative font-display">
      <header className="h-16 border-b dark:border-border-dark flex items-center justify-between px-6 bg-white dark:bg-panel-dark/50 backdrop-blur-sm">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span className="material-icons-round text-green-500 text-sm">circle</span>
            <h1 className="font-semibold text-slate-800 dark:text-white text-lg">{tab.title}</h1>
          </div>
          <span className="px-2 py-0.5 rounded text-xs font-medium bg-slate-100 dark:bg-slate-800 text-slate-500">192.168.1.55</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative">
            <span className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none text-slate-500">
              <span className="material-icons-round text-lg">search</span>
            </span>
            <input className="pl-9 pr-4 py-1.5 bg-slate-100 dark:bg-[#0c1017] border-none rounded-lg text-sm text-slate-700 dark:text-slate-200 focus:ring-1 focus:ring-primary w-64 placeholder-slate-500 outline-none" placeholder="Find file..." type="text"/>
          </div>
        </div>
      </header>

      <div className="h-14 border-b dark:border-border-dark flex items-center justify-between px-6 bg-white dark:bg-background-dark/50">
        <div className="flex items-center gap-1">
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-panel-dark rounded-lg transition-colors">
            <span className="material-icons-round text-lg text-primary">upload</span>
            Upload
          </button>
          <button className="flex items-center gap-2 px-3 py-1.5 text-sm font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-panel-dark rounded-lg transition-colors">
            <span className="material-icons-round text-lg text-primary">download</span>
            Download
          </button>
          <div className="w-px h-6 bg-slate-200 dark:bg-border-dark mx-2"></div>
          <button className="p-2 text-slate-500 hover:text-white hover:bg-slate-800 rounded-lg"><span className="material-icons-round text-xl">create_new_folder</span></button>
          <button className="p-2 text-slate-500 hover:text-white hover:bg-slate-800 rounded-lg"><span className="material-icons-round text-xl">delete_outline</span></button>
        </div>
      </div>

      <div className="flex-1 flex overflow-hidden">
        {/* Local Pane */}
        <section className="flex-1 flex flex-col border-r dark:border-border-dark min-w-[300px]">
          <div className="px-4 py-3 bg-slate-50 dark:bg-panel-dark/30 border-b dark:border-border-dark flex items-center gap-2 text-sm">
            <span className="material-icons-round text-slate-500">laptop_mac</span>
            <span className="text-slate-400">Local:</span>
            <div className="flex items-center gap-1 overflow-hidden text-slate-300">
              <span className="font-medium text-white">/Users/tacoshell/Projects</span>
            </div>
          </div>
          <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b dark:border-border-dark bg-slate-50 dark:bg-background-dark text-xs font-semibold text-slate-500 uppercase tracking-wider">
            <div className="col-span-6">Name</div>
            <div className="col-span-3 text-right">Size</div>
            <div className="col-span-3 text-right">Date</div>
          </div>
          <div className="flex-1 overflow-y-auto p-2">
            <div className="grid grid-cols-12 gap-4 px-3 py-2 rounded items-center bg-primary/20 border border-primary/20 cursor-pointer">
              <div className="col-span-6 flex items-center gap-3 overflow-hidden">
                <span className="material-icons-round text-blue-400 text-xl">description</span>
                <span className="text-sm text-white font-medium truncate">index.html</span>
              </div>
              <div className="col-span-3 text-right text-xs text-slate-300 font-mono">24 KB</div>
              <div className="col-span-3 text-right text-xs text-slate-300">Today, 14:20</div>
            </div>
          </div>
        </section>

        {/* Remote Pane */}
        <section className="flex-1 flex flex-col bg-slate-50/50 dark:bg-panel-dark/20 min-w-[300px]">
          <div className="px-4 py-3 bg-slate-50 dark:bg-panel-dark/30 border-b dark:border-border-dark flex items-center gap-2 text-sm">
            <span className="material-icons-round text-primary">dns</span>
            <span className="text-slate-400">Remote:</span>
            <div className="flex items-center gap-1 overflow-hidden text-slate-300">
              <span className="font-medium text-white">/var/www/html</span>
            </div>
          </div>
          <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b dark:border-border-dark bg-slate-50 dark:bg-background-dark text-xs font-semibold text-slate-500 uppercase tracking-wider">
            <div className="col-span-6">Name</div>
            <div className="col-span-2 text-right">Perms</div>
            <div className="col-span-2 text-right">Size</div>
            <div className="col-span-2 text-right">Owner</div>
          </div>
          <div className="flex-1 overflow-y-auto p-2 space-y-0.5">
            <div className="grid grid-cols-12 gap-4 px-3 py-2 rounded items-center hover:bg-primary/10 cursor-pointer group transition-colors">
              <div className="col-span-6 flex items-center gap-3 overflow-hidden">
                <span className="material-icons-round text-yellow-500 dark:text-yellow-400/90 text-xl">folder</span>
                <span className="text-sm text-slate-700 dark:text-slate-200 truncate group-hover:text-primary">images</span>
              </div>
              <div className="col-span-2 text-right text-xs text-slate-500 font-mono">755</div>
              <div className="col-span-2 text-right text-xs text-slate-500">-</div>
              <div className="col-span-2 text-right text-xs text-slate-500">root</div>
            </div>
          </div>
        </section>
      </div>

      {/* Transfer Queue */}
      <div className="h-auto border-t dark:border-border-dark bg-white dark:bg-panel-dark z-10 shadow-[0_-5px_15px_rgba(0,0,0,0.3)]">
        <div className="px-4 py-2 flex items-center justify-between cursor-pointer hover:bg-white/5">
          <div className="flex items-center gap-3">
            <span className="material-icons-round text-primary text-sm animate-pulse">sync</span>
            <span className="text-sm font-medium text-slate-800 dark:text-white">Active Transfers</span>
            <span className="bg-primary/20 text-primary text-xs px-2 py-0.5 rounded-full font-bold">1</span>
          </div>
          <div className="flex items-center gap-4 text-xs text-slate-500">
            <span>2.5 MB/s</span>
            <span className="material-icons-round">expand_more</span>
          </div>
        </div>
      </div>
    </div>
  );
}

function K8sView({ tab }: { tab: Tab }) {
  console.log('K8s View for', tab.title);
  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden bg-background-light dark:bg-background-dark relative">
      <header className="h-16 border-b border-slate-200 dark:border-slate-800 bg-white/50 dark:bg-surface-darker/50 backdrop-blur-sm flex items-center justify-between px-6 z-10">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-semibold text-slate-800 dark:text-white tracking-tight">Kubernetes Clusters</h1>
          <span className="px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium border border-primary/20">3 Active</span>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative group">
            <span className="material-icons-round absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 text-lg group-focus-within:text-primary transition-colors">search</span>
            <input className="pl-10 pr-4 py-2 bg-slate-100 dark:bg-surface-dark border border-transparent dark:border-slate-700 focus:border-primary/50 focus:ring-2 focus:ring-primary/20 rounded-lg text-sm w-64 text-slate-700 dark:text-slate-200 placeholder-slate-400 dark:placeholder-slate-500 transition-all outline-none" placeholder="Search resources..." type="text"/>
          </div>
          <button className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg text-sm font-medium transition-all shadow-lg shadow-primary/20">
            <span className="material-icons-round text-sm">add</span>
            Connect Cluster
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-y-auto p-6 space-y-4">
        <div className="bg-white dark:bg-surface-dark rounded-xl border border-primary/30 shadow-lg shadow-black/20 overflow-hidden">
          <div className="p-4 flex items-center justify-between cursor-pointer bg-slate-50 dark:bg-surface-darker/50 border-b border-slate-200 dark:border-slate-700">
            <div className="flex items-center gap-4">
              <span className="material-icons-round text-slate-400">expand_more</span>
              <div className="w-10 h-10 rounded bg-blue-500/10 flex items-center justify-center">
                <span className="material-icons-round text-blue-500">hub</span>
              </div>
              <div>
                <h3 className="font-semibold text-slate-800 dark:text-white">production-us-east-1</h3>
                <div className="flex items-center gap-2 text-xs text-slate-500">
                  <span className="font-mono">v1.27.4</span>
                  <span>•</span>
                  <span>12 Nodes</span>
                </div>
              </div>
            </div>
            <div className="flex items-center gap-6">
              <div className="text-right">
                <div className="flex items-center justify-end gap-2">
                  <span className="relative flex h-2.5 w-2.5">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
                  </span>
                  <span className="text-sm font-medium text-emerald-500">Healthy</span>
                </div>
              </div>
            </div>
          </div>

          <div className="border-t border-slate-200 dark:border-slate-700">
            <div className="flex items-center px-4 border-b border-slate-200 dark:border-slate-700 bg-slate-50/50 dark:bg-surface-darker/30 gap-6">
              <button className="py-3 px-1 text-sm font-medium border-b-2 border-primary text-primary">Workloads</button>
              <button className="py-3 px-1 text-sm font-medium border-b-2 border-transparent text-slate-500">Nodes</button>
              <button className="py-3 px-1 text-sm font-medium border-b-2 border-transparent text-slate-500">Services</button>
            </div>
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="border-b border-slate-200 dark:border-slate-700/50 text-xs uppercase tracking-wider text-slate-500 font-medium">
                    <th className="px-6 py-3">Name</th>
                    <th className="px-6 py-3">Status</th>
                    <th className="px-6 py-3">Restarts</th>
                    <th className="px-6 py-3 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-200 dark:divide-slate-700/50 text-sm">
                  <tr className="group hover:bg-slate-50 dark:hover:bg-slate-800/40 transition-colors">
                    <td className="px-6 py-3 font-medium text-slate-700 dark:text-slate-200 flex items-center gap-3">
                      <span className="material-icons-round text-blue-400 text-lg">layers</span>
                      frontend-app-deployment-7d8
                    </td>
                    <td className="px-6 py-3">
                      <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-emerald-500/10 text-emerald-500 border border-emerald-500/20">Running</span>
                    </td>
                    <td className="px-6 py-3 font-mono text-slate-500">0</td>
                    <td className="px-6 py-3 text-right">
                      <span className="material-icons-round text-slate-400 cursor-pointer hover:text-primary">terminal</span>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      {/* Terminal Drawer Placeholder */}
      <div className="absolute bottom-0 left-0 right-0 z-30">
        <div className="bg-surface-darker border-t border-slate-700 text-white flex flex-col h-10 overflow-hidden">
          <div className="h-10 bg-surface-dark flex items-center px-4 justify-between cursor-ns-resize border-b border-slate-700">
            <div className="flex items-center gap-3">
              <span className="material-icons-round text-sm text-primary">terminal</span>
              <span className="text-xs font-mono text-slate-300">Terminal Drawer</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export function SplitPaneLayout({ tabs }: SplitPaneLayoutProps) {
  const { activeTabId, servers, addSession, addTab } = useAppStore();
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [connectingServer, setConnectingServer] = useState<ServerType | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  const loadServers = async () => {};

  const handleConnectClick = (server: ServerType) => {
    setConnectingServer(server);
    setConnectError(null);
  };

  const handleConnect = async (password?: string, privateKey?: string, passphrase?: string) => {
    if (!connectingServer) return;

    setIsConnecting(true);
    setConnectError(null);

    try {
      const session = await connectSsh({
        server_id: connectingServer.id,
        password,
        private_key: privateKey,
        passphrase,
      });

      addSession({
        sessionId: session.session_id,
        serverId: connectingServer.id,
        connected: true,
      });

      addTab({
        id: `terminal-${session.session_id}`,
        type: 'terminal',
        title: connectingServer.name,
        serverId: connectingServer.id,
        sessionId: session.session_id,
      });

      setConnectingServer(null);
    } catch (error: any) {
      setConnectError(error.message || String(error));
    } finally {
      setIsConnecting(false);
    }
  };

  if (tabs.length === 0) {
    return (
      <main className="flex-1 flex flex-col h-full overflow-hidden bg-background-dark relative">
        {/* Top Header */}
        <header className="h-16 px-6 flex items-center justify-between border-b border-white/5 bg-background-dark/80 backdrop-blur-md sticky top-0 z-20">
          {/* Search */}
          <div className="flex-1 max-w-2xl relative group">
            <span className="material-icons-round absolute left-3 top-1/2 -translate-y-1/2 text-slate-500 group-focus-within:text-primary transition-colors">search</span>
            <input
              className="w-full bg-background-card border border-white/10 text-sm rounded-lg pl-10 pr-4 py-2 text-slate-300 placeholder-slate-500 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-all"
              placeholder="Search hosts, IPs, or tags... (Cmd+K)"
              type="text"
            />
          </div>
          {/* Actions */}
          <div className="flex items-center gap-3 ml-6">
            <button className="p-2 text-slate-400 hover:text-white hover:bg-white/5 rounded-lg transition-colors relative">
              <span className="material-icons-round">notifications</span>
              <span className="absolute top-2 right-2 w-2 h-2 bg-red-500 rounded-full border-2 border-background-dark"></span>
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
          {/* Filter Bar */}
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
                onClick={() => handleConnectClick(server)}
                className="group bg-background-card rounded-xl p-4 subtle-border hover:border-primary/50 transition-all duration-300 relative card-hover cursor-pointer"
              >
                <div className="flex justify-between items-start mb-3">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center border border-primary/20">
                      <span className="material-icons-round text-primary">terminal</span>
                    </div>
                    <div>
                      <h3 className="font-semibold text-white group-hover:text-primary transition-colors">{server.name}</h3>
                      <p className="text-xs text-slate-500 flex items-center gap-1">
                        <span className="w-1.5 h-1.5 rounded-full bg-slate-500"></span>
                        Offline
                      </p>
                    </div>
                  </div>
                  <button className="text-slate-500 hover:text-white p-1 rounded hover:bg-white/10 opacity-0 group-hover:opacity-100 transition-all">
                    <span className="material-icons-round text-xl">more_vert</span>
                  </button>
                </div>
                <div className="bg-black/30 rounded px-3 py-2 mb-3 font-mono text-xs text-slate-400 flex items-center justify-between">
                  <span>{server.host}</span>
                  <span className="text-slate-600">{server.port}</span>
                </div>
                <div className="flex items-center gap-2 mt-2">
                  <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">PROD</span>
                  <span className="px-2 py-0.5 rounded text-[10px] font-medium bg-slate-700/30 text-slate-400 border border-slate-700/50">Ubuntu</span>
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
              <span className="text-xs text-slate-600 mt-1">SSH, RDP, or Telnet</span>
            </button>
          </div>

          {/* Bottom Section */}
          <div className="mt-8 mb-6 rounded-lg bg-gradient-to-r from-primary/20 to-transparent p-6 border border-primary/20 flex items-start sm:items-center justify-between gap-4">
            <div className="flex items-start gap-4">
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
        <footer className="h-8 bg-[#0b0e14] border-t border-white/5 flex items-center px-4 justify-between text-[11px] text-slate-500 select-none">
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1.5 hover:text-slate-300 cursor-pointer transition-colors">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
              System Operational
            </span>
            <span className="hidden sm:inline">|</span>
            <span className="hidden sm:inline hover:text-slate-300 cursor-pointer transition-colors">0 Active Tunnels</span>
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
    <div className="flex-1 overflow-hidden">
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

export default SplitPaneLayout;
