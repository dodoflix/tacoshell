import { useAuthStore } from '../useAuthStore'

beforeEach(() => {
  useAuthStore.setState({
    status: 'unauthenticated',
    token: null,
    user: null,
    error: null,
  })
})

describe('useAuthStore', () => {
  it('initial state is unauthenticated with null token/user/error', () => {
    const state = useAuthStore.getState()
    expect(state.status).toBe('unauthenticated')
    expect(state.token).toBeNull()
    expect(state.user).toBeNull()
    expect(state.error).toBeNull()
  })

  it('setToken sets token and status to authenticated', () => {
    useAuthStore.getState().setToken('gho_test_token')
    const state = useAuthStore.getState()
    expect(state.token).toBe('gho_test_token')
    expect(state.status).toBe('authenticated')
  })

  it('setUser sets user fields', () => {
    const user = {
      login: 'testuser',
      name: 'Test User',
      avatarUrl: 'https://example.com/avatar.png',
    }
    useAuthStore.getState().setUser(user)
    const state = useAuthStore.getState()
    expect(state.user).toEqual(user)
  })

  it('setStatus updates status', () => {
    useAuthStore.getState().setStatus('authenticating')
    expect(useAuthStore.getState().status).toBe('authenticating')
  })

  it('setError sets error and status to error', () => {
    useAuthStore.getState().setError('Authentication failed')
    const state = useAuthStore.getState()
    expect(state.error).toBe('Authentication failed')
    expect(state.status).toBe('error')
  })

  it('logout resets all state to initial values', () => {
    useAuthStore.getState().setToken('gho_test_token')
    useAuthStore
      .getState()
      .setUser({ login: 'user', name: 'User', avatarUrl: 'https://example.com/a.png' })
    useAuthStore.getState().logout()
    const state = useAuthStore.getState()
    expect(state.status).toBe('unauthenticated')
    expect(state.token).toBeNull()
    expect(state.user).toBeNull()
    expect(state.error).toBeNull()
  })

  it('after setToken then logout, state returns to initial', () => {
    useAuthStore.getState().setToken('gho_test_token')
    expect(useAuthStore.getState().status).toBe('authenticated')
    useAuthStore.getState().logout()
    expect(useAuthStore.getState().status).toBe('unauthenticated')
    expect(useAuthStore.getState().token).toBeNull()
  })
})
