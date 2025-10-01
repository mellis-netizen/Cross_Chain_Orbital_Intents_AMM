'use client'

import { useState } from 'react'
import Link from 'next/link'
import { Menu, X, Wallet, ChevronDown } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { WalletConnector } from '@/components/wallet/WalletConnector'
import { NetworkStatus } from '@/components/wallet/NetworkStatus'
import { useWallet } from '@/hooks/useWeb3'
import { truncateAddress } from '@/utils'
import { cn } from '@/utils'

const navigation = [
  { name: 'Orbital AMM', href: '/orbital' },
  { name: 'Swap', href: '/swap' },
  { name: 'Intents', href: '/intents' },
  { name: 'Pools', href: '/pools' },
  { name: 'Analytics', href: '/analytics' },
]

export function Header() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const { address, isConnected } = useWallet()

  return (
    <header className="sticky top-0 z-40 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto px-4">
        <div className="flex h-16 items-center justify-between">
          {/* Logo */}
          <div className="flex items-center space-x-2">
            <Link href="/" className="flex items-center space-x-2">
              <div className="h-8 w-8 rounded-lg bg-gradient-to-br from-orbital-500 to-orbital-600 flex items-center justify-center">
                <span className="text-white font-bold text-sm">O</span>
              </div>
              <span className="font-bold text-xl text-foreground">
                Orbital <span className="text-orbital-500">AMM</span>
              </span>
            </Link>
          </div>

          {/* Desktop Navigation */}
          <nav className="hidden md:flex items-center space-x-8">
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
              >
                {item.name}
              </Link>
            ))}
          </nav>

          {/* Desktop Actions */}
          <div className="hidden md:flex items-center space-x-4">
            <NetworkStatus />
            <WalletConnector />
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            >
              {mobileMenuOpen ? (
                <X className="h-5 w-5" />
              ) : (
                <Menu className="h-5 w-5" />
              )}
            </Button>
          </div>
        </div>

        {/* Mobile Navigation */}
        {mobileMenuOpen && (
          <div className="md:hidden">
            <div className="px-2 pt-2 pb-3 space-y-1">
              {navigation.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  className="block px-3 py-2 text-base font-medium text-muted-foreground hover:text-foreground transition-colors"
                  onClick={() => setMobileMenuOpen(false)}
                >
                  {item.name}
                </Link>
              ))}
            </div>
            <div className="px-2 py-3 border-t border-border">
              <div className="space-y-3">
                <NetworkStatus />
                <WalletConnector />
              </div>
            </div>
          </div>
        )}
      </div>
    </header>
  )
}

// Quick stats bar component
export function StatsBar() {
  return (
    <div className="border-b bg-muted/20">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between py-3 text-sm">
          <div className="flex items-center space-x-6">
            <div className="flex items-center space-x-2">
              <span className="text-muted-foreground">TVL:</span>
              <span className="font-semibold text-orbital-600">$2.4M</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-muted-foreground">24h Volume:</span>
              <span className="font-semibold text-success-600">$148K</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-muted-foreground">Intents:</span>
              <span className="font-semibold">1,247</span>
            </div>
          </div>
          <div className="hidden sm:flex items-center space-x-2">
            <div className="h-2 w-2 rounded-full bg-success-500 animate-pulse"></div>
            <span className="text-xs text-muted-foreground">
              Holesky Testnet
            </span>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Header