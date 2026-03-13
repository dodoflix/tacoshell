import { vi } from 'vitest'
import { useVaultStore } from '../useVaultStore'
import type { VaultService, VaultItem } from '../useVaultStore'

const makeItem = (overrides: Partial<VaultItem> = {}): VaultItem => ({
  id: 'item-1',
  type: 'connection_profile',
  payload: { host: 'example.com' },
  createdAt: '2026-01-01T00:00:00.000Z',
  updatedAt: '2026-01-01T00:00:00.000Z',
  ...overrides,
})

const mockService = (): VaultService => ({
  loadVault: vi.fn(),
  pushVault: vi.fn(),
})

beforeEach(() => {
  useVaultStore.setState({
    items: [],
    syncStatus: 'idle',
    lastSyncAt: null,
    currentSha: null,
    error: null,
    _service: null,
    _token: null,
    _passphrase: null,
  })
})

describe('useVaultStore', () => {
  describe('initial state', () => {
    it('has idle syncStatus, empty items, and null nullable fields', () => {
      const state = useVaultStore.getState()
      expect(state.syncStatus).toBe('idle')
      expect(state.items).toEqual([])
      expect(state.lastSyncAt).toBeNull()
      expect(state.currentSha).toBeNull()
      expect(state.error).toBeNull()
      expect(state._service).toBeNull()
      expect(state._token).toBeNull()
      expect(state._passphrase).toBeNull()
    })
  })

  describe('setService', () => {
    it('sets _service on the store', () => {
      const svc = mockService()
      useVaultStore.getState().setService(svc)
      expect(useVaultStore.getState()._service).toBe(svc)
    })
  })

  describe('load', () => {
    it('calls loadVault with token and passphrase on success', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        items: [item],
        sha: 'abc123',
      })
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().load('tok', 'pass')
      expect(svc.loadVault).toHaveBeenCalledWith('tok', 'pass')
    })

    it('sets items, currentSha, syncStatus=synced, lastSyncAt on success', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        items: [item],
        sha: 'sha-xyz',
      })
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().load('tok', 'pass')
      const state = useVaultStore.getState()
      expect(state.items).toEqual([item])
      expect(state.currentSha).toBe('sha-xyz')
      expect(state.syncStatus).toBe('synced')
      expect(state.lastSyncAt).not.toBeNull()
    })

    it('sets syncStatus=offline and error when loadVault rejects with network error', async () => {
      const svc = mockService()
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('network error'))
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().load('tok', 'pass')
      const state = useVaultStore.getState()
      expect(state.syncStatus).toBe('offline')
      expect(state.error).toContain('network error')
    })

    it('sets syncStatus=error and error when loadVault rejects with generic error', async () => {
      const svc = mockService()
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
        new Error('something went wrong'),
      )
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().load('tok', 'pass')
      const state = useVaultStore.getState()
      expect(state.syncStatus).toBe('error')
      expect(state.error).toContain('something went wrong')
    })
  })

  describe('sync', () => {
    it('sets syncStatus=synced on success', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
        items: [item],
        sha: 'sha-sync',
      })
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().sync('tok', 'pass')
      expect(useVaultStore.getState().syncStatus).toBe('synced')
    })

    it('calls loadVault (same as load) but starts with syncing status', async () => {
      const svc = mockService()
      let capturedStatus: string | null = null
      ;(svc.loadVault as ReturnType<typeof vi.fn>).mockImplementationOnce(async () => {
        capturedStatus = useVaultStore.getState().syncStatus
        return { items: [], sha: 'sha-s' }
      })
      useVaultStore.getState().setService(svc)
      await useVaultStore.getState().sync('tok', 'pass')
      expect(capturedStatus).toBe('syncing')
    })
  })

  describe('addItem', () => {
    it('adds item with generated id, createdAt, updatedAt and calls pushVault', async () => {
      const svc = mockService()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ sha: 'new-sha' })
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({ _token: 'tok', _passphrase: 'pass', currentSha: 'old-sha' })

      await useVaultStore.getState().addItem({
        type: 'ssh_key',
        payload: { key: 'rsa...' },
      })

      const state = useVaultStore.getState()
      expect(state.items).toHaveLength(1)
      const added = state.items[0]
      expect(added.id).toBeTruthy()
      expect(added.type).toBe('ssh_key')
      expect(added.createdAt).toBeTruthy()
      expect(added.updatedAt).toBeTruthy()
      expect(svc.pushVault).toHaveBeenCalledWith('tok', 'pass', state.items, 'old-sha')
    })

    it('updates currentSha from pushVault result', async () => {
      const svc = mockService()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ sha: 'pushed-sha' })
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({ _token: 'tok', _passphrase: 'pass', currentSha: null })

      await useVaultStore.getState().addItem({ type: 'password', payload: { pwd: 'secret' } })
      expect(useVaultStore.getState().currentSha).toBe('pushed-sha')
    })

    it('reverts items on pushVault failure', async () => {
      const svc = mockService()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('push failed'))
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({ _token: 'tok', _passphrase: 'pass', items: [], currentSha: null })

      await useVaultStore.getState().addItem({ type: 'password', payload: { pwd: 'secret' } })

      const state = useVaultStore.getState()
      expect(state.items).toHaveLength(0)
      expect(state.syncStatus).toBe('error')
      expect(state.error).toContain('push failed')
    })
  })

  describe('updateItem', () => {
    it('replaces item in items, refreshes updatedAt, and calls pushVault', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ sha: 'upd-sha' })
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({
        _token: 'tok',
        _passphrase: 'pass',
        items: [item],
        currentSha: 'old',
      })

      const updated = { ...item, payload: { host: 'updated.com' } }
      await useVaultStore.getState().updateItem(updated)

      const state = useVaultStore.getState()
      expect(state.items).toHaveLength(1)
      expect(state.items[0].payload).toEqual({ host: 'updated.com' })
      expect(state.items[0].updatedAt).not.toBe(item.updatedAt)
      expect(svc.pushVault).toHaveBeenCalled()
      expect(state.currentSha).toBe('upd-sha')
    })
  })

  describe('deleteItem', () => {
    it('removes item from items and calls pushVault', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ sha: 'del-sha' })
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({
        _token: 'tok',
        _passphrase: 'pass',
        items: [item],
        currentSha: 'old',
      })

      await useVaultStore.getState().deleteItem('item-1')

      const state = useVaultStore.getState()
      expect(state.items).toHaveLength(0)
      expect(svc.pushVault).toHaveBeenCalled()
      expect(state.currentSha).toBe('del-sha')
    })

    it('reverts items on pushVault failure', async () => {
      const svc = mockService()
      const item = makeItem()
      ;(svc.pushVault as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('del failed'))
      useVaultStore.getState().setService(svc)
      useVaultStore.setState({
        _token: 'tok',
        _passphrase: 'pass',
        items: [item],
        currentSha: 'old',
      })

      await useVaultStore.getState().deleteItem('item-1')

      const state = useVaultStore.getState()
      expect(state.items).toHaveLength(1)
      expect(state.error).toContain('del failed')
    })
  })
})
