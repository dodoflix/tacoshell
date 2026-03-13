import React, { useState } from 'react'
import { Modal } from '../../components/Modal'
import { Input } from '../../components/Input'
import { Button } from '../../components/Button'

export type VaultPassphraseMode = 'create' | 'unlock'

interface VaultPassphraseModalProps {
  open: boolean
  mode: VaultPassphraseMode
  onSubmit: (passphrase: string) => void
  onCancel?: () => void
  isLoading?: boolean
  error?: string
}

export function VaultPassphraseModal({
  open,
  mode,
  onSubmit,
  onCancel,
  isLoading = false,
  error,
}: VaultPassphraseModalProps): React.ReactElement {
  const [passphrase, setPassphrase] = useState('')
  const [confirm, setConfirm] = useState('')
  const [mismatchError, setMismatchError] = useState<string | undefined>(undefined)

  const title = mode === 'create' ? 'Create Vault Passphrase' : 'Unlock Vault'
  const submitLabel = mode === 'create' ? 'Create' : 'Unlock'

  const handleSubmit = (e: React.FormEvent): void => {
    e.preventDefault()
    if (mode === 'create') {
      if (passphrase !== confirm) {
        setMismatchError('Passphrases do not match')
        return
      }
      setMismatchError(undefined)
    }
    onSubmit(passphrase)
  }

  const handleClose = (): void => {
    if (onCancel) onCancel()
  }

  return (
    <Modal open={open} onClose={handleClose} title={title}>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <Input
          variant="password"
          label="Passphrase"
          id="vault-passphrase"
          value={passphrase}
          onChange={(e) => setPassphrase(e.target.value)}
          disabled={isLoading}
        />
        {mode === 'create' && (
          <Input
            variant="password"
            label="Confirm Passphrase"
            id="vault-passphrase-confirm"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            disabled={isLoading}
          />
        )}
        {(mismatchError ?? error) && (
          <p className="text-sm text-red-600" role="alert">
            {mismatchError ?? error}
          </p>
        )}
        <div className="flex justify-end gap-2">
          {onCancel && (
            <Button type="button" variant="ghost" onClick={onCancel} disabled={isLoading}>
              Cancel
            </Button>
          )}
          <Button type="submit" variant="primary" loading={isLoading} disabled={isLoading}>
            {submitLabel}
          </Button>
        </div>
      </form>
    </Modal>
  )
}
