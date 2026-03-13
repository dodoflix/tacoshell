import React, { useCallback, useRef, useState } from 'react'
import { cn } from '../../lib/cn'

interface SplitPaneProps {
  direction?: 'horizontal' | 'vertical'
  initialRatio?: number
  minRatio?: number
  maxRatio?: number
  first: React.ReactNode
  second: React.ReactNode
  className?: string
}

export function SplitPane({
  direction = 'horizontal',
  initialRatio = 0.5,
  minRatio = 0.1,
  maxRatio = 0.9,
  first,
  second,
  className,
}: SplitPaneProps) {
  const [ratio, setRatio] = useState(initialRatio)
  const containerRef = useRef<HTMLDivElement>(null)
  const dragging = useRef(false)

  const handleMouseDown = useCallback(() => {
    dragging.current = true
  }, [])

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      if (!dragging.current || !containerRef.current) return
      const rect = containerRef.current.getBoundingClientRect()
      const newRatio =
        direction === 'horizontal'
          ? (e.clientX - rect.left) / rect.width
          : (e.clientY - rect.top) / rect.height
      setRatio(Math.min(maxRatio, Math.max(minRatio, newRatio)))
    },
    [direction, minRatio, maxRatio],
  )

  const handleMouseUp = useCallback(() => {
    dragging.current = false
  }, [])

  const isHorizontal = direction === 'horizontal'

  return (
    <div
      ref={containerRef}
      className={cn(
        'flex overflow-hidden',
        isHorizontal ? 'horizontal flex-row' : 'vertical flex-col',
        className,
      )}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={handleMouseUp}
    >
      <div style={{ flexBasis: `${ratio * 100}%`, flexShrink: 0, flexGrow: 0 }}>{first}</div>
      <div
        role="separator"
        aria-orientation={isHorizontal ? 'vertical' : 'horizontal'}
        aria-label="Resize handle"
        onMouseDown={handleMouseDown}
        className={cn(
          'flex-shrink-0 cursor-col-resize bg-gray-200 transition-colors hover:bg-blue-400',
          isHorizontal ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize',
        )}
      />
      <div style={{ flex: 1, overflow: 'hidden' }}>{second}</div>
    </div>
  )
}
