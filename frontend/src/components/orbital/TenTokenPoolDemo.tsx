'use client'

import React, { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  Atom, 
  TrendingUp, 
  Zap, 
  BarChart3, 
  DollarSign,
  Layers,
  CircleDot,
  RefreshCw,
  Play,
  CheckCircle
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'

interface DemoToken {
  symbol: string
  name: string
  category: 'stable' | 'volatile' | 'synthetic'
  reserve: number
  price: number
  weight: number
  color: string
}

interface TradingScenario {
  id: string
  name: string
  description: string
  tokenIn: string
  tokenOut: string
  amount: number
  expectedOutput: number
  priceImpact: number
  efficiency: number
  mode: 'spherical' | 'circular' | 'toroidal'
}

const DEMO_TOKENS: DemoToken[] = [
  { symbol: 'USDC', name: 'USD Coin', category: 'stable', reserve: 10000000, price: 1.00, weight: 0.25, color: '#2775CA' },
  { symbol: 'USDT', name: 'Tether USD', category: 'stable', reserve: 8000000, price: 1.00, weight: 0.20, color: '#26A17B' },
  { symbol: 'DAI', name: 'Dai Stablecoin', category: 'stable', reserve: 12000000, price: 1.00, weight: 0.30, color: '#F5AC37' },
  { symbol: 'FRAX', name: 'Frax', category: 'stable', reserve: 5000000, price: 1.00, weight: 0.125, color: '#000000' },
  { symbol: 'WETH', name: 'Wrapped Ether', category: 'volatile', reserve: 250, price: 2000, weight: 0.125, color: '#627EEA' },
  { symbol: 'WBTC', name: 'Wrapped Bitcoin', category: 'volatile', reserve: 15, price: 43000, weight: 0.162, color: '#F7931A' },
  { symbol: 'LINK', name: 'Chainlink', category: 'volatile', reserve: 50000, price: 15, weight: 0.188, color: '#375BD2' },
  { symbol: 'UNI', name: 'Uniswap', category: 'volatile', reserve: 80000, price: 7, weight: 0.140, color: '#FF007A' },
  { symbol: 'stETH', name: 'Lido Staked ETH', category: 'synthetic', reserve: 240, price: 2000, weight: 0.120, color: '#00A3FF' },
  { symbol: 'rETH', name: 'Rocket Pool ETH', category: 'synthetic', reserve: 180, price: 2000, weight: 0.090, color: '#FF6B35' },
]

const TRADING_SCENARIOS: TradingScenario[] = [
  {
    id: '1',
    name: 'Stablecoin Arbitrage',
    description: 'Low-impact swap between correlated stablecoins using superellipse optimization',
    tokenIn: 'USDC',
    tokenOut: 'USDT',
    amount: 100000,
    expectedOutput: 99950,
    priceImpact: 0.01,
    efficiency: 150,
    mode: 'spherical'
  },
  {
    id: '2',
    name: 'Volatile Asset Swap',
    description: 'Cross-volatile swap utilizing concentrated liquidity zones',
    tokenIn: 'WETH',
    tokenOut: 'WBTC',
    amount: 10,
    expectedOutput: 0.4651,
    priceImpact: 0.15,
    efficiency: 75,
    mode: 'toroidal'
  },
  {
    id: '3',
    name: 'Cross-Category Trade',
    description: 'Stable to volatile using hybrid toroidal routing',
    tokenIn: 'DAI',
    tokenOut: 'LINK',
    amount: 50000,
    expectedOutput: 3333.33,
    priceImpact: 0.08,
    efficiency: 95,
    mode: 'circular'
  },
  {
    id: '4',
    name: 'Multi-Hop Complex',
    description: 'Complex routing through multiple tokens for optimal price',
    tokenIn: 'FRAX',
    tokenOut: 'UNI',
    amount: 25000,
    expectedOutput: 3571.43,
    priceImpact: 0.22,
    efficiency: 85,
    mode: 'toroidal'
  },
  {
    id: '5',
    name: 'Synthetic Asset Rebalance',
    description: 'Liquid staking derivatives swap with minimal slippage',
    tokenIn: 'stETH',
    tokenOut: 'rETH',
    amount: 50,
    expectedOutput: 50,
    priceImpact: 0.02,
    efficiency: 125,
    mode: 'spherical'
  }
]

export function TenTokenPoolDemo() {
  const [poolMetrics, setPoolMetrics] = useState({
    totalValueLocked: 125750000,
    sphereRadius: 2.51e14,
    capitalEfficiency: 127,
    volume24h: 5200000,
    trades24h: 1543,
    avgSlippage: 0.045
  })

  const [runningScenario, setRunningScenario] = useState<string | null>(null)
  const [completedScenarios, setCompletedScenarios] = useState<Set<string>>(new Set())
  const [isRunningAll, setIsRunningAll] = useState(false)

  const runScenario = async (scenarioId: string) => {
    setRunningScenario(scenarioId)
    
    // Simulate execution time
    await new Promise(resolve => setTimeout(resolve, 2000))
    
    setCompletedScenarios(prev => new Set(prev).add(scenarioId))
    setRunningScenario(null)
    
    // Update pool metrics
    setPoolMetrics(prev => ({
      ...prev,
      volume24h: prev.volume24h + Math.random() * 100000,
      trades24h: prev.trades24h + 1
    }))
  }

  const runAllScenarios = async () => {
    setIsRunningAll(true)
    setCompletedScenarios(new Set())
    
    for (const scenario of TRADING_SCENARIOS) {
      await runScenario(scenario.id)
    }
    
    setIsRunningAll(false)
  }

  const calculateNormalizedReserve = (token: DemoToken) => {
    return token.reserve * token.price
  }

  const totalPoolValue = DEMO_TOKENS.reduce((sum, token) => sum + calculateNormalizedReserve(token), 0)

  return (
    <div className="space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">10-Token Pool Demonstration</h2>
        <p className="text-gray-400 max-w-3xl mx-auto">
          Experience the full power of N-dimensional Orbital AMM with a real-world 10-token pool.
          This demonstration showcases capital efficiency gains of 127x over traditional AMMs.
        </p>
      </div>

      {/* Pool Overview */}
      <Card className="p-8 bg-gradient-to-br from-purple-900/20 to-blue-900/20 border-purple-500/20 backdrop-blur-md">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="text-center">
            <div className="text-3xl font-bold text-white mb-2">
              ${(poolMetrics.totalValueLocked / 1000000).toFixed(1)}M
            </div>
            <div className="text-sm text-gray-400">Total Value Locked</div>
          </div>
          <div className="text-center">
            <div className="text-3xl font-bold text-purple-400 mb-2">
              {poolMetrics.capitalEfficiency}x
            </div>
            <div className="text-sm text-gray-400">Capital Efficiency</div>
          </div>
          <div className="text-center">
            <div className="text-3xl font-bold text-green-400 mb-2">
              ${(poolMetrics.volume24h / 1000000).toFixed(1)}M
            </div>
            <div className="text-sm text-gray-400">24h Volume</div>
          </div>
          <div className="text-center">
            <div className="text-3xl font-bold text-blue-400 mb-2">
              {poolMetrics.avgSlippage.toFixed(3)}%
            </div>
            <div className="text-sm text-gray-400">Avg Slippage</div>
          </div>
        </div>

        <div className="text-center">
          <div className="text-lg font-semibold text-white mb-2">
            Sphere Constraint: Σ(r²) = {poolMetrics.sphereRadius.toExponential(2)}
          </div>
          <div className="text-sm text-gray-400">
            Mathematical invariant maintaining perfect balance across all 10 dimensions
          </div>
        </div>
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Token Composition */}
        <div className="lg:col-span-2 space-y-6">
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
              <Atom className="w-5 h-5 mr-2 text-purple-400" />
              Token Composition
            </h3>

            <div className="space-y-4">
              {['stable', 'volatile', 'synthetic'].map((category) => (
                <div key={category}>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-lg font-medium text-white capitalize">
                      {category} Assets
                    </h4>
                    <Badge 
                      variant={category === 'stable' ? 'success' : category === 'volatile' ? 'warning' : 'info'}
                    >
                      {DEMO_TOKENS.filter(t => t.category === category).length} tokens
                    </Badge>
                  </div>
                  
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {DEMO_TOKENS.filter(token => token.category === category).map((token) => {
                      const normalizedValue = calculateNormalizedReserve(token)
                      const percentage = (normalizedValue / totalPoolValue * 100)
                      
                      return (
                        <div key={token.symbol} className="p-4 bg-white/5 rounded-lg border border-white/10">
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center space-x-3">
                              <div 
                                className="w-3 h-3 rounded-full"
                                style={{ backgroundColor: token.color }}
                              />
                              <span className="font-medium text-white">{token.symbol}</span>
                            </div>
                            <span className="text-sm text-gray-400">{percentage.toFixed(1)}%</span>
                          </div>
                          
                          <div className="text-xs text-gray-400 mb-2">{token.name}</div>
                          
                          <div className="flex items-center justify-between text-sm">
                            <span className="text-gray-400">Reserve:</span>
                            <span className="text-white">
                              {token.reserve.toLocaleString()} {token.symbol}
                            </span>
                          </div>
                          
                          <div className="flex items-center justify-between text-sm">
                            <span className="text-gray-400">Value:</span>
                            <span className="text-white">
                              ${(normalizedValue / 1000000).toFixed(2)}M
                            </span>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </div>
              ))}
            </div>
          </Card>

          {/* Visual Representation */}
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
              <CircleDot className="w-5 h-5 mr-2 text-blue-400" />
              Spherical Distribution
            </h3>

            <div className="relative">
              <div className="aspect-square max-w-md mx-auto relative">
                <div className="absolute inset-0 border-2 border-blue-400/30 rounded-full"></div>
                <div className="absolute inset-4 border border-purple-400/20 rounded-full"></div>
                <div className="absolute inset-8 border border-green-400/20 rounded-full"></div>
                
                {DEMO_TOKENS.map((token, index) => {
                  const angle = (index * 360) / DEMO_TOKENS.length
                  const radius = 35 + (token.weight * 20)
                  const x = 50 + radius * Math.cos((angle * Math.PI) / 180)
                  const y = 50 + radius * Math.sin((angle * Math.PI) / 180)
                  
                  return (
                    <div
                      key={token.symbol}
                      className="absolute transform -translate-x-1/2 -translate-y-1/2"
                      style={{ left: `${x}%`, top: `${y}%` }}
                    >
                      <div
                        className="w-8 h-8 rounded-full border-2 border-white/20 flex items-center justify-center text-xs font-bold text-white"
                        style={{ backgroundColor: token.color }}
                      >
                        {token.symbol.slice(0, 2)}
                      </div>
                    </div>
                  )
                })}
              </div>
              
              <div className="text-center mt-4">
                <div className="text-sm text-gray-400 mb-2">
                  N-dimensional projection showing token distribution on sphere surface
                </div>
                <div className="text-xs text-gray-500">
                  Inner rings represent concentrated liquidity zones
                </div>
              </div>
            </div>
          </Card>
        </div>

        {/* Trading Scenarios */}
        <div className="space-y-6">
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-semibold text-white flex items-center">
                <Zap className="w-5 h-5 mr-2 text-yellow-400" />
                Trading Scenarios
              </h3>
              <Button
                onClick={runAllScenarios}
                disabled={isRunningAll}
                size="sm"
                className="bg-gradient-to-r from-purple-600 to-blue-600"
              >
                {isRunningAll ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Play className="w-4 h-4" />
                )}
              </Button>
            </div>

            <div className="space-y-4">
              {TRADING_SCENARIOS.map((scenario) => (
                <motion.div
                  key={scenario.id}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: parseInt(scenario.id) * 0.1 }}
                  className="p-4 bg-white/5 rounded-lg border border-white/10"
                >
                  <div className="flex items-center justify-between mb-3">
                    <div>
                      <h4 className="font-medium text-white">{scenario.name}</h4>
                      <p className="text-xs text-gray-400 mt-1">{scenario.description}</p>
                    </div>
                    <div className="flex items-center space-x-2">
                      <Badge
                        variant={
                          scenario.mode === 'spherical' ? 'success' :
                          scenario.mode === 'circular' ? 'warning' : 'info'
                        }
                        className="text-xs"
                      >
                        {scenario.mode}
                      </Badge>
                      {completedScenarios.has(scenario.id) ? (
                        <CheckCircle className="w-4 h-4 text-green-400" />
                      ) : runningScenario === scenario.id ? (
                        <RefreshCw className="w-4 h-4 text-blue-400 animate-spin" />
                      ) : (
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => runScenario(scenario.id)}
                          className="p-1"
                        >
                          <Play className="w-3 h-3" />
                        </Button>
                      )}
                    </div>
                  </div>

                  <div className="space-y-2 text-xs">
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Trade:</span>
                      <span className="text-white">
                        {scenario.amount.toLocaleString()} {scenario.tokenIn} → {scenario.tokenOut}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Output:</span>
                      <span className="text-white">
                        {scenario.expectedOutput.toLocaleString()} {scenario.tokenOut}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Impact:</span>
                      <span className="text-green-400">{scenario.priceImpact}%</span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Efficiency:</span>
                      <span className="text-purple-400">{scenario.efficiency}x</span>
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          </Card>

          {/* Performance Metrics */}
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
              <BarChart3 className="w-5 h-5 mr-2 text-green-400" />
              Performance vs Traditional
            </h3>

            <div className="space-y-6">
              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Capital Efficiency</span>
                  <span className="text-sm font-medium text-green-400">127x better</span>
                </div>
                <div className="w-full bg-gray-700 rounded-full h-2">
                  <div className="bg-green-400 h-2 rounded-full" style={{ width: '95%' }}></div>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Slippage Reduction</span>
                  <span className="text-sm font-medium text-blue-400">85% lower</span>
                </div>
                <div className="w-full bg-gray-700 rounded-full h-2">
                  <div className="bg-blue-400 h-2 rounded-full" style={{ width: '85%' }}></div>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Impermanent Loss</span>
                  <span className="text-sm font-medium text-purple-400">70% reduced</span>
                </div>
                <div className="w-full bg-gray-700 rounded-full h-2">
                  <div className="bg-purple-400 h-2 rounded-full" style={{ width: '70%' }}></div>
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm text-gray-400">Gas Efficiency</span>
                  <span className="text-sm font-medium text-yellow-400">40% lower</span>
                </div>
                <div className="w-full bg-gray-700 rounded-full h-2">
                  <div className="bg-yellow-400 h-2 rounded-full" style={{ width: '40%' }}></div>
                </div>
              </div>
            </div>

            <div className="mt-6 p-4 bg-gradient-to-r from-green-500/20 to-blue-500/20 rounded-lg border border-green-500/20">
              <div className="text-center">
                <div className="text-2xl font-bold text-white mb-1">
                  ${((poolMetrics.volume24h * 365) / 1000000).toFixed(1)}M
                </div>
                <div className="text-sm text-gray-300">
                  Projected Annual Volume
                </div>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}