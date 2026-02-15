import { useAppStore } from '../stores/appStore';

export function Sidebar() {
  const {
    sidebarOpen,
    toggleSidebar,
    addTab,
    setActiveTab,
  } = useAppStore();

  const openView = (id: string, type: 'settings' | 'terminal' | 'sftp' | 'k8s', title: string) => {
    const existingTab = useAppStore.getState().tabs.find(t => t.id === id);
    if (existingTab) {
      setActiveTab(id);
    } else {
      addTab({ id, type, title });
    }
  };

  const navItems = [
    { id: 'hosts', icon: 'grid_view', label: 'Hosts', action: () => setActiveTab(null) },
    { id: 'clusters', icon: 'hub', label: 'Clusters', action: () => openView('kubernetes', 'k8s', 'Kubernetes') },
    { id: 'sftp', icon: 'folder_open', label: 'SFTP / FTP', action: () => openView('sftp', 'sftp', 'SFTP') },
    { id: 'secrets', icon: 'key', label: 'Keychain', action: () => openView('secrets', 'settings', 'Keychain') },
    { id: 'snippets', icon: 'code', label: 'Snippets', action: () => openView('snippets', 'settings', 'Snippets'), disabled: true },
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
              disabled={item.disabled}
              className={`flex items-center rounded-lg font-medium group transition-colors w-full ${
                sidebarOpen ? 'gap-3 px-3 py-2.5' : 'justify-center py-3'
              } ${item.disabled ? 'text-slate-600 cursor-not-allowed opacity-50' : 'text-text-secondary hover:bg-white/5 hover:text-white'} relative`}
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
              <svg
                className="w-8 h-8 rounded-full"
                viewBox="0 0 64 64"
                xmlns="http://www.w3.org/2000/svg"
              >
                <defs>
                  <linearGradient id="avatar-bg" x1="0%" y1="0%" x2="100%" y2="100%">
                    <stop offset="0%" stopColor="#3b82f6"/>
                    <stop offset="100%" stopColor="#6b21a8"/>
                  </linearGradient>
                </defs>
                <circle cx="32" cy="32" r="32" fill="url(#avatar-bg)"/>
                <text
                  x="50%"
                  y="52%"
                  textAnchor="middle"
                  dominantBaseline="middle"
                  fontFamily="system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
                  fontSize="28"
                  fill="white"
                >
                  A
                </text>
              </svg>
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
    </aside>
  );
}
