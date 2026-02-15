// Zustand store for application state

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Server, Secret, Tab, Session } from '../types';

interface AppState {
  // Data
  servers: Server[];
  secrets: Secret[];
  sessions: Map<string, Session>;

  // UI State
  tabs: Tab[];
  activeTabId: string | null;
  sidebarOpen: boolean;
  theme: 'dark' | 'light';
  fontSize: number;
  fontFamily: string;

  // Actions
  setServers: (servers: Server[]) => void;
  addServer: (server: Server) => void;
  removeServer: (id: string) => void;

  setSecrets: (secrets: Secret[]) => void;
  addSecret: (secret: Secret) => void;
  removeSecret: (id: string) => void;

  addSession: (session: Session) => void;
  removeSession: (sessionId: string) => void;

  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
  setActiveTab: (id: string | null) => void;

  toggleSidebar: () => void;
  setTheme: (theme: 'dark' | 'light') => void;
  setFontSize: (fontSize: number) => void;
  setFontFamily: (fontFamily: string) => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      // Initial state
      servers: [],
      secrets: [],
      sessions: new Map(),
      tabs: [],
      activeTabId: null,
      sidebarOpen: true,
      theme: 'dark',
      fontSize: 14,
      fontFamily: 'Consolas, "Courier New", monospace',

      // Server actions
      setServers: (servers) => set({ servers }),
      addServer: (server) => set((state) => ({
        servers: [...state.servers, server]
      })),
      removeServer: (id) => set((state) => ({
        servers: state.servers.filter((s) => s.id !== id)
      })),

      // Secret actions
      setSecrets: (secrets) => set({ secrets }),
      addSecret: (secret) => set((state) => ({
        secrets: [...state.secrets, secret]
      })),
      removeSecret: (id) => set((state) => ({
        secrets: state.secrets.filter((s) => s.id !== id)
      })),

      // Session actions
      addSession: (session) => set((state) => {
        const sessions = new Map(state.sessions);
        sessions.set(session.sessionId, session);
        return { sessions };
      }),
      removeSession: (sessionId) => set((state) => {
        const sessions = new Map(state.sessions);
        sessions.delete(sessionId);
        return { sessions };
      }),

      // Tab actions
      addTab: (tab) => set((state) => {
        const existingTab = state.tabs.find((t) => t.id === tab.id);
        if (existingTab) {
          console.log(`Tab ${tab.id} already exists, activating it.`);
          return { activeTabId: tab.id };
        }
        console.log(`Adding new tab ${tab.id}`);
        return {
          tabs: [...state.tabs, tab],
          activeTabId: tab.id,
        };
      }),
      removeTab: (id) => set((state) => {
        const newTabs = state.tabs.filter((t) => t.id !== id);
        let activeTabId = state.activeTabId;

        if (activeTabId === id) {
          activeTabId = newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null;
        }

        return { tabs: newTabs, activeTabId };
      }),
      setActiveTab: (id: string | null) => set({ activeTabId: id }),

      // UI actions
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      setTheme: (theme: 'dark' | 'light') => set({ theme }),
      setFontSize: (fontSize: number) => set({ fontSize }),
      setFontFamily: (fontFamily: string) => set({ fontFamily }),
    }),
    {
      name: 'tacoshell-storage',
      partialize: (state) => ({
        sidebarOpen: state.sidebarOpen,
        tabs: state.tabs,
        activeTabId: state.activeTabId,
        theme: state.theme,
        fontSize: state.fontSize,
        fontFamily: state.fontFamily,
      }),
    }
  )
);
