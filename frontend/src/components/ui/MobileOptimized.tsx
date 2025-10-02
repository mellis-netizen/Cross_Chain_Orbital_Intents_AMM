'use client'

import { useState, useEffect } from 'react'
import { ChevronDown, Menu, X, Home, Repeat, Bridge, History, Settings } from 'lucide-react'
import { Button } from './Button'
import { Card } from './Card'
import { Badge } from './Badge'
import { cn } from '@/utils'

// Mobile-optimized bottom navigation
export function MobileBottomNav({ activeTab, onTabChange }: {
  activeTab: string
  onTabChange: (tab: string) => void
}) {
  const tabs = [
    { id: 'swap', icon: Repeat, label: 'Swap' },
    { id: 'bridge', icon: Bridge, label: 'Bridge' },
    { id: 'intents', icon: Home, label: 'Intents' },
    { id: 'history', icon: History, label: 'History' },
  ]

  return (
    <div className="fixed bottom-0 left-0 right-0 bg-background/95 backdrop-blur-md border-t border-border z-50 md:hidden">
      <div className="flex items-center justify-around px-2 py-3">
        {tabs.map(({ id, icon: Icon, label }) => (
          <button
            key={id}
            onClick={() => onTabChange(id)}
            className={cn(
              'flex flex-col items-center space-y-1 px-3 py-2 rounded-lg transition-colors',
              activeTab === id
                ? 'text-primary bg-primary/10'
                : 'text-muted-foreground hover:text-foreground'
            )}
          >
            <Icon className="h-5 w-5" />
            <span className="text-xs font-medium">{label}</span>
          </button>
        ))}
      </div>
    </div>
  )
}

// Mobile-optimized card with collapsible sections
export function MobileCard({ 
  title, 
  children, 
  collapsible = false, 
  defaultOpen = true,
  badge,
  className = ''
}: {
  title: string
  children: React.ReactNode
  collapsible?: boolean
  defaultOpen?: boolean
  badge?: string
  className?: string
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen)

  return (
    <Card className={cn('mx-2 mb-3', className)}>
      <div 
        className={cn(
          'flex items-center justify-between p-4',
          collapsible && 'cursor-pointer'
        )}
        onClick={collapsible ? () => setIsOpen(!isOpen) : undefined}
      >
        <div className="flex items-center space-x-2">
          <h3 className="font-semibold">{title}</h3>
          {badge && (
            <Badge variant="secondary" className="text-xs">
              {badge}
            </Badge>
          )}
        </div>
        {collapsible && (
          <ChevronDown 
            className={cn(
              'h-4 w-4 transition-transform',
              isOpen && 'transform rotate-180'
            )}
          />
        )}
      </div>
      {(!collapsible || isOpen) && (
        <div className="px-4 pb-4">
          {children}
        </div>
      )}
    </Card>
  )
}

// Mobile-optimized drawer for settings and additional options
export function MobileDrawer({ 
  isOpen, 
  onClose, 
  title, 
  children 
}: {
  isOpen: boolean
  onClose: () => void
  title: string
  children: React.ReactNode
}) {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden'
    } else {
      document.body.style.overflow = 'unset'
    }
    
    return () => {
      document.body.style.overflow = 'unset'
    }
  }, [isOpen])

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 md:hidden">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black/50" 
        onClick={onClose}
      />
      
      {/* Drawer */}
      <div className="absolute bottom-0 left-0 right-0 bg-background rounded-t-2xl max-h-[90vh] overflow-auto">
        <div className="sticky top-0 bg-background border-b border-border p-4">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-semibold">{title}</h2>
            <Button
              variant="ghost"
              size="icon"
              onClick={onClose}
            >
              <X className="h-5 w-5" />
            </Button>
          </div>
        </div>
        <div className="p-4">
          {children}
        </div>
      </div>
    </div>
  )
}

