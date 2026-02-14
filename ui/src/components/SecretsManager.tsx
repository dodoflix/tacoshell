// Secrets management component

import { useState, useEffect } from 'react';
import { Key, Plus, Trash2, Link, Eye, EyeOff } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import {
  fetchSecrets,
  createSecret,
  deleteSecret,
  linkSecretToServer
} from '../hooks/useTauri';
import type { Secret } from '../types';

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
    <div className="secrets-manager">
      <div className="secrets-header">
        <h3><Key size={18} /> Secrets</h3>
        <button onClick={() => setShowAddForm(true)} className="btn-icon" title="Add Secret">
          <Plus size={18} />
        </button>
      </div>

      <div className="secrets-list">
        {secrets.length === 0 ? (
          <div className="empty-state small">
            <Key size={24} />
            <p>No secrets stored</p>
            <button onClick={() => setShowAddForm(true)} className="btn-small">
              Add Secret
            </button>
          </div>
        ) : (
          secrets.map((secret) => (
            <SecretItem
              key={secret.id}
              secret={secret}
              onDelete={() => handleDelete(secret.id)}
              onLink={serverId ? () => handleLink(secret.id) : undefined}
            />
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

interface SecretItemProps {
  secret: Secret;
  onDelete: () => void;
  onLink?: () => void;
}

function SecretItem({ secret, onDelete, onLink }: SecretItemProps) {
  const kindLabels: Record<string, string> = {
    password: '🔑 Password',
    private_key: '🔐 SSH Key',
    token: '🎫 Token',
    kubeconfig: '☸️ Kubeconfig',
  };

  return (
    <div className="secret-item">
      <div className="secret-info">
        <span className="secret-name">{secret.name}</span>
        <span className="secret-kind">{kindLabels[secret.kind] || secret.kind}</span>
      </div>
      <div className="secret-actions">
        {onLink && (
          <button onClick={onLink} title="Link to server" className="btn-icon">
            <Link size={14} />
          </button>
        )}
        <button onClick={onDelete} title="Delete" className="btn-icon danger">
          <Trash2 size={14} />
        </button>
      </div>
    </div>
  );
}

interface AddSecretFormProps {
  onClose: () => void;
  onAdded: () => void;
}

function AddSecretForm({ onClose, onAdded }: AddSecretFormProps) {
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
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h3>Add Secret</h3>
          <button onClick={onClose} className="btn-icon">×</button>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="secret-name">Name</label>
            <input
              id="secret-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My SSH Key"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="secret-kind">Type</label>
            <select
              id="secret-kind"
              value={kind}
              onChange={(e) => setKind(e.target.value)}
            >
              <option value="password">Password</option>
              <option value="private_key">SSH Private Key</option>
              <option value="token">Token</option>
              <option value="kubeconfig">Kubeconfig</option>
            </select>
          </div>

          {kind === 'password' && (
            <div className="form-group">
              <label htmlFor="secret-username">Username</label>
              <input
                id="secret-username"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="root"
              />
            </div>
          )}

          <div className="form-group">
            <label htmlFor="secret-value">
              {kind === 'private_key' ? 'Private Key' : 'Value'}
            </label>
            <div className="input-with-toggle">
              {kind === 'private_key' ? (
                <textarea
                  id="secret-value"
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                  placeholder="-----BEGIN OPENSSH PRIVATE KEY-----"
                  rows={6}
                  required
                />
              ) : (
                <input
                  id="secret-value"
                  type={showValue ? 'text' : 'password'}
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                  required
                />
              )}
              {kind !== 'private_key' && (
                <button
                  type="button"
                  onClick={() => setShowValue(!showValue)}
                  className="btn-icon"
                >
                  {showValue ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              )}
            </div>
          </div>

          {error && <div className="form-error">{error}</div>}

          <div className="dialog-footer">
            <button type="button" onClick={onClose} className="btn-secondary">
              Cancel
            </button>
            <button type="submit" disabled={loading} className="btn-primary">
              {loading ? 'Adding...' : 'Add Secret'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default SecretsManager;
