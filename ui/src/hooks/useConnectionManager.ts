import { useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { fetchServers, connectSsh } from './useTauri';
import type { Server as ServerType } from '../types';

export function useConnectionManager() {
  const { setServers, addSession, addTab, setActiveTab } = useAppStore();
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [connectingServer, setConnectingServer] = useState<ServerType | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectError, setConnectError] = useState<string | null>(null);

  const loadServers = async () => {
    try {
      const data = await fetchServers();
      setServers(data);
    } catch (error) {
      console.error('Failed to load servers:', error);
    }
  };

  const handleConnect = async (password?: string, privateKey?: string, passphrase?: string) => {
    if (!connectingServer) return;

    const existingTab = useAppStore.getState().tabs.find(
      t => t.type === 'terminal' && t.serverId === connectingServer.id
    );

    if (existingTab) {
      setActiveTab(existingTab.id);
      setConnectingServer(null);
      return;
    }

    setIsConnecting(true);
    setConnectError(null);

    try {
      const session = await connectSsh({
        server_id: connectingServer.id,
        password,
        private_key: privateKey,
        passphrase,
      });

      addSession({
        sessionId: session.session_id,
        serverId: connectingServer.id,
        connected: true,
      });

      addTab({
        id: `terminal-${session.session_id}`,
        type: 'terminal',
        title: connectingServer.name,
        serverId: connectingServer.id,
        sessionId: session.session_id,
      });

      setConnectingServer(null);
    } catch (error: unknown) {
      console.error('Failed to connect:', error);
      let message = 'Unknown error';
      if (error instanceof Error && typeof error.message === 'string') {
        message = error.message;
      } else {
        message = String(error);
      }
      setConnectError(message);
    } finally {
      setIsConnecting(false);
    }
  };

  return {
    showAddDialog,
    setShowAddDialog,
    connectingServer,
    setConnectingServer,
    isConnecting,
    connectError,
    loadServers,
    handleConnect,
  };
}
