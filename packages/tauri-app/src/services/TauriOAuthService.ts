import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { start, cancel } from '@fabianlars/tauri-plugin-oauth'
import type { GitHubUser } from '@tacoshell/ui/stores'

export interface DesktopOAuthResult {
  token: string
  user: GitHubUser
}

function generateRandom(length: number): string {
  // Use ceil(length/2) bytes so that hex encoding yields exactly `length` hex chars,
  // preserving the full entropy of every byte.
  const array = new Uint8Array(Math.ceil(length / 2))
  crypto.getRandomValues(array)
  return Array.from(array, (byte) => byte.toString(16).padStart(2, '0'))
    .join('')
    .slice(0, length)
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(digest)
  return btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '')
}

/**
 * Performs the complete desktop GitHub OAuth PKCE flow:
 * 1. Generates a PKCE code_verifier and code_challenge.
 * 2. Starts a local HTTP redirect server via tauri-plugin-oauth.
 * 3. Opens the GitHub OAuth authorize URL in the system browser.
 * 4. Waits for the OAuth callback via the `oauth://url` plugin event.
 * 5. Validates the state parameter to prevent CSRF.
 * 6. Exchanges the authorization code for an access token (PKCE, no client_secret).
 * 7. Fetches the GitHub user profile.
 * Returns the token and user profile on success.
 */
export async function startDesktopOAuth(clientId: string): Promise<DesktopOAuthResult> {
  const state = generateRandom(32)
  const codeVerifier = generateRandom(64)
  const codeChallenge = await generateCodeChallenge(codeVerifier)

  const port = await start()
  const redirectUri = `http://localhost:${port}`

  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: 'repo read:user',
    state,
    code_challenge: codeChallenge,
    code_challenge_method: 'S256',
  })
  const authorizeUrl = `https://github.com/login/oauth/authorize?${params.toString()}`

  await invoke('open_url', { url: authorizeUrl })

  return new Promise<DesktopOAuthResult>((resolve, reject) => {
    let unlistenFn: (() => void) | undefined = undefined

    const cleanup = async (): Promise<void> => {
      unlistenFn?.()
      unlistenFn = undefined
      await cancel(port).catch(() => undefined)
    }

    // Register the event listener. The returned promise resolves to an unlisten function.
    // The handler is wrapped to satisfy the sync-void signature expected by listen().
    const listenerPromise = listen<string>('oauth://url', (event) => {
      void (async () => {
        await cleanup()

        try {
          const callbackUrl = new URL(event.payload)
          const code = callbackUrl.searchParams.get('code')
          const callbackState = callbackUrl.searchParams.get('state')

          if (!code) {
            reject(new Error('OAuth callback is missing the code parameter'))
            return
          }
          if (callbackState !== state) {
            reject(new Error('OAuth state mismatch: possible CSRF attack'))
            return
          }

          const token = await invoke<string>('exchange_oauth_code', {
            clientId,
            code,
            redirectUri,
            codeVerifier,
          })

          const profile = await invoke<{
            login: string
            name: string | null
            avatar_url: string
          }>('get_user_profile', { token })

          resolve({
            token,
            user: { login: profile.login, name: profile.name, avatarUrl: profile.avatar_url },
          })
        } catch (e) {
          reject(e instanceof Error ? e : new Error(String(e)))
        }
      })()
    })

    listenerPromise
      .then((fn) => {
        unlistenFn = fn
      })
      .catch((err: unknown) => {
        reject(err instanceof Error ? err : new Error(String(err)))
      })
  })
}
