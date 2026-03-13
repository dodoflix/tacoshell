import React from 'react'
import { cn } from '@/lib/cn'

interface PanelProps {
  title?: string
  children: React.ReactNode
  className?: string
  actions?: React.ReactNode
}

export function Panel({ title, children, className, actions }: PanelProps) {
  return (
    <div
      className={cn(
        'flex flex-col rounded-lg border border-gray-200 bg-white shadow-sm',
        className,
      )}
    >
      {(title || actions) && (
        <div className="flex items-center justify-between border-b border-gray-200 px-4 py-3">
          {title && <h3 className="text-sm font-semibold text-gray-900">{title}</h3>}
          {actions && <div className="flex items-center gap-2">{actions}</div>}
        </div>
      )}
      <div className="flex-1 p-4">{children}</div>
    </div>
  )
}
