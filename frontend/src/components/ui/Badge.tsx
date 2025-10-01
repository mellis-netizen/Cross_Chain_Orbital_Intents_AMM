'use client'

import * as React from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/utils'

const badgeVariants = cva(
  'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
  {
    variants: {
      variant: {
        default: 'border-transparent bg-primary text-primary-foreground hover:bg-primary/80',
        secondary: 'border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80',
        destructive: 'border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80',
        outline: 'text-foreground',
        success: 'border-transparent bg-success-100 text-success-800 dark:bg-success-900 dark:text-success-200',
        warning: 'border-transparent bg-warning-100 text-warning-800 dark:bg-warning-900 dark:text-warning-200',
        info: 'border-transparent bg-orbital-100 text-orbital-800 dark:bg-orbital-900 dark:text-orbital-200',
        pending: 'border-transparent bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
        matched: 'border-transparent bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
        executed: 'border-transparent bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
        failed: 'border-transparent bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
        cancelled: 'border-transparent bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {
  pulse?: boolean
}

function Badge({ className, variant, pulse, ...props }: BadgeProps) {
  return (
    <div 
      className={cn(
        badgeVariants({ variant }), 
        pulse && 'animate-pulse',
        className
      )} 
      {...props} 
    />
  )
}

// Status-specific badge components
export function StatusBadge({ status, ...props }: { status: string } & Omit<BadgeProps, 'variant'>) {
  const getVariant = (status: string) => {
    switch (status.toLowerCase()) {
      case 'created':
      case 'pending':
        return 'pending'
      case 'matched':
        return 'matched'
      case 'executed':
      case 'completed':
      case 'success':
        return 'executed'
      case 'failed':
      case 'error':
        return 'failed'
      case 'cancelled':
        return 'cancelled'
      default:
        return 'default'
    }
  }

  return (
    <Badge variant={getVariant(status)} {...props}>
      {status}
    </Badge>
  )
}

export { Badge, badgeVariants }