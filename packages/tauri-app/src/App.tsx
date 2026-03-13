import { useState, useEffect } from 'react'
import { SessionGuard, VaultPassphraseModal } from '@tacoshell/ui/features/auth'
import { Button } from '@tacoshell/ui/components'
import { useAuthStore, useVaultStore } from '@tacoshell/ui/stores'
import { TauriVaultService } from './services/TauriVaultService'
import { startDesktopOAuth } from './services/TauriOAuthService'

const vaultService = new TauriVaultService()

export default function App() {
  const {
    status,
    token,
    user,
    setToken,
    setUser,
    setStatus,
    setError: setAuthError,
  } = useAuthStore()
  const { setService, load, syncStatus, error: vaultError } = useVaultStore()
  const [showPassphrase, setShowPassphrase] = useState(false)
  const [passphraseError, setPassphraseError] = useState<string | undefined>()
  const [oauthLoading, setOauthLoading] = useState(false)

  useEffect(() => {
    setService(vaultService)
  }, [setService])

  useEffect(() => {
    if (status === 'authenticated' && syncStatus === 'idle') {
      setShowPassphrase(true)
    }
  }, [status, syncStatus])

  useEffect(() => {
    if (syncStatus === 'synced') {
      setShowPassphrase(false)
      setPassphraseError(undefined)
    }
  }, [syncStatus])

  const handleLogin = async (): Promise<void> => {
    setOauthLoading(true)
    setStatus('authenticating')
    try {
      const clientId = import.meta.env.VITE_GITHUB_CLIENT_ID ?? ''
      const result = await startDesktopOAuth(clientId)
      setToken(result.token)
      setUser(result.user)
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Login failed'
      setAuthError(message)
    } finally {
      setOauthLoading(false)
    }
  }

  const handlePassphraseSubmit = async (passphrase: string): Promise<void> => {
    if (!token || !user) return
    setPassphraseError(undefined)
    try {
      await load(token, passphrase)
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
          <Button
            variant="secondary"
            onClick={() => {
              void handleLogin()
            }}
            loading={oauthLoading}
            disabled={oauthLoading}
          >
            Sign in with GitHub
          </Button>
        </div>
      }
    >
      <VaultPassphraseModal
        open={showPassphrase}
        mode="unlock"
        onSubmit={(p) => {
          void handlePassphraseSubmit(p)
        }}
        {...(passphraseError != null
          ? { error: passphraseError }
          : vaultError != null
            ? { error: vaultError }
            : {})}
      />
      <div style={{ padding: '2rem' }}>
        <p>Welcome, {user?.login ?? 'user'}!</p>
        <p>Vault status: {syncStatus}</p>
      </div>
    </SessionGuard>
  )
}
