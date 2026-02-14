// Split pane layout using react-mosaic

import { useState } from 'react';
import 'react-mosaic-component/react-mosaic-component.css';
import { useAppStore } from '../stores/appStore';
import { TerminalView } from './Terminal';
import { SecretsManager } from './SecretsManager';
import type { Tab } from '../types';

interface SplitPaneLayoutProps {
  tabs: Tab[];
}

// Render content for each tile based on tab type
function TileContent({ tabId }: { tabId: string }) {
  const { tabs } = useAppStore();
  const tab = tabs.find((t) => t.id === tabId);

  if (!tab) {
    return <div className="tile-empty">Tab not found</div>;
  }

  switch (tab.type) {
    case 'terminal':
      return tab.sessionId ? (
        <TerminalView sessionId={tab.sessionId} />
      ) : (
        <div className="tile-empty">No session</div>
      );
    case 'settings':
      return <SettingsPanel />;
    case 'sftp':
      return <SftpPlaceholder />;
    case 'k8s':
      return <K8sPlaceholder />;
    default:
      return <div className="tile-empty">Unknown tab type</div>;
  }
}

function SettingsPanel() {
  const [activeSection, setActiveSection] = useState<'general' | 'secrets' | 'about'>('general');

  return (
    <div className="settings-panel">
      <h2>⚙️ Settings</h2>

      <div className="settings-tabs">
        <button
          className={activeSection === 'general' ? 'active' : ''}
          onClick={() => setActiveSection('general')}
        >
          General
        </button>
        <button
          className={activeSection === 'secrets' ? 'active' : ''}
          onClick={() => setActiveSection('secrets')}
        >
          Secrets
        </button>
        <button
          className={activeSection === 'about' ? 'active' : ''}
          onClick={() => setActiveSection('about')}
        >
          About
        </button>
      </div>

      {activeSection === 'general' && (
        <>
          <section className="settings-section">
            <h3>Appearance</h3>
            <div className="setting-item">
              <label>Theme</label>
              <select defaultValue="dark">
                <option value="dark">Dark</option>
                <option value="light">Light (coming soon)</option>
              </select>
            </div>
            <div className="setting-item">
              <label>Font Size</label>
              <input type="number" defaultValue={14} min={10} max={24} />
            </div>
            <div className="setting-item">
              <label>Font Family</label>
              <select defaultValue="consolas">
                <option value="consolas">Consolas</option>
                <option value="jetbrains">JetBrains Mono</option>
                <option value="firacode">Fira Code</option>
              </select>
            </div>
          </section>

          <section className="settings-section">
            <h3>Terminal</h3>
            <div className="setting-item">
              <label>Cursor Style</label>
              <select defaultValue="block">
                <option value="block">Block</option>
                <option value="underline">Underline</option>
                <option value="bar">Bar</option>
              </select>
            </div>
            <div className="setting-item">
              <label>Cursor Blink</label>
              <input type="checkbox" defaultChecked />
            </div>
            <div className="setting-item">
              <label>Scrollback Lines</label>
              <input type="number" defaultValue={10000} min={1000} max={100000} />
            </div>
          </section>

          <section className="settings-section">
            <h3>SSH</h3>
            <div className="setting-item">
              <label>Default Port</label>
              <input type="number" defaultValue={22} min={1} max={65535} />
            </div>
            <div className="setting-item">
              <label>Keep Alive Interval (seconds)</label>
              <input type="number" defaultValue={60} min={0} max={300} />
            </div>
          </section>
        </>
      )}

      {activeSection === 'secrets' && (
        <section className="settings-section">
          <h3>Credential Management</h3>
          <p className="section-description">
            Store SSH keys, passwords, and tokens securely. Link them to servers for automatic authentication.
          </p>
          <SecretsManager />
        </section>
      )}

      {activeSection === 'about' && (
        <section className="settings-section">
          <h3>About Tacoshell</h3>
          <div className="about-info">
            <p><strong>Tacoshell</strong> v0.1.0</p>
            <p>Unified Infrastructure Management GUI</p>
            <p>Built with Rust + Tauri + React</p>
            <br />
            <p>🌮 A modern, fast, and secure way to manage your infrastructure.</p>
          </div>
        </section>
      )}
    </div>
  );
}

function SftpPlaceholder() {
  return (
    <div className="placeholder-panel">
      <div className="placeholder-content">
        <h2>📁 SFTP Browser</h2>
        <p>File transfer functionality coming in Phase 3</p>
        <ul>
          <li>Dual-pane file browser</li>
          <li>Drag & drop uploads</li>
          <li>Transfer queue with progress</li>
          <li>Resume interrupted transfers</li>
        </ul>
      </div>
    </div>
  );
}

function K8sPlaceholder() {
  return (
    <div className="placeholder-panel">
      <div className="placeholder-content">
        <h2>☸️ Kubernetes Dashboard</h2>
        <p>Cluster management coming in Phase 4</p>
        <ul>
          <li>Multi-cluster support</li>
          <li>Pod management & logs</li>
          <li>Exec into containers</li>
          <li>Resource visualization</li>
        </ul>
      </div>
    </div>
  );
}

export function SplitPaneLayout({ tabs }: SplitPaneLayoutProps) {
  const { activeTabId } = useAppStore();

  if (tabs.length === 0) {
    return (
      <div className="main-content empty">
        <div className="welcome">
          <h1>🌮 Tacoshell</h1>
          <p>Select a server from the sidebar to connect</p>
          <div className="quick-actions">
            <p className="hint">
              Use the sidebar to add and connect to servers.
              <br />
              Split views and Kubernetes support coming soon!
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="main-content">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          style={{
            display: tab.id === activeTabId ? 'block' : 'none',
            height: '100%',
            width: '100%',
          }}
        >
          <TileContent tabId={tab.id} />
        </div>
      ))}
    </div>
  );
}

export default SplitPaneLayout;





