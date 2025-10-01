'use client'

import React, { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  CircleDot, 
  Zap, 
  TrendingUp, 
  Shield, 
  BarChart3, 
  Layers,
  Atom,
  Orbit,
  RefreshCw,
  Settings,
  Info
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { OrbitalPoolCreator } from '@/components/orbital/OrbitalPoolCreator'
import { ToroidalTrading } from '@/components/orbital/ToroidalTrading'
import { ConcentratedLiquidityManager } from '@/components/orbital/ConcentratedLiquidityManager'
import { OrbitalAnalytics } from '@/components/orbital/OrbitalAnalytics'
import { MEVProtectionPanel } from '@/components/orbital/MEVProtectionPanel'
import { TenTokenPoolDemo } from '@/components/orbital/TenTokenPoolDemo'

interface OrbitalStats {
  totalPools: number
  totalValueLocked: string
  capitalEfficiency: string
  activeTraders: number
  sphereIntegrity: number
  mevProtectionActive: boolean
}

export default function OrbitalAMMPage() {
  const [activeTab, setActiveTab] = useState('overview')
  const [orbitalStats, setOrbitalStats] = useState<OrbitalStats>({
    totalPools: 12,
    totalValueLocked: '$15.7M',
    capitalEfficiency: '127x',
    activeTraders: 1543,
    sphereIntegrity: 99.97,
    mevProtectionActive: true
  })

  const tabs = [
    { id: 'overview', label: 'Overview', icon: CircleDot },
    { id: 'pools', label: 'Pool Management', icon: Layers },
    { id: 'trading', label: 'Toroidal Trading', icon: Zap },
    { id: 'liquidity', label: 'Concentrated Liquidity', icon: TrendingUp },
    { id: 'analytics', label: 'Analytics', icon: BarChart3 },
    { id: 'protection', label: 'MEV Protection', icon: Shield },
    { id: 'demo', label: '10-Token Demo', icon: Atom },
  ]

  const fadeInUp = {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -20 }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* Animated background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute -top-40 -right-40 w-80 h-80 bg-purple-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-pulse" />
        <div className="absolute -bottom-40 -left-40 w-80 h-80 bg-blue-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-pulse animation-delay-2000" />
        <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-60 h-60 bg-indigo-500 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-pulse animation-delay-4000" />
      </div>

      <div className="relative z-10">
        {/* Header */}
        <header className="border-b border-white/10 backdrop-blur-md bg-white/5">
          <div className="container mx-auto px-6 py-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-4">
                <div className="relative">
                  <Orbit className="w-8 h-8 text-purple-400 animate-spin" style={{ animationDuration: '8s' }} />
                  <CircleDot className="w-4 h-4 text-blue-400 absolute top-2 left-2" />
                </div>
                <div>
                  <h1 className="text-2xl font-bold text-white">Orbital AMM</h1>
                  <p className="text-sm text-gray-400">N-Dimensional Automated Market Maker</p>
                </div>
              </div>
              
              <div className="flex items-center space-x-6">
                <div className="text-right">
                  <div className="text-sm text-gray-400">Sphere Integrity</div>
                  <div className="text-lg font-semibold text-green-400">{orbitalStats.sphereIntegrity}%</div>
                </div>
                <Badge variant={orbitalStats.mevProtectionActive ? 'success' : 'warning'}>
                  MEV {orbitalStats.mevProtectionActive ? 'Protected' : 'Vulnerable'}
                </Badge>
              </div>
            </div>
          </div>
        </header>

        {/* Navigation Tabs */}
        <nav className="border-b border-white/10 backdrop-blur-md bg-white/5">
          <div className="container mx-auto px-6">
            <div className="flex space-x-1">
              {tabs.map((tab) => {
                const Icon = tab.icon
                return (
                  <button
                    key={tab.id}
                    onClick={() => setActiveTab(tab.id)}
                    className={`relative px-6 py-4 text-sm font-medium transition-all duration-200 flex items-center space-x-2 ${
                      activeTab === tab.id
                        ? 'text-white bg-white/10'
                        : 'text-gray-400 hover:text-white hover:bg-white/5'
                    }`}
                  >
                    <Icon className="w-4 h-4" />
                    <span>{tab.label}</span>
                    {activeTab === tab.id && (
                      <motion.div
                        layoutId="activeTab"
                        className="absolute bottom-0 left-0 right-0 h-0.5 bg-purple-400"
                      />
                    )}
                  </button>
                )
              })}
            </div>
          </div>
        </nav>

        {/* Main Content */}
        <main className="container mx-auto px-6 py-8">
          <AnimatePresence mode="wait">
            {activeTab === 'overview' && (
              <motion.div
                key="overview"
                {...fadeInUp}
                transition={{ duration: 0.3 }}
                className="space-y-8"
              >
                {/* Stats Overview */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                  <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">Total Pools</p>
                        <p className="text-2xl font-bold text-white">{orbitalStats.totalPools}</p>
                      </div>
                      <Layers className="w-8 h-8 text-purple-400" />
                    </div>
                  </Card>

                  <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">TVL</p>
                        <p className="text-2xl font-bold text-white">{orbitalStats.totalValueLocked}</p>
                      </div>
                      <TrendingUp className="w-8 h-8 text-green-400" />
                    </div>
                  </Card>

                  <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">Capital Efficiency</p>
                        <p className="text-2xl font-bold text-white">{orbitalStats.capitalEfficiency}</p>
                      </div>
                      <Zap className="w-8 h-8 text-yellow-400" />
                    </div>
                  </Card>

                  <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-gray-400">Active Traders</p>
                        <p className="text-2xl font-bold text-white">{orbitalStats.activeTraders.toLocaleString()}</p>
                      </div>
                      <BarChart3 className="w-8 h-8 text-blue-400" />
                    </div>
                  </Card>
                </div>

                {/* Feature Highlights */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  <Card className="p-8 bg-gradient-to-br from-purple-900/20 to-blue-900/20 border-purple-500/20 backdrop-blur-md">
                    <div className="flex items-start space-x-4">
                      <div className="p-3 bg-purple-500/20 rounded-lg">
                        <Atom className="w-6 h-6 text-purple-400" />
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-white mb-2">N-Dimensional Trading</h3>
                        <p className="text-gray-300 mb-4">
                          Revolutionary spherical invariant (Σr²=R²) enables trading across 3-1000 tokens 
                          with unprecedented capital efficiency.
                        </p>
                        <div className="flex items-center space-x-4 text-sm text-gray-400">
                          <span>✓ Spherical constraints</span>
                          <span>✓ Superellipse curves</span>
                          <span>✓ Toroidal execution</span>
                        </div>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-8 bg-gradient-to-br from-green-900/20 to-emerald-900/20 border-green-500/20 backdrop-blur-md">
                    <div className="flex items-start space-x-4">
                      <div className="p-3 bg-green-500/20 rounded-lg">
                        <TrendingUp className="w-6 h-6 text-green-400" />
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-white mb-2">Concentrated Liquidity</h3>
                        <p className="text-gray-300 mb-4">
                          Hyperplane tick boundaries create spherical caps with 15x-150x capital 
                          efficiency improvements over traditional AMMs.
                        </p>
                        <div className="flex items-center space-x-4 text-sm text-gray-400">
                          <span>✓ Tick optimization</span>
                          <span>✓ IL minimization</span>
                          <span>✓ Dynamic rebalancing</span>
                        </div>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-8 bg-gradient-to-br from-blue-900/20 to-indigo-900/20 border-blue-500/20 backdrop-blur-md">
                    <div className="flex items-start space-x-4">
                      <div className="p-3 bg-blue-500/20 rounded-lg">
                        <Shield className="w-6 h-6 text-blue-400" />
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-white mb-2">MEV Protection</h3>
                        <p className="text-gray-300 mb-4">
                          Advanced commit-reveal schemes and batch execution protect traders 
                          from sandwich attacks and other MEV exploitation.
                        </p>
                        <div className="flex items-center space-x-4 text-sm text-gray-400">
                          <span>✓ Commit-reveal</span>
                          <span>✓ Batch execution</span>
                          <span>✓ Front-run protection</span>
                        </div>
                      </div>
                    </div>
                  </Card>

                  <Card className="p-8 bg-gradient-to-br from-orange-900/20 to-red-900/20 border-orange-500/20 backdrop-blur-md">
                    <div className="flex items-start space-x-4">
                      <div className="p-3 bg-orange-500/20 rounded-lg">
                        <RefreshCw className="w-6 h-6 text-orange-400" />
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-semibold text-white mb-2">Cross-Chain Ready</h3>
                        <p className="text-gray-300 mb-4">
                          Seamlessly integrates with the existing intent-based architecture 
                          for unified liquidity across multiple chains.
                        </p>
                        <div className="flex items-center space-x-4 text-sm text-gray-400">
                          <span>✓ Intent resolution</span>
                          <span>✓ Multi-chain pools</span>
                          <span>✓ Unified routing</span>
                        </div>
                      </div>
                    </div>
                  </Card>
                </div>

                {/* Mathematical Foundation */}
                <Card className="p-8 bg-white/5 border-white/10 backdrop-blur-md">
                  <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
                    <Info className="w-5 h-5 mr-2 text-purple-400" />
                    Mathematical Foundation
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="text-center">
                      <div className="text-2xl font-mono text-purple-400 mb-2">Σ(r²) = R²</div>
                      <div className="text-sm text-gray-300">Spherical Invariant</div>
                      <div className="text-xs text-gray-400 mt-1">N-dimensional constraint</div>
                    </div>
                    <div className="text-center">
                      <div className="text-2xl font-mono text-green-400 mb-2">Σ(|r|ᵘ) = K</div>
                      <div className="text-sm text-gray-300">Superellipse</div>
                      <div className="text-xs text-gray-400 mt-1">Stablecoin optimization</div>
                    </div>
                    <div className="text-center">
                      <div className="text-2xl font-mono text-blue-400 mb-2">T = S ∪ C</div>
                      <div className="text-sm text-gray-300">Toroidal Surface</div>
                      <div className="text-xs text-gray-400 mt-1">Combined liquidity</div>
                    </div>
                  </div>
                </Card>
              </motion.div>
            )}

            {activeTab === 'pools' && (
              <motion.div key="pools" {...fadeInUp} transition={{ duration: 0.3 }}>
                <OrbitalPoolCreator />
              </motion.div>
            )}

            {activeTab === 'trading' && (
              <motion.div key="trading" {...fadeInUp} transition={{ duration: 0.3 }}>
                <ToroidalTrading />
              </motion.div>
            )}

            {activeTab === 'liquidity' && (
              <motion.div key="liquidity" {...fadeInUp} transition={{ duration: 0.3 }}>
                <ConcentratedLiquidityManager />
              </motion.div>
            )}

            {activeTab === 'analytics' && (
              <motion.div key="analytics" {...fadeInUp} transition={{ duration: 0.3 }}>
                <OrbitalAnalytics />
              </motion.div>
            )}

            {activeTab === 'protection' && (
              <motion.div key="protection" {...fadeInUp} transition={{ duration: 0.3 }}>
                <MEVProtectionPanel />
              </motion.div>
            )}

            {activeTab === 'demo' && (
              <motion.div key="demo" {...fadeInUp} transition={{ duration: 0.3 }}>
                <TenTokenPoolDemo />
              </motion.div>
            )}
          </AnimatePresence>
        </main>
      </div>
    </div>
  )
}