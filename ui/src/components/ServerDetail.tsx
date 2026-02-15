// Server detail view refactored with Tailwind

import { useState, useEffect } from 'react';
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
  const { servers, setServers, addTab, addSession } = useAppStore();
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
    return <div className="flex items-center justify-center h-full text-slate-500">Server not found</div>;
  }

  return (
    <div className="p-8 max-w-4xl mx-auto h-full overflow-y-auto font-display">
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center text-primary border border-primary/20">
            <span className="material-icons-round text-2xl">dns</span>
          </div>
          <div>
            <h2 className="text-2xl font-bold text-white">{server.name}</h2>
            <p className="text-sm text-slate-500 font-mono">{server.username}@{server.host}:{server.port}</p>
          </div>
        </div>
        <div className="flex gap-3">
          <button
            onClick={handleConnect}
            className="bg-primary hover:bg-primary-hover text-white px-5 py-2 rounded-lg text-sm font-semibold flex items-center gap-2 shadow-lg shadow-primary/20 transition-all active:scale-95"
          >
            <span className="material-icons-round text-lg">terminal</span>
            Connect
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-6">
        <section className="bg-background-card p-6 rounded-2xl border border-white/5 space-y-6">
          <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">General Settings</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-1.5">
              <label className="text-sm text-slate-400 pl-1">Display Name</label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-sm text-slate-400 pl-1">Host / IP</label>
              <input
                type="text"
                value={host}
                onChange={(e) => setHost(e.target.value)}
                className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all font-mono"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-sm text-slate-400 pl-1">Port</label>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(parseInt(e.target.value))}
                className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-sm text-slate-400 pl-1">Username</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
              />
            </div>
          </div>
        </section>

        <section className="bg-background-card p-6 rounded-2xl border border-white/5 space-y-4">
          <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">Authentication</h3>
          <div className="space-y-1.5">
            <label className="text-sm text-slate-400 pl-1">Linked Secret</label>
            <select
              value={linkedSecretId || ''}
              onChange={(e) => setLinkedSecretId(e.target.value || null)}
              className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all appearance-none"
            >
              <option value="">None (Use Agent / Temporary)</option>
              {allSecrets.map(s => (
                <option key={s.id} value={s.id}>{s.name} ({s.kind})</option>
              ))}
            </select>
            <p className="text-xs text-slate-500 pl-1 pt-1">The selected secret will be used for automatic login.</p>
          </div>
        </section>

        {message && (
          <div className={`p-4 rounded-xl flex items-center gap-3 border ${
            message.type === 'success'
              ? 'bg-emerald-500/10 border-emerald-500/20 text-emerald-400'
              : 'bg-red-500/10 border-red-500/20 text-red-400'
          }`}>
            <span className="material-icons-round text-lg">
              {message.type === 'success' ? 'check_circle' : 'error'}
            </span>
            <span className="text-sm font-medium">{message.text}</span>
          </div>
        )}

        <div className="pt-4 flex justify-end">
          <button
            onClick={handleSave}
            disabled={loading}
            className="bg-white/5 hover:bg-white/10 text-white px-8 py-3 rounded-xl text-sm font-bold transition-all border border-white/10 flex items-center gap-2"
          >
            {loading ? (
              <span className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></span>
            ) : (
              <span className="material-icons-round text-lg">save</span>
            )}
            Save Configuration
          </button>
        </div>
      </div>
    </div>
  );
}
