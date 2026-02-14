// Tab bar component

import { X, Terminal, FolderOpen, Box } from 'lucide-react';
import { useAppStore } from '../stores/appStore';
import { disconnectSsh } from '../hooks/useTauri';

export function TabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab, removeSession } = useAppStore();

  const getTabIcon = (type: string) => {
    switch (type) {
      case 'terminal':
        return <Terminal size={14} />;
      case 'sftp':
        return <FolderOpen size={14} />;
      case 'k8s':
        return <Box size={14} />;
      default:
        return null;
    }
  };

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
    // 1 is middle click
    if (e.button === 1) {
      e.preventDefault();
      handleCloseTab(tabId, sessionId);
    }
  };

  if (tabs.length === 0) {
    return null;
  }

  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`tab ${activeTabId === tab.id ? 'active' : ''}`}
          onClick={() => setActiveTab(tab.id)}
          onMouseDown={(e) => handleMouseDown(e, tab.id, tab.sessionId)}
        >
          {getTabIcon(tab.type)}
          <span className="tab-title">{tab.title}</span>
          <button
            className="tab-close"
            onClick={(e) => {
              e.stopPropagation();
              handleCloseTab(tab.id, tab.sessionId);
            }}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}

