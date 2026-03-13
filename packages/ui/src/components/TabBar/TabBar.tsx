import React from 'react'
import { cn } from '../../lib/cn'

interface Tab {
  id: string
  label: string
  closable?: boolean
}

interface TabBarProps {
  tabs: Tab[]
  activeTabId: string
  onTabClick: (id: string) => void
  onTabClose?: (id: string) => void
  className?: string
}

export function TabBar({ tabs, activeTabId, onTabClick, onTabClose, className }: TabBarProps) {
  return (
    <div
      role="tablist"
      className={cn(
        'flex items-stretch overflow-x-auto border-b border-gray-200 bg-gray-50',
        className,
      )}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId
        return (
          <div
            key={tab.id}
            role="tab"
            aria-selected={isActive}
            data-active={isActive ? '' : undefined}
            className={cn(
              'relative flex min-w-0 cursor-pointer items-center gap-1 px-4 py-2 text-sm',
              'border-b-2 transition-colors',
              isActive
                ? 'border-blue-500 bg-white text-blue-600'
                : 'border-transparent text-gray-600 hover:bg-gray-100 hover:text-gray-900',
            )}
            onClick={() => onTabClick(tab.id)}
          >
            <span className="truncate">{tab.label}</span>
            {tab.closable && (
              <button
                type="button"
                aria-label={`Close ${tab.label}`}
                className="ml-1 flex h-4 w-4 items-center justify-center rounded hover:bg-gray-200"
                onClick={(e) => {
                  e.stopPropagation()
                  onTabClose?.(tab.id)
                }}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-3 w-3"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            )}
          </div>
        )
      })}
    </div>
  )
}
