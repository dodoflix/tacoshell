// Custom hooks for Tauri commands

import { invoke } from '@tauri-apps/api/core';
import type {
  Server,
  Secret,
  AddServerRequest,
  AddSecretRequest,
  ConnectRequest,
  SshOutputResponse
} from '../types';

// Server hooks
export async function fetchServers(): Promise<Server[]> {
  return invoke<Server[]>('get_servers');
}

export async function createServer(request: AddServerRequest): Promise<Server> {
  return invoke<Server>('add_server', { request });
}

export async function updateServer(server: Server): Promise<void> {
  return invoke('update_server', { request: server });
}

export async function deleteServer(id: string): Promise<void> {
  return invoke('delete_server', { id });
}

// Secret hooks
export async function fetchSecrets(): Promise<Secret[]> {
  return invoke<Secret[]>('get_secrets');
}

export async function createSecret(request: AddSecretRequest): Promise<Secret> {
  return invoke<Secret>('add_secret', { request });
}

export async function deleteSecret(id: string): Promise<void> {
  return invoke('delete_secret', { id });
}

export async function linkSecretToServer(
  serverId: string,
  secretId: string,
  priority?: number
): Promise<void> {
  return invoke('link_secret_to_server', {
    server_id: serverId,
    secret_id: secretId,
    priority
  });
}

export async function unlinkSecretFromServer(
  serverId: string,
  secretId: string
): Promise<void> {
  return invoke('unlink_secret_from_server', {
    server_id: serverId,
    secret_id: secretId
  });
}

// SSH hooks
export interface SessionResponse {
  session_id: string;
  server_id: string;
  connected: boolean;
}

export async function connectSsh(request: ConnectRequest): Promise<SessionResponse> {
  return invoke<SessionResponse>('connect_ssh', { request });
}

export async function disconnectSsh(sessionId: string): Promise<void> {
  return invoke('disconnect_ssh', { session_id: sessionId });
}

export async function sendSshInput(
  sessionId: string,
  input: string
): Promise<SshOutputResponse> {
  return invoke<SshOutputResponse>('send_ssh_input', {
    session_id: sessionId,
    input
  });
}

export async function resizeTerminal(
  sessionId: string,
  cols: number,
  rows: number
): Promise<void> {
  return invoke('resize_terminal', {
    session_id: sessionId,
    cols,
    rows
  });
}

