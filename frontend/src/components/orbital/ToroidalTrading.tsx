'use client'

import React, { useState, useEffect, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Zap, 
  ArrowUpDown, 
  Settings, 
  TrendingUp, 
  Shield, 
  RefreshCw,
  CircleDot,
  BarChart3,
  AlertTriangle,
  CheckCircle,
  Route
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Badge } from '@/components/ui/Badge'

interface Token {
  address: string
  symbol: string
  name: string
  balance: string
  price: number
  reserve: string
}

interface SwapRoute {
  path: number[]
  tokens: string[]
  priceImpact: number
  gas: number
  sphereUtilization: number
  toroidalMode: 'spherical' | 'circular' | 'hybrid'
}

interface Pool {
  id: string
  tokens: Token[]
  sphereRadius: number
  totalValueLocked: string
  volume24h: string
  concentratedLiquidity: number
}

const SAMPLE_POOLS: Pool[] = [
  {
    id: '0x1',
    tokens: [
      { address: '0x1', symbol: 'USDC', name: 'USD Coin', balance: '1000', price: 1.00, reserve: '10000000' },
      { address: '0x2', symbol: 'USDT', name: 'Tether', balance: '500', price: 1.00, reserve: '8000000' },
      { address: '0x3', symbol: 'DAI', name: 'Dai', balance: '750', price: 1.00, reserve: '12000000' },
      { address: '0x4', symbol: 'FRAX', name: 'Frax', balance: '250', price: 1.00, reserve: '5000000' },
    ],
    sphereRadius: 15811388.3,
    totalValueLocked: '$35.0M',
    volume24h: '$2.3M',
    concentratedLiquidity: 85
  },
  {
    id: '0x2',
    tokens: [
      { address: '0x5', symbol: 'WETH', name: 'Wrapped Ether', balance: '10', price: 2000, reserve: '250' },
      { address: '0x6', symbol: 'WBTC', name: 'Wrapped Bitcoin', balance: '0.5', price: 43000, reserve: '15' },
      { address: '0x7', symbol: 'LINK', name: 'Chainlink', balance: '1000', price: 15, reserve: '50000' },
      { address: '0x8', symbol: 'UNI', name: 'Uniswap', balance: '500', price: 7, reserve: '80000' },
    ],
    sphereRadius: 9486832.98,
    totalValueLocked: '$28.5M',
    volume24h: '$4.1M',
    concentratedLiquidity: 72
  }
]

