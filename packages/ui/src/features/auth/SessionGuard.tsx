import React from 'react'
import { useAuthStore } from '@/stores/useAuthStore'

interface SessionGuardProps {
  children: React.ReactNode
  fallback?: React.ReactNode
}

export function SessionGuard({ children, fallback = null }: SessionGuardProps): React.ReactElement {
  const status = useAuthStore((state) => state.status)

  if (status === 'authenticated') {
    return <>{children}</>
  }

  return <>{fallback}</>
}
