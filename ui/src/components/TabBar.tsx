import { useAppStore } from '../stores/appStore';
import { disconnectSsh } from '../hooks/useTauri';

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab, removeSession } = useAppStore();

  const handleCloseTab = async (tabId: string, sessionId?: string) => {
    if (sessionId) {
      try {
        await disconnectSsh(sessionId);
        removeSession(sessionId);
      } catch (error) {
        console.error('Error disconnecting:', error);
      }
    }
    removeTab(tabId);
  };

  const handleMouseDown = (e: React.MouseEvent, tabId: string, sessionId?: string) => {
    if (e.button === 1) {
      e.preventDefault();
      handleCloseTab(tabId, sessionId);
    }
  };

  if (tabs.length === 0) {
    return null;
  }

  return (
    <header className="h-14 bg-background-dark border-b border-white/5 flex items-center justify-between px-4 flex-shrink-0">
      {/* Tabs */}
      <div className="flex items-center gap-1 overflow-x-auto no-scrollbar">
        {tabs.map((tab) => {
          const isActive = activeTabId === tab.id;
          return (
            <div
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              onMouseDown={(e) => handleMouseDown(e, tab.id, tab.sessionId)}
              className={`group flex items-center gap-2 px-4 py-2 rounded-lg cursor-pointer min-w-[160px] transition-all relative ${
                isActive
                  ? 'bg-primary/10 border border-primary/20 rounded-t-lg top-[1px] cursor-default'
                  : 'hover:bg-white/5 text-slate-400 border border-transparent hover:border-white/10'
              }`}
            >
              <div className={`w-2 h-2 rounded-full ${
                  tab.type === 'terminal' ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]' :
                  tab.type === 'sftp' ? 'bg-blue-500' : 'bg-yellow-500'
              }`}></div>
              <span className={`text-sm font-medium ${isActive ? 'text-primary' : ''}`}>
                {tab.title}
              </span>
              <button
                className={`ml-auto p-0.5 rounded-full transition-opacity ${
                  isActive
                    ? 'text-primary/50 hover:text-primary hover:bg-primary/10'
                    : 'opacity-0 group-hover:opacity-100 text-slate-400 hover:text-white hover:bg-white/10'
                }`}
                onClick={(e) => {
                  e.stopPropagation();
                  handleCloseTab(tab.id, tab.sessionId);
                }}
              >
                <span className="material-icons-round text-[14px]">close</span>
              </button>
              {isActive && (
                <div className="absolute bottom-[-1px] left-0 w-full h-[2px] bg-primary"></div>
              )}
            </div>
          );
        })}
      </div>

      {/* Global Actions */}
      <div className="flex items-center gap-2">
        <div className="h-6 w-px bg-white/10 mx-2"></div>
        <button className="p-2 text-slate-400 hover:text-white rounded-lg hover:bg-white/5" title="Split Screen">
          <span className="material-icons-round text-[20px]">splitscreen</span>
        </button>
        <button className="p-2 text-slate-400 hover:text-white rounded-lg hover:bg-white/5" title="Search Logs">
          <span className="material-icons-round text-[20px]">search</span>
        </button>
        <button className="p-2 text-slate-400 hover:text-white rounded-lg hover:bg-white/5" title="Toggle Right Panel">
          <span className="material-icons-round text-[20px]">vertical_split</span>
        </button>
      </div>
    </header>
  );
}
