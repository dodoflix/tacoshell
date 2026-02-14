// Sidebar component for server list

import { useEffect, useState } from 'react';
import { Server, Plus, Settings, ChevronLeft, ChevronRight, Trash2, Play } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import { fetchServers, deleteServer, connectSsh } from '../hooks/useTauri';
import type { Server as ServerType } from '../types';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';

export function Sidebar() {
  const {
    servers,
    setServers,
    sidebarOpen,
    toggleSidebar,
    addTab,
    addSession,
  } = useAppStore();

  const [showAddDialog, setShowAddDialog] = useState(false);
  const [connectingServer, setConnectingServer] = useState<ServerType | null>(null);
  const [connectError, setConnectError] = useState<string | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

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

  // Open the connect dialog
  const handleConnectClick = (server: ServerType) => {
    setConnectingServer(server);
    setConnectError(null);
  };

  // Actually perform the connection
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

      // Close dialog on success
      setConnectingServer(null);
    } catch (error: unknown) {
      console.error('Failed to connect:', error);
      const errorMessage = error instanceof Error
        ? error.message
        : typeof error === 'object' && error !== null && 'message' in error
          ? String((error as { message: unknown }).message)
          : String(error);
      setConnectError(errorMessage);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm('Are you sure you want to delete this server?')) {
      try {
        await deleteServer(id);
        await loadServers();
      } catch (error) {
        console.error('Failed to delete server:', error);
      }
    }
  };

  const openSettings = () => {
    addTab({
      id: 'settings',
      type: 'settings',
      title: 'Settings',
    });
  };

  if (!sidebarOpen) {
    return (
      <div className="sidebar-collapsed">
        <button onClick={toggleSidebar} className="sidebar-toggle">
          <ChevronRight size={20} />
        </button>
      </div>
    );
  }

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2>Servers</h2>
        <div className="sidebar-actions">
          <button onClick={() => setShowAddDialog(true)} title="Add Server">
            <Plus size={18} />
          </button>
          <button onClick={toggleSidebar} title="Collapse">
            <ChevronLeft size={18} />
          </button>
        </div>
      </div>

      <div className="server-list">
        {servers.length === 0 ? (
          <div className="empty-state">
            <Server size={32} />
            <p>No servers yet</p>
            <button onClick={() => setShowAddDialog(true)}>Add Server</button>
          </div>
        ) : (
          servers.map((server) => (
            <div key={server.id} className="server-item">
              <div className="server-info">
                <Server size={16} />
                <div>
                  <span className="server-name">{server.name}</span>
                  <span className="server-host">{server.username}@{server.host}:{server.port}</span>
                </div>
              </div>
              <div className="server-actions">
                <button
                  onClick={() => handleConnectClick(server)}
                  disabled={isConnecting && connectingServer?.id === server.id}
                  title="Connect"
                >
                  <Play size={14} />
                </button>
                <button onClick={() => handleDelete(server.id)} title="Delete">
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="sidebar-footer">
        <button onClick={openSettings}>
          <Settings size={18} />
          <span>Settings</span>
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
    </div>
  );
}
