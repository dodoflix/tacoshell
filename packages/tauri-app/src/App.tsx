import { useState, useEffect } from 'react'
import { SessionGuard, VaultPassphraseModal, GitHubLoginButton } from '@tacoshell/ui/features/auth'
import { useAuthStore, useVaultStore } from '@tacoshell/ui/stores'
import { TauriVaultService } from './services/TauriVaultService'

const vaultService = new TauriVaultService()

export default function App() {
  const { status, token, user } = useAuthStore()
  const { setService, load, syncStatus } = useVaultStore()
  const [showPassphrase, setShowPassphrase] = useState(false)
  const [passphraseError, setPassphraseError] = useState<string | undefined>()

  useEffect(() => {
    setService(vaultService)
  }, [setService])

  useEffect(() => {
    if (status === 'authenticated' && syncStatus === 'idle') {
      setShowPassphrase(true)
    }
  }, [status, syncStatus])

  const handlePassphraseSubmit = async (passphrase: string) => {
    if (!token || !user) return
    try {
      useVaultStore.getState().setCredentials(token, passphrase)
      await load(token, passphrase)
      setShowPassphrase(false)
    } catch (e) {
      setPassphraseError(e instanceof Error ? e.message : 'Failed to unlock vault')
    }
  }

  return (
    <SessionGuard
      fallback={
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: '100vh',
          }}
        >
          <GitHubLoginButton
            clientId={import.meta.env.VITE_GITHUB_CLIENT_ID ?? ''}
            redirectUri={import.meta.env.VITE_OAUTH_REDIRECT_URI ?? ''}
          />
        </div>
      }
    >
      <VaultPassphraseModal
        open={showPassphrase}
        mode="unlock"
        onSubmit={(p) => {
          void handlePassphraseSubmit(p)
        }}
        error={passphraseError}
      />
      <div style={{ padding: '2rem' }}>
        <p>Welcome, {user?.login ?? 'user'}!</p>
        <p>Vault status: {syncStatus}</p>
      </div>
    </SessionGuard>
  )
}
