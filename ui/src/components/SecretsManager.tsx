// Secrets management component refactored with Tailwind

import { useState, useEffect } from 'react';
import { useAppStore } from '../stores/appStore';
import {
  fetchSecrets,
  createSecret,
  deleteSecret,
  linkSecretToServer
} from '../hooks/useTauri';

interface SecretsManagerProps {
  serverId?: string;
}

export function SecretsManager({ serverId }: SecretsManagerProps) {
  const { secrets, setSecrets } = useAppStore();
  const [showAddForm, setShowAddForm] = useState(false);

  useEffect(() => {
    loadSecrets();
  }, []);

  const loadSecrets = async () => {
    try {
      const data = await fetchSecrets();
      setSecrets(data);
    } catch (error) {
      console.error('Failed to load secrets:', error);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this secret?')) return;

    try {
      await deleteSecret(id);
      await loadSecrets();
    } catch (error) {
      console.error('Failed to delete secret:', error);
    }
  };

  const handleLink = async (secretId: string) => {
    if (!serverId) return;

    try {
      await linkSecretToServer(serverId, secretId);
    } catch (error) {
      console.error('Failed to link secret:', error);
    }
  };

  return (
    <div className="p-8 max-w-4xl mx-auto h-full overflow-y-auto font-display">
      <div className="flex items-center justify-between mb-8">
        <h2 className="text-2xl font-bold text-white">Keychain</h2>
        <button
            onClick={() => setShowAddForm(true)}
            className="bg-primary hover:bg-primary-hover text-white px-4 py-2 rounded-lg text-sm font-medium flex items-center gap-2 shadow-lg shadow-primary/20 transition-all active:scale-95"
        >
          <span className="material-icons-round text-lg">add</span>
          Add Secret
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {secrets.length === 0 ? (
          <div className="col-span-full py-12 flex flex-col items-center justify-center bg-background-card rounded-2xl border border-dashed border-white/10 text-slate-500">
            <span className="material-icons-round text-5xl mb-4">key</span>
            <p>No secrets stored in your keychain</p>
          </div>
        ) : (
          secrets.map((secret) => (
            <div key={secret.id} className="bg-background-card p-4 rounded-xl border border-white/5 hover:border-white/10 transition-colors flex items-center justify-between group">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-white/5 flex items-center justify-center text-slate-400">
                   <span className="material-icons-round">
                    {secret.kind === 'password' ? 'password' :
                     secret.kind === 'private_key' ? 'vpn_key' : 'badge'}
                   </span>
                </div>
                <div>
                  <h4 className="font-semibold text-white">{secret.name}</h4>
                  <p className="text-xs text-slate-500 uppercase tracking-wider">{secret.kind.replace('_', ' ')}</p>
                </div>
              </div>
              <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                {serverId && (
                  <button onClick={() => handleLink(secret.id)} className="p-2 hover:bg-white/5 rounded-lg text-primary" title="Link to server">
                    <span className="material-icons-round text-lg">link</span>
                  </button>
                )}
                <button onClick={() => handleDelete(secret.id)} className="p-2 hover:bg-red-500/10 rounded-lg text-slate-500 hover:text-red-400" title="Delete">
                  <span className="material-icons-round text-lg">delete</span>
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {showAddForm && (
        <AddSecretForm
          onClose={() => setShowAddForm(false)}
          onAdded={loadSecrets}
        />
      )}
    </div>
  );
}

function AddSecretForm({ onClose, onAdded }: { onClose: () => void; onAdded: () => void }) {
  const [name, setName] = useState('');
  const [kind, setKind] = useState('password');
  const [value, setValue] = useState('');
  const [username, setUsername] = useState('');
  const [showValue, setShowValue] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      await createSecret({
        name,
        kind,
        value,
        username: kind === 'password' ? username : undefined
      });
      onAdded();
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1100] p-4" onClick={onClose}>
      <div className="bg-background-card border border-white/10 rounded-2xl p-6 w-full max-w-md shadow-2xl shadow-black/50" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-bold text-white">New Secret</h2>
          <button onClick={onClose} className="p-1 hover:bg-white/5 rounded-lg text-slate-500 hover:text-white transition-colors">
            <span className="material-icons-round">close</span>
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
              placeholder="e.g. Production SSH Key"
              required
            />
          </div>

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">Type</label>
            <select
              value={kind}
              onChange={(e) => setKind(e.target.value)}
              className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all appearance-none"
            >
              <option value="password">Password</option>
              <option value="private_key">SSH Private Key</option>
              <option value="token">Token</option>
            </select>
          </div>

          {kind === 'password' && (
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">Username</label>
              <input
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
                placeholder="root"
              />
            </div>
          )}

          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider pl-1">
              {kind === 'private_key' ? 'Private Key' : 'Value'}
            </label>
            <div className="relative">
              {kind === 'private_key' ? (
                <textarea
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                  className="w-full bg-background-dark border border-white/10 rounded-lg px-4 py-2.5 text-slate-200 font-mono text-xs focus:outline-none focus:border-primary transition-all h-32"
                  placeholder="Paste your private key here..."
                  required
                />
              ) : (
                <>
                  <input
                    type={showValue ? 'text' : 'password'}
                    value={value}
                    onChange={(e) => setValue(e.target.value)}
                    className="w-full bg-background-dark border border-white/10 rounded-lg pl-4 pr-10 py-2.5 text-slate-200 focus:outline-none focus:border-primary transition-all"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => setShowValue(!showValue)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300"
                  >
                    <span className="material-icons-round text-lg">{showValue ? 'visibility_off' : 'visibility'}</span>
                  </button>
                </>
              )}
            </div>
          </div>

          {error && (
             <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-3 text-red-400 text-xs">
              {error}
            </div>
          )}

          <div className="flex justify-end gap-3 pt-4">
            <button type="button" onClick={onClose} className="px-5 py-2.5 rounded-lg text-sm font-medium text-slate-400 hover:text-white hover:bg-white/5 transition-all">
              Cancel
            </button>
            <button type="submit" disabled={loading} className="bg-primary hover:bg-primary-hover disabled:opacity-50 text-white px-6 py-2.5 rounded-lg text-sm font-semibold transition-all shadow-lg shadow-primary/20 active:scale-95">
              {loading ? 'Adding...' : 'Save Secret'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default SecretsManager;
