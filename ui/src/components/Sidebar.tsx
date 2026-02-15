import { useEffect } from 'react';
import { useAppStore } from '../stores/appStore';
import { AddServerDialog } from './AddServerDialog';
import { ConnectDialog } from './ConnectDialog';
import { useConnectionManager } from '../hooks/useConnectionManager';

export function Sidebar() {
  const {
    sidebarOpen,
    toggleSidebar,
    addTab,
    setActiveTab,
  } = useAppStore();

  const {
    showAddDialog,
    setShowAddDialog,
    connectingServer,
    setConnectingServer,
    isConnecting,
    connectError,
    loadServers,
    handleConnect,
  } = useConnectionManager();

  useEffect(() => {
    loadServers();
  }, []);

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
    { id: 'clusters', icon: 'hub', label: 'Clusters', action: () => openView('kubernetes', 'k8s', 'Kubernetes') },
    { id: 'sftp', icon: 'folder_open', label: 'SFTP / FTP', action: () => openView('sftp', 'sftp', 'SFTP') },
    { id: 'secrets', icon: 'key', label: 'Keychain', action: () => openView('secrets', 'settings', 'Keychain') },
    { id: 'snippets', icon: 'code', label: 'Snippets', action: () => {} },
  ];

  return (
    <aside
      className={`${sidebarOpen ? 'w-64' : 'w-16'} bg-background-sidebar border-r border-white/5 flex flex-col justify-between shrink-0 transition-all duration-300 z-30`}
    >
      <div className="overflow-y-auto no-scrollbar flex-1">
        {/* Header / Logo */}
        <div className={`h-16 flex items-center border-b border-white/5 sticky top-0 bg-background-sidebar z-10 ${sidebarOpen ? 'px-6' : 'justify-center'}`}>
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

          {sidebarOpen && (
            <div className="mt-8 px-3">
              <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-4 px-3">Groups</h3>
              <div className="space-y-3 px-3">
                <button className="flex items-center w-full group">
                  <span className="w-2 h-2 rounded-full bg-emerald-500 mr-3 shadow-[0_0_8px_rgba(16,185,129,0.4)]"></span>
                  <span className="text-sm text-text-secondary group-hover:text-white">Production</span>
                  <span className="ml-auto text-xs text-slate-600 bg-white/5 px-1.5 py-0.5 rounded">12</span>
                </button>
                <button className="flex items-center w-full group">
                  <span className="w-2 h-2 rounded-full bg-amber-500 mr-3 shadow-[0_0_8px_rgba(245,158,11,0.4)]"></span>
                  <span className="text-sm text-text-secondary group-hover:text-white">Staging</span>
                  <span className="ml-auto text-xs text-slate-600 bg-white/5 px-1.5 py-0.5 rounded">4</span>
                </button>
                <button className="flex items-center w-full group">
                  <span className="w-2 h-2 rounded-full bg-purple-500 mr-3 shadow-[0_0_8px_rgba(168,85,247,0.4)]"></span>
                  <span className="text-sm text-text-secondary group-hover:text-white">AWS East</span>
                  <span className="ml-auto text-xs text-slate-600 bg-white/5 px-1.5 py-0.5 rounded">8</span>
                </button>
              </div>
            </div>
          )}

          <div className={`my-4 border-t border-white/5 mx-2`} />

          <button
            onClick={() => setShowAddDialog(true)}
            className={`flex items-center rounded-lg font-medium group transition-colors w-full ${
              sidebarOpen ? 'gap-3 px-3 py-2.5' : 'justify-center py-3'
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
      <div className={`p-4 border-t border-white/5 ${sidebarOpen ? '' : 'flex flex-col items-center gap-4'}`}>
        <button
          onClick={() => openView('settings', 'settings', 'Settings')}
          className={`flex items-center rounded-lg text-text-secondary hover:bg-white/5 hover:text-white transition-colors text-sm ${
            sidebarOpen ? 'gap-3 px-3 py-2 w-full' : 'p-2'
          }`}
        >
          {sidebarOpen ? (
            <>
              <img
                src="https://lh3.googleusercontent.com/aida-public/AB6AXuA_0dmYfq_iOoIXuNR1OhcdvqSQoWcJpcen7bXgZilPu88tw-pFsAc72TeecFdU0FtN9hxOC2m28wPWYpq4VehJSUlG8Q6F93P-eShrUMpzJRNBvzTMbrZyDwyidcG4KHjWgi1Ji0337RcosTiI8e0GrZomj5XBwc7WGXKxqG2fWUOPqncLhvYHRmglqLWuJuP3l6nWLlcNWvMiSuLaM0_HCZZIQYARLfnso-pu8cdHcIW0PMyh1cAHmkdwZPzcZ8y9z7PwXeW8L3BV"
                alt="User"
                className="w-8 h-8 rounded-full"
              />
              <div className="flex flex-col items-start overflow-hidden">
                <span className="font-medium truncate w-full text-left">Alex Doe</span>
                <span className="text-xs text-slate-500">Pro Plan</span>
              </div>
              <span className="material-icons-round ml-auto text-lg text-slate-500">settings</span>
            </>
          ) : (
            <span className="material-icons-round text-xl">settings</span>
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
