import React, { useState } from 'react'
import { cn } from '../../lib/cn'

export interface TreeNode {
  id: string
  label: string
  icon?: React.ReactNode
  children?: TreeNode[]
}

interface TreeViewProps {
  nodes: TreeNode[]
  onNodeClick?: ((node: TreeNode) => void) | undefined
  className?: string
}

interface TreeNodeItemProps {
  node: TreeNode
  depth: number
  expandedIds: Set<string>
  onToggle: (id: string) => void
  onNodeClick?: ((node: TreeNode) => void) | undefined
}

function TreeNodeItem({ node, depth, expandedIds, onToggle, onNodeClick }: TreeNodeItemProps) {
  const hasChildren = node.children && node.children.length > 0
  const isExpanded = expandedIds.has(node.id)

  const handleClick = () => {
    if (hasChildren) {
      onToggle(node.id)
    } else {
      onNodeClick?.(node)
    }
  }

  return (
    <li>
      <button
        type="button"
        onClick={handleClick}
        className={cn(
          'flex w-full items-center gap-1 rounded px-2 py-1 text-left text-sm',
          'hover:bg-gray-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500',
        )}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
      >
        {hasChildren && (
          <span
            aria-label={isExpanded ? 'Collapse' : 'Expand'}
            className="flex h-4 w-4 flex-shrink-0 items-center justify-center text-gray-400"
          >
            {isExpanded ? (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-3 w-3"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path strokeLinecap="round" strokeLinejoin="round" d="M19 9l-7 7-7-7" />
              </svg>
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-3 w-3"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                strokeWidth={2}
              >
                <path strokeLinecap="round" strokeLinejoin="round" d="M9 5l7 7-7 7" />
              </svg>
            )}
          </span>
        )}
        {node.icon && <span className="flex-shrink-0">{node.icon}</span>}
        <span className="truncate">{node.label}</span>
      </button>
      {hasChildren && isExpanded && (
        <ul>
          {node.children!.map((child) => (
            <TreeNodeItem
              key={child.id}
              node={child}
              depth={depth + 1}
              expandedIds={expandedIds}
              onToggle={onToggle}
              onNodeClick={onNodeClick}
            />
          ))}
        </ul>
      )}
    </li>
  )
}

export function TreeView({ nodes, onNodeClick, className }: TreeViewProps) {
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set())

  const handleToggle = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  return (
    <ul role="tree" className={cn('select-none', className)}>
      {nodes.map((node) => (
        <TreeNodeItem
          key={node.id}
          node={node}
          depth={0}
          expandedIds={expandedIds}
          onToggle={handleToggle}
          onNodeClick={onNodeClick}
        />
      ))}
    </ul>
  )
}