export function ToroidalTrading() {
  const [selectedPool, setSelectedPool] = useState<Pool>(SAMPLE_POOLS[0])
  const [tokenIn, setTokenIn] = useState<Token | null>(null)
  const [tokenOut, setTokenOut] = useState<Token | null>(null)
  const [amountIn, setAmountIn] = useState('')
  const [amountOut, setAmountOut] = useState('')
  const [routes, setRoutes] = useState<SwapRoute[]>([])
  const [selectedRoute, setSelectedRoute] = useState<SwapRoute | null>(null)
  const [slippageTolerance, setSlippageTolerance] = useState('0.5')
  const [mevProtection, setMevProtection] = useState(true)
  const [isCalculating, setIsCalculating] = useState(false)

  const calculateRoutes = useCallback(async () => {
    if (!tokenIn || !tokenOut || !amountIn || parseFloat(amountIn) <= 0) {
      setRoutes([])
      setSelectedRoute(null)
      return
    }

    setIsCalculating(true)
    
    // Simulate route calculation
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    const mockRoutes: SwapRoute[] = [
      {
        path: [0, 1],
        tokens: [tokenIn.symbol, tokenOut.symbol],
        priceImpact: 0.05,
        gas: 120000,
        sphereUtilization: 23.5,
        toroidalMode: 'spherical'
      },
      {
        path: [0, 2, 1],
        tokens: [tokenIn.symbol, 'DAI', tokenOut.symbol],
        priceImpact: 0.08,
        gas: 180000,
        sphereUtilization: 45.2,
        toroidalMode: 'hybrid'
      },
      {
        path: [0, 3, 1],
        tokens: [tokenIn.symbol, 'FRAX', tokenOut.symbol],
        priceImpact: 0.12,
        gas: 190000,
        sphereUtilization: 67.8,
        toroidalMode: 'circular'
      }
    ]
    
    setRoutes(mockRoutes)
    setSelectedRoute(mockRoutes[0])
    
    // Calculate output amount
    const baseAmount = parseFloat(amountIn) * tokenOut.price / tokenIn.price
    const adjustedAmount = baseAmount * (1 - mockRoutes[0].priceImpact / 100)
    setAmountOut(adjustedAmount.toFixed(6))
    
    setIsCalculating(false)
  }, [tokenIn, tokenOut, amountIn])

  useEffect(() => {
    calculateRoutes()
  }, [calculateRoutes])

  const executeSwap = async () => {
    if (!selectedRoute) return
    
    // Simulate swap execution
    console.log('Executing toroidal swap:', {
      tokenIn: tokenIn?.symbol,
      tokenOut: tokenOut?.symbol,
      amountIn,
      route: selectedRoute,
      mevProtection
    })
  }

  const swapTokens = () => {
    const temp = tokenIn
    setTokenIn(tokenOut)
    setTokenOut(temp)
    setAmountIn(amountOut)
    setAmountOut('')
  }

  return (
    <div className="space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">Toroidal Trading Interface</h2>
        <p className="text-gray-400 max-w-2xl mx-auto">
          Experience revolutionary N-dimensional trading with combined spherical and circular liquidity.
          Optimal routing across toroidal surfaces for maximum capital efficiency.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Trading Interface */}
        <div className="lg:col-span-2 space-y-6">
          {/* Pool Selection */}
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
              <CircleDot className="w-5 h-5 mr-2 text-purple-400" />
              Select Pool
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {SAMPLE_POOLS.map((pool) => (
                <div
                  key={pool.id}
                  onClick={() => setSelectedPool(pool)}
                  className={`p-4 rounded-lg border cursor-pointer transition-all ${
                    selectedPool.id === pool.id
                      ? 'border-purple-400 bg-purple-500/10'
                      : 'border-white/20 bg-white/5 hover:border-white/40'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-2">
                      {pool.tokens.slice(0, 3).map((token, i) => (
                        <Badge key={i} variant="outline" className="text-xs">
                          {token.symbol}
                        </Badge>
                      ))}
                      {pool.tokens.length > 3 && (
                        <Badge variant="outline" className="text-xs">
                          +{pool.tokens.length - 3}
                        </Badge>
                      )}
                    </div>
                    <Badge variant="success" className="text-xs">
                      {pool.concentratedLiquidity}% CL
                    </Badge>
                  </div>
                  <div className="text-sm text-gray-400">
                    TVL: {pool.totalValueLocked} • 24h: {pool.volume24h}
                  </div>
                </div>
              ))}
            </div>
          </Card>

          {/* Swap Interface */}
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-semibold text-white flex items-center">
                <Zap className="w-5 h-5 mr-2 text-yellow-400" />
                Toroidal Swap
              </h3>
              <div className="flex items-center space-x-2">
                <Badge variant={mevProtection ? 'success' : 'warning'}>
                  MEV {mevProtection ? 'Protected' : 'Exposed'}
                </Badge>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setMevProtection(!mevProtection)}
                >
                  <Shield className="w-4 h-4" />
                </Button>
              </div>
            </div>

            <div className="space-y-4">
              {/* Token In */}
              <div className="p-4 bg-white/5 rounded-lg border border-white/10">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm text-gray-400">From</label>
                  <span className="text-sm text-gray-400">
                    Balance: {tokenIn?.balance || '0'} {tokenIn?.symbol || ''}
                  </span>
                </div>
                <div className="flex items-center space-x-3">
                  <Input
                    placeholder="0.0"
                    value={amountIn}
                    onChange={(e) => setAmountIn(e.target.value)}
                    className="bg-transparent border-none text-2xl font-semibold text-white placeholder-gray-500 flex-1"
                  />
                  <select
                    value={tokenIn?.address || ''}
                    onChange={(e) => {
                      const token = selectedPool.tokens.find(t => t.address === e.target.value)
                      setTokenIn(token || null)
                    }}
                    className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
                  >
                    <option value="">Select Token</option>
                    {selectedPool.tokens.map((token) => (
                      <option key={token.address} value={token.address}>
                        {token.symbol}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Swap Direction */}
              <div className="flex justify-center">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={swapTokens}
                  className="p-2 hover:bg-white/10 rounded-full"
                >
                  <ArrowUpDown className="w-5 h-5 text-gray-400" />
                </Button>
              </div>

              {/* Token Out */}
              <div className="p-4 bg-white/5 rounded-lg border border-white/10">
                <div className="flex items-center justify-between mb-2">
                  <label className="text-sm text-gray-400">To</label>
                  <span className="text-sm text-gray-400">
                    Balance: {tokenOut?.balance || '0'} {tokenOut?.symbol || ''}
                  </span>
                </div>
                <div className="flex items-center space-x-3">
                  <Input
                    placeholder="0.0"
                    value={amountOut}
                    readOnly
                    className="bg-transparent border-none text-2xl font-semibold text-white placeholder-gray-500 flex-1"
                  />
                  <select
                    value={tokenOut?.address || ''}
                    onChange={(e) => {
                      const token = selectedPool.tokens.find(t => t.address === e.target.value)
                      setTokenOut(token || null)
                    }}
                    className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
                  >
                    <option value="">Select Token</option>
                    {selectedPool.tokens.map((token) => (
                      <option key={token.address} value={token.address}>
                        {token.symbol}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Settings */}
              <div className="p-4 bg-white/5 rounded-lg border border-white/10">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-400">Slippage Tolerance</span>
                  <div className="flex items-center space-x-2">
                    {['0.1', '0.5', '1.0'].map((value) => (
                      <button
                        key={value}
                        onClick={() => setSlippageTolerance(value)}
                        className={`px-3 py-1 text-xs rounded ${
                          slippageTolerance === value
                            ? 'bg-purple-500 text-white'
                            : 'bg-white/10 text-gray-400 hover:bg-white/20'
                        }`}
                      >
                        {value}%
                      </button>
                    ))}
                    <Input
                      placeholder="Custom"
                      value={slippageTolerance}
                      onChange={(e) => setSlippageTolerance(e.target.value)}
                      className="w-16 h-6 text-xs bg-white/10 border-white/20"
                    />
                  </div>
                </div>
              </div>

              <Button
                onClick={executeSwap}
                disabled={!selectedRoute || isCalculating}
                className="w-full bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700 py-4 text-lg font-semibold"
              >
                {isCalculating ? (
                  <>
                    <RefreshCw className="w-5 h-5 mr-2 animate-spin" />
                    Calculating Routes...
                  </>
                ) : selectedRoute ? (
                  'Execute Toroidal Swap'
                ) : (
                  'Enter Amount'
                )}
              </Button>
            </div>
          </Card>
        </div>

        {/* Route Analysis */}
        <div className="space-y-6">
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
              <Route className="w-5 h-5 mr-2 text-green-400" />
              Route Analysis
            </h3>

            {routes.length > 0 ? (
              <div className="space-y-4">
                {routes.map((route, index) => (
                  <motion.div
                    key={index}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: index * 0.1 }}
                    onClick={() => setSelectedRoute(route)}
                    className={`p-4 rounded-lg border cursor-pointer transition-all ${
                      selectedRoute === route
                        ? 'border-green-400 bg-green-500/10'
                        : 'border-white/20 bg-white/5 hover:border-white/40'
                    }`}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center space-x-2">
                        {route.tokens.map((token, i) => (
                          <React.Fragment key={i}>
                            <span className="text-sm font-medium text-white">{token}</span>
                            {i < route.tokens.length - 1 && (
                              <span className="text-gray-400">→</span>
                            )}
                          </React.Fragment>
                        ))}
                      </div>
                      <Badge
                        variant={
                          route.toroidalMode === 'spherical' ? 'success' :
                          route.toroidalMode === 'hybrid' ? 'warning' : 'info'
                        }
                        className="text-xs"
                      >
                        {route.toroidalMode}
                      </Badge>
                    </div>

                    <div className="grid grid-cols-2 gap-2 text-xs text-gray-400">
                      <div>Impact: {route.priceImpact}%</div>
                      <div>Gas: {route.gas.toLocaleString()}</div>
                      <div>Sphere: {route.sphereUtilization}%</div>
                      <div className="flex items-center">
                        {index === 0 && <CheckCircle className="w-3 h-3 text-green-400 mr-1" />}
                        {index === 0 ? 'Best' : `+${(route.priceImpact - routes[0].priceImpact).toFixed(2)}%`}
                      </div>
                    </div>
                  </motion.div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-gray-400">
                <BarChart3 className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>Select tokens to see routes</p>
              </div>
            )}
          </Card>

          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-lg font-semibold text-white mb-4 flex items-center">
              <TrendingUp className="w-5 h-5 mr-2 text-blue-400" />
              Pool Metrics
            </h3>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-400">Sphere Radius²</span>
                <span className="text-sm font-medium text-white">
                  {selectedPool.sphereRadius.toFixed(2)}
                </span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-400">Concentrated Liquidity</span>
                <div className="flex items-center">
                  <div className="w-16 bg-blue-500/20 rounded-full h-2 mr-2">
                    <div 
                      className="bg-blue-400 h-2 rounded-full"
                      style={{ width: `${selectedPool.concentratedLiquidity}%` }}
                    />
                  </div>
                  <span className="text-sm font-medium text-white">
                    {selectedPool.concentratedLiquidity}%
                  </span>
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-400">24h Volume</span>
                <span className="text-sm font-medium text-white">{selectedPool.volume24h}</span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-gray-400">TVL</span>
                <span className="text-sm font-medium text-white">{selectedPool.totalValueLocked}</span>
              </div>

              {selectedRoute && (
                <div className="pt-4 border-t border-white/10">
                  <div className="text-sm font-medium text-white mb-2">Selected Route</div>
                  <div className="space-y-2 text-xs text-gray-400">
                    <div className="flex justify-between">
                      <span>Mode:</span>
                      <span className="capitalize text-white">{selectedRoute.toroidalMode}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Price Impact:</span>
                      <span className="text-white">{selectedRoute.priceImpact}%</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Sphere Usage:</span>
                      <span className="text-white">{selectedRoute.sphereUtilization}%</span>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}