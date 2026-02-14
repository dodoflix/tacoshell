// Tab bar component

import { X } from 'lucide-react';
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
        >
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

