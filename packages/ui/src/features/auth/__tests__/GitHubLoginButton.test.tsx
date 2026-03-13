import { vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { GitHubLoginButton } from '../GitHubLoginButton'

const mockStartLogin = vi.fn()

vi.mock('../useGitHubOAuth', () => ({
  useGitHubOAuth: vi.fn(() => ({
    startLogin: mockStartLogin,
    exchangeCode: vi.fn(),
    isExchanging: false,
  })),
}))

describe('GitHubLoginButton', () => {
  beforeEach(() => {
    mockStartLogin.mockReset()
  })

  it('renders "Sign in with GitHub" text', () => {
    render(
      <GitHubLoginButton clientId="test_client_id" redirectUri="http://localhost:3000/callback" />,
    )
    expect(screen.getByText(/sign in with github/i)).toBeInTheDocument()
  })

  it('clicking the button calls startLogin', async () => {
    const user = userEvent.setup()
    render(
      <GitHubLoginButton clientId="test_client_id" redirectUri="http://localhost:3000/callback" />,
    )
    await user.click(screen.getByRole('button'))
    expect(mockStartLogin).toHaveBeenCalledTimes(1)
  })

  it('shows disabled/loading state when isExchanging', async () => {
    const { useGitHubOAuth } = await import('../useGitHubOAuth')
    vi.mocked(useGitHubOAuth).mockReturnValueOnce({
      startLogin: mockStartLogin,
      exchangeCode: vi.fn(),
      isExchanging: true,
    })
    render(
      <GitHubLoginButton clientId="test_client_id" redirectUri="http://localhost:3000/callback" />,
    )
    const button = screen.getByRole('button')
    expect(button).toBeDisabled()
  })
})
