import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { vi } from 'vitest'
import { VaultPassphraseModal } from '../VaultPassphraseModal'

describe('VaultPassphraseModal', () => {
  it('renders "Create Vault Passphrase" title in create mode', () => {
    render(<VaultPassphraseModal open={true} mode="create" onSubmit={vi.fn()} />)
    expect(screen.getByText('Create Vault Passphrase')).toBeInTheDocument()
  })

  it('renders "Unlock Vault" title in unlock mode', () => {
    render(<VaultPassphraseModal open={true} mode="unlock" onSubmit={vi.fn()} />)
    expect(screen.getByText('Unlock Vault')).toBeInTheDocument()
  })

  it('unlock mode: submits passphrase when form submitted', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<VaultPassphraseModal open={true} mode="unlock" onSubmit={onSubmit} />)
    const input = screen.getByLabelText(/passphrase/i)
    await user.type(input, 'my-secret-passphrase')
    await user.click(screen.getByRole('button', { name: /unlock/i }))
    expect(onSubmit).toHaveBeenCalledWith('my-secret-passphrase')
  })

  it('create mode: shows error when passphrases do not match', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<VaultPassphraseModal open={true} mode="create" onSubmit={onSubmit} />)
    await user.type(screen.getByLabelText(/^passphrase/i), 'passphrase1')
    await user.type(screen.getByLabelText(/confirm/i), 'passphrase2')
    await user.click(screen.getByRole('button', { name: /create/i }))
    expect(screen.getByText(/do not match/i)).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('create mode: calls onSubmit with passphrase when passphrases match', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<VaultPassphraseModal open={true} mode="create" onSubmit={onSubmit} />)
    await user.type(screen.getByLabelText(/^passphrase/i), 'my-passphrase')
    await user.type(screen.getByLabelText(/confirm/i), 'my-passphrase')
    await user.click(screen.getByRole('button', { name: /create/i }))
    expect(onSubmit).toHaveBeenCalledWith('my-passphrase')
  })

  it('shows error message when error prop is provided', () => {
    render(
      <VaultPassphraseModal
        open={true}
        mode="unlock"
        onSubmit={vi.fn()}
        error="Invalid passphrase"
      />,
    )
    expect(screen.getByText('Invalid passphrase')).toBeInTheDocument()
  })

  it('shows loading state on submit button when isLoading=true', () => {
    render(<VaultPassphraseModal open={true} mode="unlock" onSubmit={vi.fn()} isLoading={true} />)
    const button = screen.getByRole('button', { name: /unlock/i })
    expect(button).toBeDisabled()
  })

  it('calls onCancel when cancel button clicked', async () => {
    const user = userEvent.setup()
    const onCancel = vi.fn()
    render(
      <VaultPassphraseModal open={true} mode="unlock" onSubmit={vi.fn()} onCancel={onCancel} />,
    )
    await user.click(screen.getByRole('button', { name: /cancel/i }))
    expect(onCancel).toHaveBeenCalled()
  })
})
