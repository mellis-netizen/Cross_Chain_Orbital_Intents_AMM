'use client'

import React from 'react'
import { cn } from '@/utils'

interface ProgressProps {
  value: number
  max?: number
  className?: string
  color?: 'blue' | 'green' | 'yellow' | 'red' | 'purple'
}

export function Progress({ value, max = 100, className, color = 'blue' }: ProgressProps) {
  const percentage = Math.min(Math.max((value / max) * 100, 0), 100)

  const colorClasses = {
    blue: 'bg-blue-500',
    green: 'bg-green-500',
    yellow: 'bg-yellow-500',
    red: 'bg-red-500',
    purple: 'bg-purple-500'
  }

  return (
    <div 
      className={cn(
        'relative h-2 w-full overflow-hidden rounded-full bg-white/20',
        className
      )}
    >
      <div
        className={cn(
          'h-full transition-all duration-300 ease-in-out',
          colorClasses[color]
        )}
        style={{ width: `${percentage}%` }}
      />
    </div>
  )
}