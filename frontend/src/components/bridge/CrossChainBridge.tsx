'use client'

import React, { useState, useEffect, useCallback } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  ArrowRightLeft, 
  Clock, 
  AlertCircle, 
  CheckCircle, 
  ExternalLink,
  Loader2,
  Shield,
  Zap,
  Route,
  Info
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Badge } from '@/components/ui/Badge'
import { Modal } from '@/components/ui/Modal'
import { useCreateIntent, useIntent } from '@/hooks/useContracts'
import { useWallet } from '@/hooks/useWeb3'
import { formatUnits, parseUnits } from 'viem'
import toast from 'react-hot-toast'

interface ChainConfig {
  id: number
  name: string
  symbol: string
  rpc: string
  explorer: string
  icon: string
  color: string
}

interface Token {
  address: string
  symbol: string
  name: string
  decimals: number
  icon: string
  coingeckoId?: string
}

interface BridgeRoute {
  id: string
  fromChain: ChainConfig
  toChain: ChainConfig
  estimatedTime: string
  fee: string
  security: 'high' | 'medium' | 'low'
  type: 'native' | 'canonical' | 'third-party'
}

const SUPPORTED_CHAINS: ChainConfig[] = [
  {
    id: 1,
    name: 'Ethereum',
    symbol: 'ETH',
    rpc: 'https://eth-mainnet.g.alchemy.com/v2/your-api-key',
    explorer: 'https://etherscan.io',
    icon: '/icons/ethereum.svg',
    color: 'from-blue-400 to-blue-600'
  },
  {
    id: 17000,
    name: 'Holesky',
    symbol: 'ETH',
    rpc: 'https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/',
    explorer: 'https://holesky.etherscan.io',
    icon: '/icons/ethereum.svg',
    color: 'from-purple-400 to-purple-600'
  },
  {
    id: 137,
    name: 'Polygon',
    symbol: 'MATIC',
    rpc: 'https://polygon-rpc.com',
    explorer: 'https://polygonscan.com',
    icon: '/icons/polygon.svg',
    color: 'from-purple-400 to-indigo-600'
  },
  {
    id: 42161,
    name: 'Arbitrum',
    symbol: 'ETH',
    rpc: 'https://arb1.arbitrum.io/rpc',
    explorer: 'https://arbiscan.io',
    icon: '/icons/arbitrum.svg',
    color: 'from-blue-400 to-cyan-600'
  },
  {
    id: 10,
    name: 'Optimism',
    symbol: 'ETH',
    rpc: 'https://mainnet.optimism.io',
    explorer: 'https://optimistic.etherscan.io',
    icon: '/icons/optimism.svg',
    color: 'from-red-400 to-pink-600'
  }
]

