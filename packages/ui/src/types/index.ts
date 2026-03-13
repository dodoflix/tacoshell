// Shared TypeScript types

export type Protocol = 'ssh' | 'sftp' | 'ftp' | 'k8s'

export type TabType = 'terminal' | 'sftp' | 'k8s' | 'welcome'

export type ConnectionStatus =
  | 'idle'
  | 'connecting'
  | 'authenticating'
  | 'connected'
  | 'reconnecting'
  | 'disconnecting'
  | 'disconnected'
  | 'error'

export interface Tab {
  id: string
  connectionId: string | null
  type: TabType
  title: string
  isDirty: boolean
}

export interface PaneNode {
  type: 'pane'
  id: string
  tabs: Tab[]
  activeTabId: string | null
}

export interface SplitNode {
  type: 'split'
  id: string
  direction: 'horizontal' | 'vertical'
  ratio: number // 0.0–1.0
  first: TabNode
  second: TabNode
}

export type TabNode = PaneNode | SplitNode

export interface ActiveConnection {
  id: string
  profileId: string
  status: ConnectionStatus
  error: string | null
  connectedAt: Date | null
  reconnectAttempts: number
}
