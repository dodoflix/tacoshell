import React from 'react'
import { cn } from '../../lib/cn'

interface SidebarProps {
  children: React.ReactNode
  className?: string
  header?: React.ReactNode
  footer?: React.ReactNode
}

export function Sidebar({ children, className, header, footer }: SidebarProps) {
  return (
    <aside className={cn('flex h-full flex-col border-r border-gray-200 bg-gray-50', className)}>
      {header && <div className="flex-shrink-0 border-b border-gray-200 px-3 py-2">{header}</div>}
      <div className="flex-1 overflow-y-auto p-2">{children}</div>
      {footer && <div className="flex-shrink-0 border-t border-gray-200 px-3 py-2">{footer}</div>}
    </aside>
  )
}
