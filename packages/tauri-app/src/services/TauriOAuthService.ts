import { useAuthStore } from '@tacoshell/ui/stores'

/**
 * Completes the desktop OAuth flow using the Tauri OAuth plugin.
 * Called after the user is redirected back from GitHub.
 * The plugin handles the local redirect server; we just exchange the code.
 */
export async function handleDesktopOAuthCallback(code: string): Promise<void> {
  const { setToken, setError, setStatus } = useAuthStore.getState()
  setStatus('authenticating')

  try {
    // On desktop, exchange happens via a backend endpoint or directly
    // The token endpoint receives the code and returns the access token
    const resp = await fetch('/api/oauth/callback', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code }),
    })
    if (!resp.ok) throw new Error(`Token exchange failed: ${resp.status}`)
    const data = (await resp.json()) as { access_token: string }
    setToken(data.access_token)
  } catch (e) {
    setError(e instanceof Error ? e.message : 'OAuth failed')
  }
}
