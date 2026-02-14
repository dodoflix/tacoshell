// Type definitions for Tacoshell

export interface Server {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  protocol: 'ssh' | 'sftp' | 'ftp';
  tags: string[];
}

export interface Secret {
  id: string;
  name: string;
  kind: 'password' | 'private_key' | 'token' | 'kubeconfig';
}

export interface Session {
  sessionId: string;
  serverId: string;
  connected: boolean;
}

export interface Tab {
  id: string;
  type: 'terminal' | 'sftp' | 'k8s' | 'settings';
  title: string;
  serverId?: string;
  sessionId?: string;
}

export interface AddServerRequest {
  name: string;
  host: string;
  port: number;
  username: string;
  protocol?: string;
  tags?: string[];
}

export interface AddSecretRequest {
  name: string;
  kind: string;
  value: string;
}

export interface ConnectRequest {
  server_id: string;
  password?: string;
  private_key?: string;
  passphrase?: string;
}

export interface SshOutputResponse {
  data: string;
  eof: boolean;
}

