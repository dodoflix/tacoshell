import { vi, describe, it, expect, beforeEach } from 'vitest'
import type { Event } from '@tauri-apps/api/event'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

vi.mock('@fabianlars/tauri-plugin-oauth', () => ({
  start: vi.fn(),
  cancel: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { start, cancel } from '@fabianlars/tauri-plugin-oauth'
import { startDesktopOAuth } from '../TauriOAuthService'

const mockInvoke = vi.mocked(invoke)
const mockStart = vi.mocked(start)
const mockCancel = vi.mocked(cancel)

const MOCK_PORT = 12345
const MOCK_TOKEN = 'gho_test_token'
const MOCK_PROFILE = { login: 'testuser', name: 'Test User', avatar_url: 'https://avatars.test/1' }

/** Returns the state parameter from the open_url invoke call. */
function getStateFromOpenUrl(): string {
  const call = mockInvoke.mock.calls.find((c) => c[0] === 'open_url')
  const url = (call?.[1] as { url?: string })?.url ?? ''
  try {
    return new URL(url).searchParams.get('state') ?? ''
  } catch {
    return ''
  }
}

/**
 * Sets up the listen mock so it captures the event handler for test use.
 * Returns a promise that resolves with the captured handler once listen() is called.
 */
function setupListenMock(): Promise<(event: { payload: string }) => Promise<void>> {
  return new Promise((resolveCapture) => {
    vi.mocked(listen).mockImplementation(
      async (
        _event: string,
        handler: (event: Event<string>) => void | Promise<void>,
      ): Promise<() => void> => {
        resolveCapture(handler as (event: { payload: string }) => Promise<void>)
        return () => undefined
      },
    )
  })
}

describe('startDesktopOAuth', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockStart.mockResolvedValue(MOCK_PORT)
    mockCancel.mockResolvedValue(undefined)
  })

  it('starts the oauth local server', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockResolvedValueOnce(MOCK_TOKEN) // exchange_oauth_code
      .mockResolvedValueOnce(MOCK_PROFILE) // get_user_profile

    const promise = startDesktopOAuth('test-client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=test-code&state=${state}` })
    await promise

    expect(mockStart).toHaveBeenCalledOnce()
  })

  it('opens the github authorize url in the system browser', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockResolvedValueOnce(MOCK_TOKEN)
      .mockResolvedValueOnce(MOCK_PROFILE)

    const promise = startDesktopOAuth('my-client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    const openUrlCall = mockInvoke.mock.calls.find((c) => c[0] === 'open_url')
    expect(openUrlCall).toBeDefined()
    const url = (openUrlCall![1] as { url: string }).url
    expect(url).toContain('https://github.com/login/oauth/authorize')
    expect(url).toContain('client_id=my-client-id')
    expect(url).toContain('code_challenge_method=S256')
    expect(url).toContain(`redirect_uri=http%3A%2F%2Flocalhost%3A${MOCK_PORT}`)

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=abc&state=${state}` })
    await promise
  })

  it('calls exchange_oauth_code with client_id, code, redirectUri, and codeVerifier', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockResolvedValueOnce(MOCK_TOKEN) // exchange_oauth_code
      .mockResolvedValueOnce(MOCK_PROFILE) // get_user_profile

    const promise = startDesktopOAuth('cli-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=the-code&state=${state}` })
    await promise

    const exchangeCall = mockInvoke.mock.calls.find((c) => c[0] === 'exchange_oauth_code')
    expect(exchangeCall).toBeDefined()
    const args = exchangeCall![1] as {
      clientId: string
      code: string
      redirectUri: string
      codeVerifier: string
    }
    expect(args.clientId).toBe('cli-id')
    expect(args.code).toBe('the-code')
    expect(args.redirectUri).toBe(`http://localhost:${MOCK_PORT}`)
    expect(typeof args.codeVerifier).toBe('string')
    expect(args.codeVerifier.length).toBeGreaterThan(0)
  })

  it('resolves with token and mapped user profile on success', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockResolvedValueOnce(MOCK_TOKEN)
      .mockResolvedValueOnce(MOCK_PROFILE)

    const promise = startDesktopOAuth('client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=abc&state=${state}` })
    const result = await promise

    expect(result.token).toBe(MOCK_TOKEN)
    expect(result.user).toEqual({
      login: 'testuser',
      name: 'Test User',
      avatarUrl: 'https://avatars.test/1',
    })
  })

  it('rejects when OAuth callback state does not match (CSRF protection)', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke.mockResolvedValueOnce(undefined) // open_url

    const promise = startDesktopOAuth('client-id')
    const handler = await handlerPromise

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=abc&state=wrong-state` })

    await expect(promise).rejects.toThrow(/state mismatch/i)
  })

  it('rejects when OAuth callback is missing the code parameter', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke.mockResolvedValueOnce(undefined) // open_url

    const promise = startDesktopOAuth('client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?state=${state}` })

    await expect(promise).rejects.toThrow(/missing the code parameter/i)
  })

  it('cancels the oauth server after a successful callback', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockResolvedValueOnce(MOCK_TOKEN)
      .mockResolvedValueOnce(MOCK_PROFILE)

    const promise = startDesktopOAuth('client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=abc&state=${state}` })
    await promise

    expect(mockCancel).toHaveBeenCalledWith(MOCK_PORT)
  })

  it('cancels the oauth server even when exchange_oauth_code rejects', async () => {
    const handlerPromise = setupListenMock()
    mockInvoke
      .mockResolvedValueOnce(undefined) // open_url
      .mockRejectedValueOnce(new Error('token exchange failed')) // exchange_oauth_code

    const promise = startDesktopOAuth('client-id')
    const handler = await handlerPromise
    const state = getStateFromOpenUrl()

    await handler({ payload: `http://localhost:${MOCK_PORT}?code=abc&state=${state}` })

    await expect(promise).rejects.toThrow('token exchange failed')
    expect(mockCancel).toHaveBeenCalledWith(MOCK_PORT)
  })
})
