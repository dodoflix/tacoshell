import { useEffect, useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { fetchServers, connectSsh } from '../hooks/useTauri';
import type { Server as ServerType } from '../types';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';

export function Sidebar() {
  const {
    sidebarOpen,
    toggleSidebar,
    addTab,
    addSession,
    setActiveTab,
    setServers,
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

  const openView = (id: string, type: 'settings' | 'terminal' | 'sftp' | 'k8s', title: string) => {
    const existingTab = useAppStore.getState().tabs.find(t => t.id === id);
    if (existingTab) {
      setActiveTab(id);
    } else {
      addTab({ id, type, title });
    }
  };

  const navItems = [
    { id: 'hosts', icon: 'grid_view', label: 'Hosts', action: () => setActiveTab(null as any) }, // Setting activeTab to null shows Dashboard
    { id: 'secrets', icon: 'key', label: 'Keychain', action: () => openView('secrets', 'settings', 'Keychain') },
    { id: 'snippets', icon: 'code', label: 'Snippets', action: () => {} },
  ];

  return (
    <aside
      className={`${sidebarOpen ? 'w-64' : 'w-16'} bg-background-sidebar border-r border-white/5 flex flex-col justify-between shrink-0 transition-all duration-300 z-30`}
    >
      <div>
        {/* Header / Logo */}
        <div className={`h-16 flex items-center border-b border-white/5 ${sidebarOpen ? 'px-6' : 'justify-center'}`}>
          <div className="flex items-center gap-3">
            <div
              onClick={toggleSidebar}
              className="w-8 h-8 rounded bg-gradient-to-br from-primary to-blue-600 flex items-center justify-center text-white font-bold text-lg shadow-lg shadow-primary/20 cursor-pointer hover:scale-105 transition-transform"
            >
              <span className="material-icons-round text-xl">dns</span>
            </div>
            {sidebarOpen && <span className="font-bold text-lg tracking-tight text-white">Tacoshell</span>}
          </div>
        </div>

        {/* Navigation */}
        <nav className={`mt-6 space-y-1 ${sidebarOpen ? 'px-3' : 'px-2'}`}>
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={item.action}
              className={`flex items-center rounded-lg font-medium group transition-colors w-full ${
                sidebarOpen ? 'gap-3 px-3 py-2.5' : 'justify-center py-3'
              } text-text-secondary hover:bg-white/5 hover:text-white relative`}
            >
              <span className="material-icons-round text-xl">{item.icon}</span>
              {sidebarOpen ? (
                <span>{item.label}</span>
              ) : (
                <span className="absolute left-14 bg-gray-900 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50 shadow-xl border border-white/10">
                  {item.label}
                </span>
              )}
            </button>
          ))}

          <div className={`my-4 border-t border-white/5 mx-2`} />

          {sidebarOpen && (
            <div className="px-3 mb-2 flex items-center justify-between">
              <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Actions</h3>
            </div>
          )}

          <button
            onClick={() => setShowAddDialog(true)}
            className={`flex items-center rounded-lg font-medium group transition-colors w-full ${
              sidebarOpen ? 'gap-3 px-3 py-2.5' : 'justify-center py-3 text-primary bg-primary/10'
            } text-text-secondary hover:bg-white/5 hover:text-white relative`}
          >
            <span className="material-icons-round text-xl">add</span>
            {sidebarOpen ? (
              <span>New Connection</span>
            ) : (
              <span className="absolute left-14 bg-gray-900 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50 shadow-xl border border-white/10">
                New Connection
              </span>
            )}
          </button>
        </nav>
      </div>

      {/* Bottom Actions */}
      <div className={`p-4 border-t border-white/5 ${sidebarOpen ? '' : 'flex justify-center'}`}>
        <button
          onClick={() => openView('settings', 'settings', 'Settings')}
          className={`flex items-center rounded-lg text-text-secondary hover:bg-white/5 hover:text-white transition-colors text-sm ${
            sidebarOpen ? 'gap-3 px-3 py-2 w-full' : 'p-2'
          }`}
        >
          <span className="material-icons-round text-xl">settings</span>
          {sidebarOpen && (
            <div className="flex flex-col items-start overflow-hidden">
              <span className="font-medium truncate w-full text-left">Settings</span>
            </div>
          )}
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
