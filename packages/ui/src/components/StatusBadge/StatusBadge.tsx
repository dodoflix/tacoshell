import { cn } from '@/lib/cn'

type StatusBadgeStatus = 'connected' | 'disconnected' | 'loading' | 'error'

interface StatusBadgeProps {
  status: StatusBadgeStatus
  label?: string
  className?: string
}

const statusConfig: Record<StatusBadgeStatus, { dotClass: string; ariaLabel: string }> = {
  connected: {
    dotClass: 'bg-green-500 connected',
    ariaLabel: 'Connected',
  },
  disconnected: {
    dotClass: 'bg-gray-400 disconnected',
    ariaLabel: 'Disconnected',
  },
  loading: {
    dotClass: 'bg-blue-400 animate-pulse loading',
    ariaLabel: 'Connecting',
  },
  error: {
    dotClass: 'bg-red-500 error',
    ariaLabel: 'Error',
  },
}

export function StatusBadge({ status, label, className }: StatusBadgeProps) {
  const { dotClass, ariaLabel } = statusConfig[status]

  return (
    <span
      role="status"
      aria-label={label ? `${ariaLabel}: ${label}` : ariaLabel}
      className={cn('inline-flex items-center gap-1.5', className)}
    >
      <span className={cn('h-2 w-2 rounded-full', dotClass)} />
      {label && <span className="text-sm text-gray-700">{label}</span>}
    </span>
  )
}
