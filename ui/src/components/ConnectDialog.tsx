// Connection dialog refactored with Tailwind

import { useState } from 'react';
import type { Server } from '../types';

interface ConnectDialogProps {
  server: Server;
  onConnect: (password?: string, privateKey?: string, passphrase?: string) => void;
  onCancel: () => void;
  loading?: boolean;
  error?: string | null;
}

export function ConnectDialog({ server, onConnect, onCancel, loading, error }: ConnectDialogProps) {
  const [authMethod, setAuthMethod] = useState<'password' | 'key' | 'agent' | 'stored'>('stored');
  const [password, setPassword] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showPassphrase, setShowPassphrase] = useState(false);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (authMethod === 'password') {
      onConnect(password, undefined, undefined);
    } else if (authMethod === 'key') {
      onConnect(undefined, privateKey, passphrase || undefined);
    } else if (authMethod === 'agent') {
      onConnect(undefined, undefined, undefined);
    } else {
      onConnect();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000] p-4" onClick={onCancel}>
      <div className="bg-background-card border border-white/10 rounded-2xl p-6 w-full max-w-md shadow-2xl shadow-black/50" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-2">
          <h2 className="text-xl font-bold text-white">Connect to Host</h2>
          <button onClick={onCancel} className="p-1 hover:bg-white/5 rounded-lg text-slate-500 hover:text-white transition-colors">
            <span className="material-icons-round">close</span>
          </button>
        </div>

        <div className="bg-primary/5 rounded-lg p-3 mb-6 flex items-center gap-3 border border-primary/10">
          <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
             <span className="material-icons-round">dns</span>
          </div>
          <div className="overflow-hidden">
            <h3 className="font-semibold text-white truncate">{server.name}</h3>
            <p className="text-xs text-slate-500 font-mono truncate">{server.username}@{server.host}:{server.port}</p>
          </div>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">Authentication Method</label>
            <div className="grid grid-cols-2 gap-2">
              {[
                { id: 'stored', icon: 'lock', label: 'Saved' },
                { id: 'password', icon: 'password', label: 'Password' },
                { id: 'key', icon: 'vpn_key', label: 'SSH Key' },
                { id: 'agent', icon: 'admin_panel_settings', label: 'Agent' },
              ].map((method) => (
                <button
                  key={method.id}
                  type="button"
                  onClick={() => setAuthMethod(method.id as any)}
                  className={`flex flex-col items-center gap-2 p-3 rounded-xl border transition-all ${
                    authMethod === method.id
                      ? 'bg-primary/10 border-primary text-primary shadow-sm shadow-primary/10'
                      : 'bg-background-dark/30 border-white/5 text-slate-500 hover:text-slate-300 hover:bg-white/5'
                  }`}
                >
                  <span className="material-icons-round text-xl">{method.icon}</span>
                  <span className="text-[11px] font-medium uppercase tracking-wider">{method.label}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="bg-background-dark/30 rounded-xl p-4 border border-white/5 min-h-[140px] flex flex-col justify-center">
            {authMethod === 'stored' && (
              <div className="text-center space-y-2">
                <span className="material-icons-round text-primary/40 text-3xl">verified_user</span>
                <p className="text-sm text-slate-300">Using linked credentials</p>
                <p className="text-xs text-slate-500">Falls back to agent if no secrets are found.</p>
              </div>
            )}

            {authMethod === 'password' && (
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-slate-400">Password</label>
                <div className="relative">
                  <input
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="w-full bg-background-dark border border-white/10 rounded-lg pl-4 pr-10 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
                    placeholder="Enter password"
                    autoFocus
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                  >
                    <span className="material-icons-round text-lg">{showPassword ? 'visibility_off' : 'visibility'}</span>
                  </button>
                </div>
              </div>
            )}

            {authMethod === 'key' && (
              <div className="space-y-3">
                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-slate-400">Private Key</label>
                  <textarea
                    value={privateKey}
                    onChange={(e) => setPrivateKey(e.target.value)}
                    className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2 text-slate-200 font-mono text-xs focus:outline-none focus:border-primary transition-all h-24"
                    placeholder="Paste private key content..."
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs font-medium text-slate-400">Passphrase (optional)</label>
                  <div className="relative">
                    <input
                      type={showPassphrase ? 'text' : 'password'}
                      value={passphrase}
                      onChange={(e) => setPassphrase(e.target.value)}
                      className="w-full bg-background-dark border border-white/10 rounded-lg pl-4 pr-10 py-2 text-slate-200 focus:outline-none focus:border-primary transition-all"
                      placeholder="Key passphrase"
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassphrase(!showPassphrase)}
                      className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                    >
                      <span className="material-icons-round text-lg">{showPassphrase ? 'visibility_off' : 'visibility'}</span>
                    </button>
                  </div>
                </div>
              </div>
            )}

            {authMethod === 'agent' && (
              <div className="text-center space-y-2">
                <span className="material-icons-round text-primary/40 text-3xl">terminal</span>
                <p className="text-sm text-slate-300">Using SSH Agent</p>
                <p className="text-xs text-slate-500">Ensure your agent has keys loaded.</p>
              </div>
            )}
          </div>

          {error && (
            <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-red-400 text-xs flex items-start gap-2">
              <span className="material-icons-round text-sm">error</span>
              <div className="overflow-hidden truncate">{error}</div>
            </div>
          )}

          <div className="flex justify-end gap-3 pt-4">
            <button
              type="button"
              onClick={onCancel}
              className="px-5 py-2.5 rounded-lg text-sm font-medium text-slate-400 hover:text-white hover:bg-white/5 transition-all"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="bg-primary hover:bg-primary-hover disabled:opacity-50 text-white px-8 py-2.5 rounded-lg text-sm font-bold transition-all shadow-lg shadow-primary/20 active:scale-95 flex items-center gap-2"
            >
              {loading ? (
                <>
                  <span className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin"></span>
                  Connecting...
                </>
              ) : 'Connect'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ConnectDialog;
