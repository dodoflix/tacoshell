import { vi, describe, it, expect, beforeEach } from 'vitest'
import { TauriVaultService } from '../TauriVaultService'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
const mockInvoke = vi.mocked(invoke)

const MOCK_PROFILE = { login: 'testuser' }

const MOCK_LOAD_RESULT = {
  items: [
    {
      id: 'item-1',
      type: 'connection_profile',
      payload: { host: 'example.com', port: 22, type: 'connection_profile' },
      created_at: '2026-01-01T00:00:00Z',
      updated_at: '2026-01-02T00:00:00Z',
    },
  ],
  sha: 'abc123',
}

const MOCK_SAVE_RESULT = {
  sha: 'def456',
}

describe('TauriVaultService', () => {
  let service: TauriVaultService

  beforeEach(() => {
    service = new TauriVaultService()
    vi.clearAllMocks()
  })

  describe('loadVault', () => {
    it('calls invoke("get_user_profile") to get github_user_id', async () => {
      mockInvoke
        .mockResolvedValueOnce(MOCK_PROFILE) // get_user_profile
        .mockResolvedValueOnce(MOCK_LOAD_RESULT) // load_vault

      await service.loadVault('test-token', 'my-passphrase')

      expect(mockInvoke).toHaveBeenCalledWith('get_user_profile', { token: 'test-token' })
    })

    it('calls invoke("load_vault") with token, passphrase, and githubUserId', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockResolvedValueOnce(MOCK_LOAD_RESULT)

      await service.loadVault('test-token', 'my-passphrase')

      expect(mockInvoke).toHaveBeenCalledWith('load_vault', {
        token: 'test-token',
        passphrase: 'my-passphrase',
        githubUserId: 'testuser',
      })
    })

    it('maps Tauri result to VaultLoadResult with camelCase fields', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockResolvedValueOnce(MOCK_LOAD_RESULT)

      const result = await service.loadVault('test-token', 'my-passphrase')

      expect(result.sha).toBe('abc123')
      expect(result.items).toHaveLength(1)
      expect(result.items[0]).toMatchObject({
        id: 'item-1',
        type: 'connection_profile',
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-02T00:00:00Z',
      })
    })

    it('propagates errors from invoke', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockRejectedValueOnce(new Error('load failed'))

      await expect(service.loadVault('test-token', 'my-passphrase')).rejects.toThrow('load failed')
    })
  })

  describe('pushVault', () => {
    const mockItems = [
      {
        id: 'item-1',
        type: 'connection_profile' as const,
        payload: { host: 'example.com' },
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-02T00:00:00Z',
      },
    ]

    it('calls invoke("save_vault") with mapped items', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockResolvedValueOnce(MOCK_SAVE_RESULT)

      await service.pushVault('test-token', 'my-passphrase', mockItems, 'current-sha')

      expect(mockInvoke).toHaveBeenCalledWith('save_vault', {
        token: 'test-token',
        passphrase: 'my-passphrase',
        githubUserId: 'testuser',
        items: [
          {
            id: 'item-1',
            type: 'connection_profile',
            payload: { host: 'example.com' },
            created_at: '2026-01-01T00:00:00Z',
            updated_at: '2026-01-02T00:00:00Z',
          },
        ],
        currentSha: 'current-sha',
      })
    })

    it('returns VaultPushResult with sha', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockResolvedValueOnce(MOCK_SAVE_RESULT)

      const result = await service.pushVault(
        'test-token',
        'my-passphrase',
        mockItems,
        'current-sha',
      )

      expect(result.sha).toBe('def456')
    })

    it('converts null currentSha to empty string', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockResolvedValueOnce(MOCK_SAVE_RESULT)

      await service.pushVault('test-token', 'my-passphrase', mockItems, null)

      expect(mockInvoke).toHaveBeenCalledWith(
        'save_vault',
        expect.objectContaining({ currentSha: '' }),
      )
    })

    it('propagates errors from invoke', async () => {
      mockInvoke.mockResolvedValueOnce(MOCK_PROFILE).mockRejectedValueOnce(new Error('push failed'))

      await expect(
        service.pushVault('test-token', 'my-passphrase', mockItems, 'sha'),
      ).rejects.toThrow('push failed')
    })
  })
})
