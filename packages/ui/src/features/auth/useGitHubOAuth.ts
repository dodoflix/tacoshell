import { useState } from 'react'
import { generateRandom, generateCodeChallenge } from '../../lib/crypto'

interface UseGitHubOAuthOptions {
  clientId: string
  redirectUri: string
  tokenEndpoint?: string
  onSuccess: (token: string) => void
  onError: (error: string) => void
}

interface UseGitHubOAuthReturn {
  startLogin: () => void
  exchangeCode: (code: string, state: string) => Promise<void>
  isExchanging: boolean
}

export function useGitHubOAuth({
  clientId,
  redirectUri,
  tokenEndpoint = '/api/oauth/callback',
  onSuccess,
  onError,
}: UseGitHubOAuthOptions): UseGitHubOAuthReturn {
  const [isExchanging, setIsExchanging] = useState(false)

  const startLogin = (): void => {
    const state = generateRandom(32)
    const codeVerifier = generateRandom(64)
    sessionStorage.setItem('oauth_state', state)
    sessionStorage.setItem('oauth_code_verifier', codeVerifier)

    void generateCodeChallenge(codeVerifier).then((codeChallenge) => {
      const params = new URLSearchParams({
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: 'repo read:user',
        state,
        code_challenge: codeChallenge,
        code_challenge_method: 'S256',
      })
      window.location.href = `https://github.com/login/oauth/authorize?${params.toString()}`
    })
  }

  const exchangeCode = async (code: string, state: string): Promise<void> => {
    const storedState = sessionStorage.getItem('oauth_state')
    if (state !== storedState) {
      onError('State mismatch: possible CSRF attack')
      return
    }

    const codeVerifier = sessionStorage.getItem('oauth_code_verifier')

    setIsExchanging(true)
    try {
      const response = await fetch(tokenEndpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, code_verifier: codeVerifier, redirect_uri: redirectUri }),
      })

      if (!response.ok) {
        const body = (await response.json()) as Record<string, unknown>
        onError(typeof body['error'] === 'string' ? body['error'] : 'Token exchange failed')
        return
      }

      const body = (await response.json()) as Record<string, unknown>
      const token = body['access_token']
      if (typeof token !== 'string') {
        onError('Invalid token response')
        return
      }

      sessionStorage.removeItem('oauth_state')
      sessionStorage.removeItem('oauth_code_verifier')
      onSuccess(token)
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Network error')
    } finally {
      setIsExchanging(false)
    }
  }

  return { startLogin, exchangeCode, isExchanging }
}