const SUPPORTED_TOKENS: Record<number, Token[]> = {
  1: [
    { address: '0x0000000000000000000000000000000000000000', symbol: 'ETH', name: 'Ether', decimals: 18, icon: '/icons/ethereum.svg' },
    { address: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7', symbol: 'USDC', name: 'USD Coin', decimals: 6, icon: '/icons/usdc.svg' },
    { address: '0xdAC17F958D2ee523a2206206994597C13D831ec7', symbol: 'USDT', name: 'Tether', decimals: 6, icon: '/icons/usdt.svg' },
  ],
  17000: [
    { address: '0x0000000000000000000000000000000000000000', symbol: 'ETH', name: 'Ether', decimals: 18, icon: '/icons/ethereum.svg' },
    { address: '0x0000000000000000000000000000000000000000', symbol: 'USDC', name: 'Mock USDC', decimals: 6, icon: '/icons/usdc.svg' },
  ],
  137: [
    { address: '0x0000000000000000000000000000000000000000', symbol: 'MATIC', name: 'Polygon', decimals: 18, icon: '/icons/polygon.svg' },
    { address: '0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174', symbol: 'USDC', name: 'USD Coin', decimals: 6, icon: '/icons/usdc.svg' },
  ],
  42161: [
    { address: '0x0000000000000000000000000000000000000000', symbol: 'ETH', name: 'Ether', decimals: 18, icon: '/icons/ethereum.svg' },
    { address: '0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8', symbol: 'USDC', name: 'USD Coin', decimals: 6, icon: '/icons/usdc.svg' },
  ],
  10: [
    { address: '0x0000000000000000000000000000000000000000', symbol: 'ETH', name: 'Ether', decimals: 18, icon: '/icons/ethereum.svg' },
    { address: '0x7F5c764cBc14f9669B88837ca1490cCa17c31607', symbol: 'USDC', name: 'USD Coin', decimals: 6, icon: '/icons/usdc.svg' },
  ],
}

const BRIDGE_ROUTES: BridgeRoute[] = [
  {
    id: 'orbital-intent',
    fromChain: SUPPORTED_CHAINS[1], // Ethereum
    toChain: SUPPORTED_CHAINS[0], // Holesky
    estimatedTime: '2-5 min',
    fee: '0.1-0.3%',
    security: 'high',
    type: 'native'
  }
]

export function CrossChainBridge() {
  const { address, isConnected } = useWallet()
  const { createIntent, isLoading: isCreatingIntent, isSuccess, data: intentTxData } = useCreateIntent()
  
  const [fromChain, setFromChain] = useState<ChainConfig>(SUPPORTED_CHAINS[1]) // Ethereum
  const [toChain, setToChain] = useState<ChainConfig>(SUPPORTED_CHAINS[0]) // Holesky
  const [fromToken, setFromToken] = useState<Token | null>(null)
  const [toToken, setToToken] = useState<Token | null>(null)
  const [amount, setAmount] = useState('')
  const [estimatedOutput, setEstimatedOutput] = useState('')
  const [slippageTolerance, setSlippageTolerance] = useState('0.5')
  const [selectedRoute, setSelectedRoute] = useState<BridgeRoute | null>(null)
  const [showRouteModal, setShowRouteModal] = useState(false)
  const [showConfirmModal, setShowConfirmModal] = useState(false)
  const [currentIntent, setCurrentIntent] = useState<string | null>(null)

  // Load default tokens when chains change
  useEffect(() => {
    const fromTokens = SUPPORTED_TOKENS[fromChain.id] || []
    const toTokens = SUPPORTED_TOKENS[toChain.id] || []
    
    if (fromTokens.length > 0 && !fromToken) {
      setFromToken(fromTokens[0])
    }
    if (toTokens.length > 0 && !toToken) {
      setToToken(toTokens[0])
    }
  }, [fromChain.id, toChain.id, fromToken, toToken])

  // Calculate estimated output
  useEffect(() => {
    if (amount && fromToken && toToken && parseFloat(amount) > 0) {
      // Simple 1:1 conversion for demo - would use real price feeds in production
      const baseOutput = parseFloat(amount)
      const slippage = parseFloat(slippageTolerance) / 100
      const output = baseOutput * (1 - slippage)
      setEstimatedOutput(output.toFixed(6))
    } else {
      setEstimatedOutput('')
    }
  }, [amount, fromToken, toToken, slippageTolerance])

  const swapChains = () => {
    const tempChain = fromChain
    const tempToken = fromToken
    setFromChain(toChain)
    setToChain(tempChain)
    setFromToken(toToken)
    setToToken(tempToken)
    setAmount('')
    setEstimatedOutput('')
  }

  const handleBridge = async () => {
    if (!isConnected || !fromToken || !toToken || !amount) {
      toast.error('Please connect wallet and fill all fields')
      return
    }

    try {
      const sourceAmount = parseUnits(amount, fromToken.decimals)
      const minDestAmount = parseUnits(estimatedOutput, toToken.decimals)
      const deadline = Math.floor(Date.now() / 1000) + 30 * 60 // 30 minutes

      createIntent(
        fromChain.id,
        toChain.id,
        fromToken.address as `0x${string}`,
        toToken.address as `0x${string}`,
        sourceAmount.toString(),
        minDestAmount.toString(),
        deadline,
        '0x', // Additional data
        fromToken.address === '0x0000000000000000000000000000000000000000' ? sourceAmount : undefined
      )

      setShowConfirmModal(false)
      toast.success('Intent created successfully!')
    } catch (error) {
      console.error('Bridge error:', error)
      toast.error('Failed to create bridge intent')
    }
  }

  const RouteCard = ({ route }: { route: BridgeRoute }) => (
    <div 
      className={`p-4 rounded-lg border cursor-pointer transition-all ${
        selectedRoute?.id === route.id 
          ? 'border-blue-400 bg-blue-500/10' 
          : 'border-white/20 bg-white/5 hover:border-white/40'
      }`}
      onClick={() => setSelectedRoute(route)}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center space-x-2">
          <Badge variant="outline" className="text-xs">
            {route.type}
          </Badge>
          <Badge 
            variant={route.security === 'high' ? 'success' : route.security === 'medium' ? 'warning' : 'destructive'}
            className="text-xs"
          >
            {route.security} security
          </Badge>
        </div>
        <div className="flex items-center text-sm text-gray-400">
          <Clock className="w-4 h-4 mr-1" />
          {route.estimatedTime}
        </div>
      </div>
      
      <div className="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span className="text-gray-400">Fee:</span>
          <span className="ml-2 text-white">{route.fee}</span>
        </div>
        <div>
          <span className="text-gray-400">Type:</span>
          <span className="ml-2 text-white capitalize">{route.type}</span>
        </div>
      </div>
    </div>
  )

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">Cross-Chain Bridge</h2>
        <p className="text-gray-400 max-w-xl mx-auto">
          Bridge assets across different blockchains using intent-based execution with MEV protection.
        </p>
      </div>

      <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
        <div className="space-y-6">
          {/* From Chain */}
          <div className="space-y-3">
            <label className="text-sm font-medium text-gray-300">From</label>
            <div className="grid grid-cols-2 gap-4">
              <select
                value={fromChain.id}
                onChange={(e) => {
                  const chain = SUPPORTED_CHAINS.find(c => c.id === parseInt(e.target.value))
                  if (chain) setFromChain(chain)
                }}
                className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
              >
                {SUPPORTED_CHAINS.map((chain) => (
                  <option key={chain.id} value={chain.id}>
                    {chain.name}
                  </option>
                ))}
              </select>
              
              <select
                value={fromToken?.address || ''}
                onChange={(e) => {
                  const token = SUPPORTED_TOKENS[fromChain.id]?.find(t => t.address === e.target.value)
                  setFromToken(token || null)
                }}
                className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
              >
                <option value="">Select Token</option>
                {(SUPPORTED_TOKENS[fromChain.id] || []).map((token) => (
                  <option key={token.address} value={token.address}>
                    {token.symbol}
                  </option>
                ))}
              </select>
            </div>
            
            <div className="p-4 bg-white/5 rounded-lg border border-white/10">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-400">Amount</span>
                <span className="text-sm text-gray-400">
                  Balance: 0.00 {fromToken?.symbol || ''}
                </span>
              </div>
              <Input
                placeholder="0.0"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="bg-transparent border-none text-2xl font-semibold text-white placeholder-gray-500"
              />
            </div>
          </div>

          {/* Swap Direction */}
          <div className="flex justify-center">
            <Button
              variant="ghost"
              size="sm"
              onClick={swapChains}
              className="p-3 hover:bg-white/10 rounded-full"
            >
              <ArrowRightLeft className="w-5 h-5 text-gray-400" />
            </Button>
          </div>

          {/* To Chain */}
          <div className="space-y-3">
            <label className="text-sm font-medium text-gray-300">To</label>
            <div className="grid grid-cols-2 gap-4">
              <select
                value={toChain.id}
                onChange={(e) => {
                  const chain = SUPPORTED_CHAINS.find(c => c.id === parseInt(e.target.value))
                  if (chain) setToChain(chain)
                }}
                className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
              >
                {SUPPORTED_CHAINS.map((chain) => (
                  <option key={chain.id} value={chain.id}>
                    {chain.name}
                  </option>
                ))}
              </select>
              
              <select
                value={toToken?.address || ''}
                onChange={(e) => {
                  const token = SUPPORTED_TOKENS[toChain.id]?.find(t => t.address === e.target.value)
                  setToToken(token || null)
                }}
                className="bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white"
              >
                <option value="">Select Token</option>
                {(SUPPORTED_TOKENS[toChain.id] || []).map((token) => (
                  <option key={token.address} value={token.address}>
                    {token.symbol}
                  </option>
                ))}
              </select>
            </div>
            
            <div className="p-4 bg-white/5 rounded-lg border border-white/10">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-gray-400">You will receive</span>
                <span className="text-sm text-gray-400">
                  ~{estimatedOutput} {toToken?.symbol || ''}
                </span>
              </div>
              <div className="text-2xl font-semibold text-white">
                {estimatedOutput || '0.0'}
              </div>
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
                        ? 'bg-blue-500 text-white'
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

          {/* Route Selection */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-300">Route</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setShowRouteModal(true)}
                className="text-blue-400 hover:text-blue-300"
              >
                <Route className="w-4 h-4 mr-1" />
                View All Routes
              </Button>
            </div>
            
            {selectedRoute && (
              <div className="p-3 bg-green-500/10 border border-green-500/20 rounded-lg">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <Shield className="w-4 h-4 text-green-400" />
                    <span className="text-sm text-white">Orbital Intent Bridge</span>
                    <Badge variant="success" className="text-xs">
                      {selectedRoute.security}
                    </Badge>
                  </div>
                  <div className="text-sm text-gray-400">
                    Fee: {selectedRoute.fee}
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Bridge Button */}
          <Button
            onClick={() => setShowConfirmModal(true)}
            disabled={!isConnected || !amount || !fromToken || !toToken || isCreatingIntent}
            className="w-full bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 py-4 text-lg font-semibold"
          >
            {isCreatingIntent ? (
              <>
                <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                Creating Intent...
              </>
            ) : !isConnected ? (
              'Connect Wallet'
            ) : (
              'Bridge Assets'
            )}
          </Button>
        </div>
      </Card>

      {/* Route Selection Modal */}
      <Modal 
        isOpen={showRouteModal} 
        onClose={() => setShowRouteModal(false)}
        title="Select Bridge Route"
      >
        <div className="space-y-4">
          <div className="text-sm text-gray-400 mb-4">
            Choose the best route for your cross-chain transfer:
          </div>
          
          {BRIDGE_ROUTES.map((route) => (
            <RouteCard key={route.id} route={route} />
          ))}
          
          <div className="mt-6 flex justify-end space-x-3">
            <Button variant="outline" onClick={() => setShowRouteModal(false)}>
              Cancel
            </Button>
            <Button 
              onClick={() => setShowRouteModal(false)}
              disabled={!selectedRoute}
            >
              Select Route
            </Button>
          </div>
        </div>
      </Modal>

      {/* Confirmation Modal */}
      <Modal 
        isOpen={showConfirmModal} 
        onClose={() => setShowConfirmModal(false)}
        title="Confirm Bridge Transaction"
      >
        <div className="space-y-4">
          <div className="p-4 bg-yellow-500/10 border border-yellow-500/20 rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <Info className="w-4 h-4 text-yellow-400" />
              <span className="text-sm font-medium text-yellow-400">Review Details</span>
            </div>
            <p className="text-sm text-gray-300">
              You are bridging {amount} {fromToken?.symbol} from {fromChain.name} to {toChain.name}.
              This will create an intent that solvers will execute.
            </p>
          </div>

          <div className="space-y-3">
            <div className="flex justify-between">
              <span className="text-gray-400">From:</span>
              <span className="text-white">{amount} {fromToken?.symbol} on {fromChain.name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">To:</span>
              <span className="text-white">~{estimatedOutput} {toToken?.symbol} on {toChain.name}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Estimated Time:</span>
              <span className="text-white">{selectedRoute?.estimatedTime || '2-5 min'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Fee:</span>
              <span className="text-white">{selectedRoute?.fee || '0.1-0.3%'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Slippage:</span>
              <span className="text-white">{slippageTolerance}%</span>
            </div>
          </div>

          <div className="mt-6 flex justify-end space-x-3">
            <Button variant="outline" onClick={() => setShowConfirmModal(false)}>
              Cancel
            </Button>
            <Button 
              onClick={handleBridge}
              disabled={isCreatingIntent}
            >
              {isCreatingIntent ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Creating...
                </>
              ) : (
                'Confirm Bridge'
              )}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}