// Mobile-optimized action sheet for quick actions
export function MobileActionSheet({ 
  isOpen, 
  onClose, 
  actions 
}: {
  isOpen: boolean
  onClose: () => void
  actions: Array<{
    label: string
    icon: React.ComponentType<{ className?: string }>
    onClick: () => void
    variant?: 'default' | 'destructive'
  }>
}) {
  if (!isOpen) return null

  return (
    <div className="fixed inset-0 z-50 md:hidden">
      <div 
        className="absolute inset-0 bg-black/50" 
        onClick={onClose}
      />
      
      <div className="absolute bottom-0 left-0 right-0 bg-background rounded-t-2xl p-4">
        <div className="w-12 h-1 bg-muted rounded-full mx-auto mb-4" />
        
        <div className="space-y-2">
          {actions.map((action, index) => (
            <button
              key={index}
              onClick={() => {
                action.onClick()
                onClose()
              }}
              className={cn(
                'w-full flex items-center space-x-3 p-4 rounded-lg transition-colors',
                action.variant === 'destructive'
                  ? 'text-destructive hover:bg-destructive/10'
                  : 'text-foreground hover:bg-muted'
              )}
            >
              <action.icon className="h-5 w-5" />
              <span className="font-medium">{action.label}</span>
            </button>
          ))}
        </div>
        
        <Button
          variant="outline"
          onClick={onClose}
          className="w-full mt-4"
        >
          Cancel
        </Button>
      </div>
    </div>
  )
}

// Mobile-optimized form input with improved touch targets
export function MobileInput({ 
  label, 
  value, 
  onChange, 
  placeholder, 
  type = 'text',
  rightElement,
  error
}: {
  label: string
  value: string
  onChange: (value: string) => void
  placeholder?: string
  type?: string
  rightElement?: React.ReactNode
  error?: string
}) {
  return (
    <div className="space-y-2">
      <label className="text-sm font-medium text-muted-foreground">
        {label}
      </label>
      <div className="relative">
        <input
          type={type}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className={cn(
            'w-full px-4 py-3 text-base border rounded-lg bg-background text-foreground',
            'focus:ring-2 focus:ring-primary focus:border-transparent',
            'placeholder:text-muted-foreground',
            rightElement && 'pr-12',
            error && 'border-destructive'
          )}
        />
        {rightElement && (
          <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
            {rightElement}
          </div>
        )}
      </div>
      {error && (
        <p className="text-sm text-destructive">{error}</p>
      )}
    </div>
  )
}

// Mobile-optimized transaction summary card
export function MobileTransactionSummary({ 
  type, 
  fromToken, 
  toToken, 
  fromAmount, 
  toAmount, 
  fee, 
  estimatedTime 
}: {
  type: 'swap' | 'bridge' | 'intent'
  fromToken: string
  toToken: string
  fromAmount: string
  toAmount: string
  fee?: string
  estimatedTime?: string
}) {
  return (
    <Card className="mx-2 mb-4">
      <div className="p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold capitalize">{type} Summary</h3>
          <Badge variant="info" className="capitalize">
            {type}
          </Badge>
        </div>
        
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 bg-muted/20 rounded-lg">
            <div>
              <div className="text-sm text-muted-foreground">You send</div>
              <div className="font-medium">{fromAmount} {fromToken}</div>
            </div>
            <div className="text-right">
              <div className="text-sm text-muted-foreground">You receive</div>
              <div className="font-medium">{toAmount} {toToken}</div>
            </div>
          </div>
          
          {(fee || estimatedTime) && (
            <div className="grid grid-cols-2 gap-3 text-sm">
              {fee && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Fee:</span>
                  <span>{fee}</span>
                </div>
              )}
              {estimatedTime && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Time:</span>
                  <span>{estimatedTime}</span>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </Card>
  )
}

// Mobile-responsive grid layout
export function MobileResponsiveGrid({ 
  children, 
  cols = { mobile: 1, tablet: 2, desktop: 3 } 
}: {
  children: React.ReactNode
  cols?: {
    mobile: number
    tablet: number
    desktop: number
  }
}) {
  return (
    <div className={cn(
      'grid gap-4 px-2',
      `grid-cols-${cols.mobile}`,
      `md:grid-cols-${cols.tablet}`,
      `lg:grid-cols-${cols.desktop}`
    )}>
      {children}
    </div>
  )
}

// Mobile-optimized safe area wrapper
export function MobileSafeArea({ 
  children, 
  className = '' 
}: {
  children: React.ReactNode
  className?: string
}) {
  return (
    <div className={cn(
      'min-h-screen pb-20 md:pb-0', // Add bottom padding for mobile nav
      'safe-area-inset-top safe-area-inset-bottom',
      className
    )}>
      {children}
    </div>
  )
}

export default {
  MobileBottomNav,
  MobileCard,
  MobileDrawer,
  MobileActionSheet,
  MobileInput,
  MobileTransactionSummary,
  MobileResponsiveGrid,
  MobileSafeArea
}