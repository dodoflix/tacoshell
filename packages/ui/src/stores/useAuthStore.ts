import { create } from 'zustand'

export type AuthStatus = 'unauthenticated' | 'authenticating' | 'authenticated' | 'error'

export interface GitHubUser {
  login: string
  name: string | null
  avatarUrl: string
}

export interface AuthState {
  status: AuthStatus
  token: string | null
  user: GitHubUser | null
  error: string | null
  setToken: (token: string) => void
  setUser: (user: GitHubUser) => void
  setStatus: (status: AuthStatus) => void
  setError: (error: string) => void
  logout: () => void
}

export const useAuthStore = create<AuthState>()((set) => ({
  status: 'unauthenticated',
  token: null,
  user: null,
  error: null,

  setToken: (token: string) => set({ token, status: 'authenticated' }),

  setUser: (user: GitHubUser) => set({ user }),

  setStatus: (status: AuthStatus) => set({ status }),

  setError: (error: string) => set({ error, status: 'error' }),

  logout: () => set({ status: 'unauthenticated', token: null, user: null, error: null }),
}))
