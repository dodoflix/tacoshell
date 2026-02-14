// Sidebar component for server list

import { useEffect, useState } from 'react';
import {
  Plus,
  Settings,
  ChevronRight,
  Trash2,
  Play,
  LayoutGrid,
  Shield,
  FileText,
  UserCheck,
  Server as ServerIcon,
  ChevronDown,
  Terminal,
  FolderOpen
} from 'lucide-react';
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
    activeTabId,
    setActiveTab,
  } = useAppStore();

  const [navigation, setNavigation] = useState<'hosts' | 'secrets' | 'logs' | 'known_hosts' | 'settings'>('hosts');
  const [expandedServers, setExpandedServers] = useState<Set<string>>(new Set());
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

  // Open the connect dialog
  const handleConnectClick = (server: ServerType) => {
    setConnectingServer(server);
    setConnectError(null);
  };

  // Actually perform the connection
  const handleConnect = async (password?: string, privateKey?: string, passphrase?: string) => {
    if (!connectingServer) return;

    // Check if we already have an active terminal tab for this server
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
    const existingTab = useAppStore.getState().tabs.find(t => t.id === 'settings');
    if (existingTab) {
      setActiveTab('settings');
      return;
    }
    setNavigation('settings');
    addTab({
      id: 'settings',
      type: 'settings',
      title: 'Settings',
    });
  };

  const toggleServerExpand = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const newExpanded = new Set(expandedServers);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedServers(newExpanded);
  };

  const openServerDetail = (server: ServerType) => {
    const existingTab = useAppStore.getState().tabs.find(
      t => t.id === `server-${server.id}`
    );
    if (existingTab) {
      setActiveTab(existingTab.id);
      return;
    }

    addTab({
      id: `server-${server.id}`,
      type: 'settings', // Reusing settings type for now, or create a new one
      title: `Server: ${server.name}`,
      serverId: server.id,
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
      <div className="sidebar-nav">
        <button
          className={`nav-item ${navigation === 'hosts' ? 'active' : ''}`}
          onClick={() => setNavigation('hosts')}
        >
          <LayoutGrid size={18} />
          <span>Hosts</span>
        </button>
        <button
          className={`nav-item ${navigation === 'secrets' ? 'active' : ''}`}
          onClick={() => {
            const existingTab = useAppStore.getState().tabs.find(t => t.id === 'secrets');
            if (existingTab) {
              setActiveTab('secrets');
            } else {
              addTab({ id: 'secrets', type: 'settings', title: 'Secrets' });
            }
            setNavigation('secrets');
          }}
        >
          <Shield size={18} />
          <span>Keychain</span>
        </button>
        <button
          className={`nav-item ${navigation === 'logs' ? 'active' : ''}`}
          onClick={() => setNavigation('logs')}
        >
          <FileText size={18} />
          <span>Logs</span>
        </button>
        <button
          className={`nav-item ${navigation === 'known_hosts' ? 'active' : ''}`}
          onClick={() => setNavigation('known_hosts')}
        >
          <UserCheck size={18} />
          <span>Known Hosts</span>
        </button>
        <button
          className={`nav-item ${navigation === 'settings' ? 'active' : ''}`}
          onClick={openSettings}
        >
          <Settings size={18} />
          <span>Settings</span>
        </button>
        <div className="sidebar-divider" />
      </div>

      <div className="sidebar-content">
        {navigation === 'hosts' && (
          <>
            <div className="sidebar-header">
              <h2>Servers</h2>
              <button onClick={() => setShowAddDialog(true)} title="Add Server">
                <Plus size={18} />
              </button>
            </div>

            <div className="server-list">
              {servers.length === 0 ? (
                <div className="empty-state">
                  <ServerIcon size={32} />
                  <p>No servers yet</p>
                  <button onClick={() => setShowAddDialog(true)}>Add Server</button>
                </div>
              ) : (
                servers.map((server) => (
                  <div key={server.id} className="server-group">
                    <div
                      className={`server-item ${activeTabId === `server-${server.id}` ? 'active' : ''}`}
                      onClick={() => openServerDetail(server)}
                    >
                      <button
                        className={`expand-toggle ${expandedServers.has(server.id) ? 'expanded' : ''}`}
                        onClick={(e) => toggleServerExpand(server.id, e)}
                      >
                        <ChevronDown size={14} />
                      </button>
                      <div className="server-info">
                        <span className="server-name">{server.name}</span>
                      </div>
                      <div className="server-actions">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleConnectClick(server);
                          }}
                          disabled={isConnecting && connectingServer?.id === server.id}
                          title="Connect"
                        >
                          <Play size={14} />
                        </button>
                      </div>
                    </div>
                    {expandedServers.has(server.id) && (
                      <div className="server-sub-items">
                        <div className="sub-item" onClick={() => handleConnectClick(server)}>
                          <Terminal size={12} /> Terminal
                        </div>
                        <div className="sub-item" onClick={() => {
                          const tabId = `sftp-${server.id}`;
                          const existingTab = useAppStore.getState().tabs.find(t => t.id === tabId);
                          if (existingTab) {
                            setActiveTab(tabId);
                          } else {
                            addTab({
                              id: tabId,
                              type: 'sftp',
                              title: `SFTP: ${server.name}`,
                              serverId: server.id
                            });
                          }
                        }}>
                          <FolderOpen size={12} /> SFTP
                        </div>
                        <div className="sub-item danger" onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(server.id);
                        }}>
                          <Trash2 size={12} /> Delete
                        </div>
                      </div>
                    )}
                  </div>
                ))
              )}
            </div>
          </>
        )}

        {navigation === 'secrets' && (
          <div className="sidebar-placeholder">
            <h3>Secrets</h3>
            <p>Use the Secrets tab to manage your credentials.</p>
          </div>
        )}

        {navigation === 'logs' && (
          <div className="sidebar-placeholder">
            <h3>Logs</h3>
            <p>Session logs will appear here.</p>
          </div>
        )}

        {navigation === 'known_hosts' && (
          <div className="sidebar-placeholder">
            <h3>Known Hosts</h3>
            <p>Manage SSH host keys.</p>
          </div>
        )}
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
