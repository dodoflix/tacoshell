import { invoke } from '@tauri-apps/api/core'
import type {
  VaultService,
  VaultLoadResult,
  VaultPushResult,
  VaultItem,
} from '@tacoshell/ui/stores'

interface TauriLoadResult {
  items: Array<{
    id: string
    type: string
    payload: Record<string, unknown>
    created_at: string
    updated_at: string
  }>
  sha: string
}

interface TauriSaveResult {
  sha: string
}

export class TauriVaultService implements VaultService {
  async loadVault(token: string, passphrase: string): Promise<VaultLoadResult> {
    const result = await invoke<TauriLoadResult>('load_vault', {
      token,
      passphrase,
      githubUserId: await this.getGitHubUserId(token),
    })
    const items: VaultItem[] = result.items.map((i) => ({
      id: i.id,
      type: i.type as VaultItem['type'],
      payload: i.payload,
      createdAt: i.created_at,
      updatedAt: i.updated_at,
    }))
    return { items, sha: result.sha }
  }

  async pushVault(
    token: string,
    passphrase: string,
    items: VaultItem[],
    currentSha: string | null,
  ): Promise<VaultPushResult> {
    const tauriItems = items.map((i) => ({
      id: i.id,
      type: i.type,
      payload: i.payload,
      created_at: i.createdAt,
      updated_at: i.updatedAt,
    }))
    const result = await invoke<TauriSaveResult>('save_vault', {
      token,
      passphrase,
      githubUserId: await this.getGitHubUserId(token),
      items: tauriItems,
      currentSha: currentSha ?? '',
    })
    return { sha: result.sha }
  }

  private async getGitHubUserId(token: string): Promise<string> {
    const profile = await invoke<{ login: string }>('get_user_profile', { token })
    return profile.login
  }
}
