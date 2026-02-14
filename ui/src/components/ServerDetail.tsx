// Server detail view
import { useState, useEffect } from 'react';
import { Server as ServerIcon, Save, Play, FolderOpen } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import {
  updateServer,
  fetchSecrets,
  linkSecretToServer,
  connectSsh
} from '../hooks/useTauri';
import type { Server, Secret } from '../types';

interface ServerDetailProps {
  serverId: string;
}

export function ServerDetail({ serverId }: ServerDetailProps) {
  const { servers, setServers, addTab, addSession, setActiveTab } = useAppStore();
  const server = servers.find((s) => s.id === serverId);

  const [name, setName] = useState(server?.name || '');
  const [host, setHost] = useState(server?.host || '');
  const [port, setPort] = useState(server?.port || 22);
  const [username, setUsername] = useState(server?.username || '');
  const [protocol, setProtocol] = useState(server?.protocol || 'ssh');

  const [allSecrets, setAllSecrets] = useState<Secret[]>([]);
  const [linkedSecretId, setLinkedSecretId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null);

  useEffect(() => {
    if (server) {
      setName(server.name);
      setHost(server.host);
      setPort(server.port);
      setUsername(server.username);
      setProtocol(server.protocol);
    }
    loadData();
  }, [server]);

  const loadData = async () => {
    try {
      const secrets = await fetchSecrets();
      setAllSecrets(secrets);
      // In a real app, we'd fetch the specifically linked secret from backend
      // For now, this is a placeholder for that logic
    } catch (err) {
      console.error(err);
    }
  };

  const handleSave = async () => {
    if (!server) return;
    setLoading(true);
    setMessage(null);

    try {
      const updated: Server = {
        ...server,
        name,
        host,
        port,
        username,
        protocol: protocol as any,
      };

      await updateServer(updated);
      // Update local store
      setServers(servers.map(s => s.id === serverId ? updated : s));
      setMessage({ type: 'success', text: 'Server updated successfully' });

      if (linkedSecretId) {
        await linkSecretToServer(serverId, linkedSecretId);
      }
    } catch (err) {
      setMessage({ type: 'error', text: String(err) });
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = async () => {
    if (!server) return;
    try {
      const session = await connectSsh({ server_id: server.id });
      addSession({
        sessionId: session.session_id,
        serverId: server.id,
        connected: true,
      });
      addTab({
        id: `terminal-${session.session_id}`,
        type: 'terminal',
        title: server.name,
        serverId: server.id,
        sessionId: session.session_id,
      });
    } catch (err) {
      setMessage({ type: 'error', text: `Failed to connect: ${err}` });
    }
  };

  if (!server) {
    return <div className="tile-empty">Server not found</div>;
  }

  return (
    <div className="settings-panel">
      <div className="detail-header">
        <h2><ServerIcon size={24} /> {server.name}</h2>
      </div>

      <div className="tab-actions-bar">
        <button onClick={handleConnect} className="btn-primary">
          <Play size={16} /> Terminal
        </button>
        <button onClick={() => {
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
        }} className="btn-secondary">
          <FolderOpen size={16} /> SFTP
        </button>
      </div>

      <section className="settings-section">
        <h3>General Settings</h3>
        <div className="form-grid">
          <div className="form-group">
            <label>Display Name</label>
            <input type="text" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="form-group">
            <label>Host / IP</label>
            <input type="text" value={host} onChange={(e) => setHost(e.target.value)} />
          </div>
          <div className="form-group">
            <label>Port</label>
            <input type="number" value={port} onChange={(e) => setPort(parseInt(e.target.value))} />
          </div>
          <div className="form-group">
            <label>Username</label>
            <input type="text" value={username} onChange={(e) => setUsername(e.target.value)} />
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3>Authentication</h3>
        <div className="form-group">
          <label>Linked Secret</label>
          <select
            value={linkedSecretId || ''}
            onChange={(e) => setLinkedSecretId(e.target.value || null)}
          >
            <option value="">None (Use Agent / Temporary)</option>
            {allSecrets.map(s => (
              <option key={s.id} value={s.id}>{s.name} ({s.kind})</option>
            ))}
          </select>
          <p className="hint">The selected secret will be used for automatic login.</p>
        </div>
      </section>

      {message && (
        <div className={`message ${message.type}`}>
          {message.text}
        </div>
      )}

      <div className="detail-footer">
        <button
          onClick={handleSave}
          disabled={loading}
          className="btn-primary"
        >
          <Save size={18} /> {loading ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </div>
  );
}
