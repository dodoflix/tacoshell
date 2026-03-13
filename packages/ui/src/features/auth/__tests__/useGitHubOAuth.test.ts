import { vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useGitHubOAuth } from '../useGitHubOAuth'

function makeFetchResponse(body: unknown, ok = true, status = 200): Response {
  return {
    ok,
    status,
    json: () => Promise.resolve(body),
  } as unknown as Response
}

describe('useGitHubOAuth', () => {
  beforeEach(() => {
    sessionStorage.clear()
    Object.defineProperty(window, 'location', {
      value: { href: '', assign: vi.fn() },
      writable: true,
    })
  })

  it('startLogin sets sessionStorage oauth_state and oauth_code_verifier', async () => {
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    await act(async () => {
      result.current.startLogin()
      await new Promise((resolve) => setTimeout(resolve, 50))
    })
    expect(sessionStorage.getItem('oauth_state')).not.toBeNull()
    expect(sessionStorage.getItem('oauth_code_verifier')).not.toBeNull()
  })

  it('startLogin navigates to GitHub OAuth URL', async () => {
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    await act(async () => {
      result.current.startLogin()
      await new Promise((resolve) => setTimeout(resolve, 50))
    })
    expect(window.location.href).toContain('github.com/login/oauth/authorize')
  })

  it('exchangeCode with mismatched state calls onError', async () => {
    sessionStorage.setItem('oauth_state', 'correct_state')
    sessionStorage.setItem('oauth_code_verifier', 'test_verifier')
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    await act(async () => {
      await result.current.exchangeCode('test_code', 'wrong_state')
    })
    expect(onError).toHaveBeenCalledWith('State mismatch: possible CSRF attack')
    expect(onSuccess).not.toHaveBeenCalled()
  })

  it('exchangeCode with valid code+state calls token endpoint and calls onSuccess', async () => {
    sessionStorage.setItem('oauth_state', 'valid_state')
    sessionStorage.setItem('oauth_code_verifier', 'test_verifier')
    const fetchSpy = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(makeFetchResponse({ access_token: 'gho_test_token' }))
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    await act(async () => {
      await result.current.exchangeCode('test_code', 'valid_state')
    })
    expect(onSuccess).toHaveBeenCalledWith('gho_test_token')
    expect(onError).not.toHaveBeenCalled()
    fetchSpy.mockRestore()
  })

  it('exchangeCode when server returns error calls onError', async () => {
    sessionStorage.setItem('oauth_state', 'valid_state')
    sessionStorage.setItem('oauth_code_verifier', 'test_verifier')
    const fetchSpy = vi
      .spyOn(globalThis, 'fetch')
      .mockResolvedValueOnce(makeFetchResponse({ error: 'bad_verification_code' }, false, 400))
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    await act(async () => {
      await result.current.exchangeCode('test_code', 'valid_state')
    })
    expect(onError).toHaveBeenCalled()
    expect(onSuccess).not.toHaveBeenCalled()
    fetchSpy.mockRestore()
  })

  it('isExchanging is true during exchange and false after', async () => {
    sessionStorage.setItem('oauth_state', 'valid_state')
    sessionStorage.setItem('oauth_code_verifier', 'test_verifier')
    let resolveRequest!: (value: Response) => void
    const fetchSpy = vi.spyOn(globalThis, 'fetch').mockReturnValueOnce(
      new Promise<Response>((resolve) => {
        resolveRequest = resolve
      }),
    )
    const onSuccess = vi.fn()
    const onError = vi.fn()
    const { result } = renderHook(() =>
      useGitHubOAuth({
        clientId: 'test_client_id',
        redirectUri: 'http://localhost:3000/callback',
        onSuccess,
        onError,
      }),
    )
    expect(result.current.isExchanging).toBe(false)

    act(() => {
      void result.current.exchangeCode('test_code', 'valid_state')
    })
    await waitFor(() => expect(result.current.isExchanging).toBe(true))

    await act(async () => {
      resolveRequest(makeFetchResponse({ access_token: 'tok' }))
      await new Promise((resolve) => setTimeout(resolve, 10))
    })
    expect(result.current.isExchanging).toBe(false)
    fetchSpy.mockRestore()
  })
})
