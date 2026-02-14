// Connection dialog for entering credentials

import { useState } from 'react';
import { Eye, EyeOff, Key, Lock } from 'lucide-react';
import type { Server } from '../types';

interface ConnectDialogProps {
  server: Server;
  onConnect: (password?: string, privateKey?: string, passphrase?: string) => void;
  onCancel: () => void;
  loading?: boolean;
  error?: string | null;
}

export function ConnectDialog({ server, onConnect, onCancel, loading, error }: ConnectDialogProps) {
  const [authMethod, setAuthMethod] = useState<'password' | 'key' | 'agent'>('password');
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
    } else {
      // SSH Agent
      onConnect(undefined, undefined, undefined);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h3>Connect to {server.name}</h3>
          <button onClick={onCancel} className="btn-icon">×</button>
        </div>

        <div className="connect-info">
          <span className="server-address">{server.username}@{server.host}:{server.port}</span>
        </div>

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Authentication Method</label>
            <div className="auth-method-buttons">
              <button
                type="button"
                className={`auth-method-btn ${authMethod === 'password' ? 'active' : ''}`}
                onClick={() => setAuthMethod('password')}
              >
                <Lock size={16} />
                Password
              </button>
              <button
                type="button"
                className={`auth-method-btn ${authMethod === 'key' ? 'active' : ''}`}
                onClick={() => setAuthMethod('key')}
              >
                <Key size={16} />
                SSH Key
              </button>
              <button
                type="button"
                className={`auth-method-btn ${authMethod === 'agent' ? 'active' : ''}`}
                onClick={() => setAuthMethod('agent')}
              >
                🔐
                Agent
              </button>
            </div>
          </div>

          {authMethod === 'password' && (
            <div className="form-group">
              <label htmlFor="password">Password</label>
              <div className="input-with-toggle">
                <input
                  id="password"
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter password"
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="btn-icon"
                >
                  {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
                </button>
              </div>
            </div>
          )}

          {authMethod === 'key' && (
            <>
              <div className="form-group">
                <label htmlFor="privateKey">Private Key</label>
                <textarea
                  id="privateKey"
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
                  placeholder="Paste your private key or enter the file path"
                  rows={4}
                />
              </div>
              <div className="form-group">
                <label htmlFor="passphrase">Passphrase (optional)</label>
                <div className="input-with-toggle">
                  <input
                    id="passphrase"
                    type={showPassphrase ? 'text' : 'password'}
                    value={passphrase}
                    onChange={(e) => setPassphrase(e.target.value)}
                    placeholder="Key passphrase"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassphrase(!showPassphrase)}
                    className="btn-icon"
                  >
                    {showPassphrase ? <EyeOff size={16} /> : <Eye size={16} />}
                  </button>
                </div>
              </div>
            </>
          )}

          {authMethod === 'agent' && (
            <div className="agent-info">
              <p>Will attempt to authenticate using SSH Agent.</p>
              <p className="hint">Make sure ssh-agent is running and has your key loaded.</p>
            </div>
          )}

          {error && <div className="form-error">{error}</div>}

          <div className="dialog-footer">
            <button type="button" onClick={onCancel} className="btn-secondary" disabled={loading}>
              Cancel
            </button>
            <button type="submit" className="btn-primary" disabled={loading}>
              {loading ? 'Connecting...' : 'Connect'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ConnectDialog;

