import { useEffect, useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { fetchServers, connectSsh } from '../hooks/useTauri';
import type { Server as ServerType } from '../types';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';

export function Sidebar() {
  const {
    setServers,
    sidebarOpen,
    addTab,
    addSession,
    setActiveTab,
  } = useAppStore();

  const [showAddDialog, setShowAddDialog] = useState(false);
  const [connectingServer, setConnectingServer] = useState<ServerType | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    try {
      const data = await fetchServers();
      setServers(data);
    } catch (error) {
      console.error('Failed to load servers:', error);
    }
  };

  const handleConnect = async (password?: string, privateKey?: string, passphrase?: string) => {
    if (!connectingServer) return;

    const existingTab = useAppStore.getState().tabs.find(
      t => t.type === 'terminal' && t.serverId === connectingServer.id
    );

    if (existingTab) {
      setActiveTab(existingTab.id);
      setConnectingServer(null);
      return;
    }

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
      console.error('Failed to connect:', error);
      setConnectError(error.message || String(error));
    } finally {
      setIsConnecting(false);
    }
  };

  if (!sidebarOpen) return null;

  const groups = [
    { name: 'Production', color: 'bg-emerald-500', count: 12, shadow: 'shadow-[0_0_8px_rgba(16,185,129,0.4)]' },
    { name: 'Staging', color: 'bg-amber-500', count: 4, shadow: 'shadow-[0_0_8px_rgba(245,158,11,0.4)]' },
    { name: 'AWS East', color: 'bg-purple-500', count: 8, shadow: 'shadow-[0_0_8px_rgba(168,85,247,0.4)]' },
  ];

  return (
    <aside className="w-64 bg-background-sidebar border-r border-white/5 flex flex-col justify-between shrink-0 transition-all duration-300">
      <div>
        {/* Title Area */}
        <div className="h-16 flex items-center px-6 border-b border-white/5">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded bg-gradient-to-br from-primary to-blue-600 flex items-center justify-center text-white font-bold text-lg shadow-lg shadow-primary/20">
              <span className="material-icons-round text-xl">dns</span>
            </div>
            <span className="font-bold text-lg tracking-tight text-white">Tacoshell</span>
          </div>
        </div>

        {/* Navigation */}
        <nav className="mt-6 px-3 space-y-1">
          <a className="flex items-center gap-3 px-3 py-2.5 rounded-lg bg-primary/10 text-primary font-medium group transition-colors" href="#">
            <span className="material-icons-round text-xl">grid_view</span>
            Hosts
          </a>
          <a className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-text-secondary hover:bg-white/5 hover:text-white font-medium group transition-colors" href="#">
            <span className="material-icons-round text-xl">hub</span>
            Clusters
          </a>
          <a className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-text-secondary hover:bg-white/5 hover:text-white font-medium group transition-colors" href="#">
            <span className="material-icons-round text-xl">folder_open</span>
            SFTP / FTP
          </a>
          <a className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-text-secondary hover:bg-white/5 hover:text-white font-medium group transition-colors" href="#">
            <span className="material-icons-round text-xl">key</span>
            Keychain
          </a>
          <a className="flex items-center gap-3 px-3 py-2.5 rounded-lg text-text-secondary hover:bg-white/5 hover:text-white font-medium group transition-colors" href="#">
            <span className="material-icons-round text-xl">code</span>
            Snippets
          </a>
        </nav>

        {/* Group Labels */}
        <div className="mt-8 px-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Groups</h3>
            <button onClick={() => setShowAddDialog(true)} className="text-slate-500 hover:text-white">
                 <span className="material-icons-round text-sm">add</span>
            </button>
          </div>
          <div className="space-y-3">
            {groups.map((group) => (
              <button key={group.name} className="flex items-center w-full group">
                <span className={`w-2 h-2 rounded-full ${group.color} mr-3 ${group.shadow}`}></span>
                <span className="text-sm text-text-secondary group-hover:text-white">{group.name}</span>
                <span className="ml-auto text-xs text-slate-600 bg-white/5 px-1.5 py-0.5 rounded">{group.count}</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Bottom Actions */}
      <div className="p-4 border-t border-white/5">
        <button className="flex items-center gap-3 px-3 py-2 w-full rounded-lg text-text-secondary hover:bg-white/5 hover:text-white transition-colors text-sm text-left">
          <img
            alt="User Avatar"
            className="w-8 h-8 rounded-full"
            src="https://lh3.googleusercontent.com/aida-public/AB6AXuA_0dmYfq_iOoIXuNR1OhcdvqSQoWcJpcen7bXgZilPu88tw-pFsAc72TeecFdU0FtN9hxOC2m28wPWYpq4VehJSUlG8Q6F93P-eShrUMpzJRNBvzTMbrZyDwyidcG4KHjWgi1Ji0337RcosTiI8e0GrZomj5XBwc7WGXKxqG2fWUOPqncLhvYHRmglqLWuJuP3l6nWLlcNWvMiSuLaM0_HCZZIQYARLfnso-pu8cdHcIW0PMyh1cAHmkdwZPzcZ8y9z7PwXeW8L3BV"
          />
          <div className="flex flex-col items-start overflow-hidden">
            <span className="font-medium truncate w-full">Alex Doe</span>
            <span className="text-xs text-slate-500">Pro Plan</span>
          </div>
          <span className="material-icons-round ml-auto text-lg">settings</span>
        </button>
      </div>

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
    </aside>
  );
}
