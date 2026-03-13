import { render, screen, act } from '@testing-library/react'
import { useAuthStore } from '../../../stores/useAuthStore'
import { SessionGuard } from '../SessionGuard'

beforeEach(() => {
  useAuthStore.setState({ status: 'unauthenticated', token: null, user: null, error: null })
})

describe('SessionGuard', () => {
  it('renders children when authenticated', () => {
    useAuthStore.setState({ status: 'authenticated', token: 'tok', user: null, error: null })
    render(
      <SessionGuard>
        <div>Protected Content</div>
      </SessionGuard>,
    )
    expect(screen.getByText('Protected Content')).toBeInTheDocument()
  })

  it('does not render children when unauthenticated', () => {
    render(
      <SessionGuard>
        <div>Protected Content</div>
      </SessionGuard>,
    )
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument()
  })

  it('renders fallback when unauthenticated and fallback is provided', () => {
    render(
      <SessionGuard fallback={<div>Please Login</div>}>
        <div>Protected Content</div>
      </SessionGuard>,
    )
    expect(screen.getByText('Please Login')).toBeInTheDocument()
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument()
  })

  it('renders children after store transitions from unauthenticated to authenticated', () => {
    render(
      <SessionGuard>
        <div>Protected Content</div>
      </SessionGuard>,
    )
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument()

    act(() => {
      useAuthStore.setState({ status: 'authenticated', token: 'tok', user: null, error: null })
    })

    expect(screen.getByText('Protected Content')).toBeInTheDocument()
  })
})
