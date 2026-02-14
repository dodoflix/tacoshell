import { useAppStore } from '../stores/appStore';
import { useState } from 'react';

export function GlobalSidebar() {
  const { setActiveTab, addTab } = useAppStore();
  const [activeNav, setActiveNav] = useState('hosts');

  const navItems = [
    { id: 'hosts', icon: 'storage', label: 'Hosts' },
    { id: 'terminal', icon: 'terminal', label: 'Terminal' },
    { id: 'sftp', icon: 'folder_shared', label: 'SFTP' },
    { id: 'keys', icon: 'vpn_key', label: 'Keys' },
  ];

  const handleNavClick = (id: string) => {
    setActiveNav(id);
    if (id === 'keys') {
        const existingTab = useAppStore.getState().tabs.find(t => t.id === 'secrets');
        if (existingTab) {
          setActiveTab('secrets');
        } else {
          addTab({ id: 'secrets', type: 'settings', title: 'Secrets' });
        }
    }
    // For others, we might want to switch views or something
  };

  return (
    <aside className="w-16 bg-white dark:bg-panel-dark border-r border-gray-200 dark:border-border-dark flex flex-col items-center py-6 gap-6 z-20 flex-shrink-0">
      {/* Logo */}
      <div className="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center text-primary mb-2">
        <span className="material-icons-round text-2xl">dns</span>
      </div>

      {/* Nav Items */}
      <nav className="flex flex-col gap-4 w-full">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => handleNavClick(item.id)}
            className={`w-10 h-10 mx-auto rounded-lg flex items-center justify-center transition-colors group relative ${
              activeNav === item.id
                ? 'bg-primary text-white shadow-lg shadow-primary/30'
                : 'text-slate-400 hover:text-primary hover:bg-primary/10'
            }`}
          >
            <span className="material-icons-round">{item.icon}</span>
            <span className="absolute left-12 bg-gray-900 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-50">
              {item.label}
            </span>
          </button>
        ))}
      </nav>

      <div className="mt-auto flex flex-col gap-4 w-full">
        <button
          onClick={() => {
            const existingTab = useAppStore.getState().tabs.find(t => t.id === 'settings');
            if (existingTab) {
              setActiveTab('settings');
            } else {
              addTab({ id: 'settings', type: 'settings', title: 'Settings' });
            }
          }}
          className="w-10 h-10 mx-auto rounded-lg flex items-center justify-center text-slate-400 hover:text-primary hover:bg-primary/10 transition-colors"
        >
          <span className="material-icons-round">settings</span>
        </button>
        <div className="w-8 h-8 mx-auto rounded-full bg-gradient-to-tr from-primary to-purple-500 overflow-hidden ring-2 ring-background-dark">
          <img
            alt="User avatar"
            className="w-full h-full object-cover"
            src="https://lh3.googleusercontent.com/aida-public/AB6AXuB98cpgJZi12KsQvrqjisyWtRJIZfWYrY0zVM-1-eYAL901eEfGl5L5zSg4R3cn1p39uht9ezeWcgZxF_dmZ9AhCyp6Xo-5WviU-jNiyJqUBpVqitEHELyda3m3zgbqoLiRYMUb_PhGihVncPS_NSeWidIhV2de3YVMWslJ1CGo9IA0a1d83mH-WRgRBLsQs2xFykS4a7QGKl6j6xVKq2tBd43zKVMcXADu9pAONvTSZvYT4YMVHm1p-lB_GLI7a6QgMe4YxlKN9Nbu"
          />
        </div>
      </div>
    </aside>
  );
}
