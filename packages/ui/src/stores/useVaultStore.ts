import { create } from 'zustand'

export type VaultItemType = 'connection_profile' | 'ssh_key' | 'password' | 'kube_config'

export interface VaultItem {
  id: string
  type: VaultItemType
  payload: Record<string, unknown>
  createdAt: string
  updatedAt: string
}

export interface VaultLoadResult {
  items: VaultItem[]
  sha: string
}

export interface VaultPushResult {
  sha: string
}

export interface VaultService {
  /** Load and decrypt the vault from the remote (or offline cache). */
  loadVault(token: string, passphrase: string): Promise<VaultLoadResult>
  /** Encrypt and push the full item list to remote. */
  pushVault(
    token: string,
    passphrase: string,
    items: VaultItem[],
    currentSha: string | null,
  ): Promise<VaultPushResult>
}

export type SyncStatus = 'idle' | 'loading' | 'syncing' | 'synced' | 'offline' | 'error'

export interface VaultState {
  items: VaultItem[]
  syncStatus: SyncStatus
  lastSyncAt: Date | null
  currentSha: string | null
  error: string | null

  setService: (service: VaultService) => void
  load: (token: string, passphrase: string) => Promise<void>
  sync: (token: string, passphrase: string) => Promise<void>
  addItem: (item: Omit<VaultItem, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>
  updateItem: (item: VaultItem) => Promise<void>
  deleteItem: (id: string) => Promise<void>

  _service: VaultService | null
  _token: string | null
  _passphrase: string | null
  setCredentials: (token: string, passphrase: string) => void
}

function isOfflineError(message: string): boolean {
  return /offline|network|fetch/i.test(message)
}

async function loadFromService(
  get: () => VaultState,
  set: (partial: Partial<VaultState>) => void,
  token: string,
  passphrase: string,
  startStatus: SyncStatus,
): Promise<void> {
  const { _service } = get()
  if (_service === null) {
    throw new Error('VaultService not initialized')
  }
  set({ syncStatus: startStatus, error: null })
  get().setCredentials(token, passphrase)
  try {
    const result = await _service.loadVault(token, passphrase)
    set({
      items: result.items,
      currentSha: result.sha,
      syncStatus: 'synced',
      lastSyncAt: new Date(),
    })
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err)
    if (isOfflineError(message)) {
      set({ syncStatus: 'offline', error: message })
    } else {
      set({ syncStatus: 'error', error: message })
    }
  }
}

export const useVaultStore = create<VaultState>()((set, get) => ({
  items: [],
  syncStatus: 'idle',
  lastSyncAt: null,
  currentSha: null,
  error: null,
  _service: null,
  _token: null,
  _passphrase: null,

  setService: (service: VaultService) => set({ _service: service }),

  setCredentials: (token: string, passphrase: string) =>
    set({ _token: token, _passphrase: passphrase }),

  load: (token: string, passphrase: string) =>
    loadFromService(get, set, token, passphrase, 'loading'),

  sync: (token: string, passphrase: string) =>
    loadFromService(get, set, token, passphrase, 'syncing'),

  addItem: async (partial: Omit<VaultItem, 'id' | 'createdAt' | 'updatedAt'>) => {
    const { _service, _token, _passphrase, items, currentSha } = get()
    if (_service === null) throw new Error('VaultService not initialized')
    if (_token === null || _passphrase === null) throw new Error('Credentials not set')

    const now = new Date().toISOString()
    const newItem: VaultItem = {
      ...partial,
      id: crypto.randomUUID(),
      createdAt: now,
      updatedAt: now,
    }
    const newItems = [...items, newItem]
    set({ items: newItems })

    try {
      const result = await _service.pushVault(_token, _passphrase, newItems, currentSha)
      set({ currentSha: result.sha })
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      set({ items, syncStatus: 'error', error: message })
    }
  },

  updateItem: async (item: VaultItem) => {
    const { _service, _token, _passphrase, items, currentSha } = get()
    if (_service === null) throw new Error('VaultService not initialized')
    if (_token === null || _passphrase === null) throw new Error('Credentials not set')

    const updatedItem = { ...item, updatedAt: new Date().toISOString() }
    const newItems = items.map((i) => (i.id === item.id ? updatedItem : i))
    set({ items: newItems })

    try {
      const result = await _service.pushVault(_token, _passphrase, newItems, currentSha)
      set({ currentSha: result.sha })
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      set({ items, syncStatus: 'error', error: message })
    }
  },

  deleteItem: async (id: string) => {
    const { _service, _token, _passphrase, items, currentSha } = get()
    if (_service === null) throw new Error('VaultService not initialized')
    if (_token === null || _passphrase === null) throw new Error('Credentials not set')

    const newItems = items.filter((i) => i.id !== id)
    set({ items: newItems })

    try {
      const result = await _service.pushVault(_token, _passphrase, newItems, currentSha)
      set({ currentSha: result.sha })
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      set({ items, syncStatus: 'error', error: message })
    }
  },
}))
