'use client'

import React, { useState, useCallback } from 'react'
import { motion } from 'framer-motion'
import { 
  Plus, 
  Trash2, 
  CircleDot, 
  Settings, 
  Info, 
  CheckCircle,
  AlertTriangle,
  Loader2
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Badge } from '@/components/ui/Badge'

interface Token {
  address: string
  symbol: string
  name: string
  decimals: number
  initialReserve: string
}

interface PoolConfig {
  tokens: Token[]
  radiusSquared: string
  superellipseU: string
  concentratedLiquidity: boolean
  mevProtection: boolean
}

const SAMPLE_TOKENS = [
  { address: '0xA0b86a33E6441abDe0be74e1ca7bDED3cfF65e4c', symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  { address: '0xdAC17F958D2ee523a2206206994597C13D831ec7', symbol: 'USDT', name: 'Tether USD', decimals: 6 },
  { address: '0x6B175474E89094C44Da98b954EedeAC495271d0F', symbol: 'DAI', name: 'Dai Stablecoin', decimals: 18 },
  { address: '0x853d955aCEf822Db058eb8505911ED77F175b99e', symbol: 'FRAX', name: 'Frax', decimals: 18 },
  { address: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', symbol: 'WETH', name: 'Wrapped Ether', decimals: 18 },
  { address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599', symbol: 'WBTC', name: 'Wrapped BTC', decimals: 8 },
]

export function OrbitalPoolCreator() {
  const [poolConfig, setPoolConfig] = useState<PoolConfig>({
    tokens: [],
    radiusSquared: '',
    superellipseU: '2.5',
    concentratedLiquidity: true,
    mevProtection: true
  })
  
  const [isCreating, setIsCreating] = useState(false)
  const [sphereIntegrity, setSphereIntegrity] = useState<number | null>(null)
  const [validationError, setValidationError] = useState<string | null>(null)

  const addToken = useCallback(() => {
    if (poolConfig.tokens.length >= 1000) {
      setValidationError('Maximum 1000 tokens allowed per pool')
      return
    }
    
    setPoolConfig(prev => ({
      ...prev,
      tokens: [...prev.tokens, {
        address: '',
        symbol: '',
        name: '',
        decimals: 18,
        initialReserve: ''
      }]
    }))
    setValidationError(null)
  }, [poolConfig.tokens.length])

  const removeToken = useCallback((index: number) => {
    setPoolConfig(prev => ({
      ...prev,
      tokens: prev.tokens.filter((_, i) => i !== index)
    }))
    setSphereIntegrity(null)
  }, [])

  const updateToken = useCallback((index: number, updates: Partial<Token>) => {
    setPoolConfig(prev => ({
      ...prev,
      tokens: prev.tokens.map((token, i) => 
        i === index ? { ...token, ...updates } : token
      )
    }))
    setSphereIntegrity(null)
  }, [])

  const loadSampleToken = useCallback((index: number, sampleToken: typeof SAMPLE_TOKENS[0]) => {
    updateToken(index, sampleToken)
  }, [updateToken])

  const calculateSphereIntegrity = useCallback(() => {
    if (poolConfig.tokens.length < 3) {
      setValidationError('Minimum 3 tokens required for orbital pool')
      return
    }

    const reserves = poolConfig.tokens.map(token => {
      const reserve = parseFloat(token.initialReserve || '0')
      return reserve * Math.pow(10, token.decimals)
    })

    if (reserves.some(r => r <= 0)) {
      setValidationError('All tokens must have positive initial reserves')
      return
    }

    // Calculate sum of squares for sphere constraint
    const sumOfSquares = reserves.reduce((sum, r) => sum + r * r, 0)
    const calculatedRadius = Math.sqrt(sumOfSquares)
    
    // Update radius if not manually set
    if (!poolConfig.radiusSquared) {
      setPoolConfig(prev => ({
        ...prev,
        radiusSquared: sumOfSquares.toString()
      }))
    }

    // Calculate sphere integrity (how well reserves fit the sphere)
    const currentRadius = parseFloat(poolConfig.radiusSquared || sumOfSquares.toString())
    const integrity = Math.max(0, 100 - Math.abs(sumOfSquares - currentRadius) / currentRadius * 100)
    
    setSphereIntegrity(integrity)
    setValidationError(null)
  }, [poolConfig])

  const createPool = async () => {
    if (sphereIntegrity === null || sphereIntegrity < 95) {
      setValidationError('Sphere integrity must be at least 95% to create pool')
      return
    }

    setIsCreating(true)
    try {
      // Simulate pool creation
      await new Promise(resolve => setTimeout(resolve, 3000))
      
      // Reset form
      setPoolConfig({
        tokens: [],
        radiusSquared: '',
        superellipseU: '2.5',
        concentratedLiquidity: true,
        mevProtection: true
      })
      setSphereIntegrity(null)
      
    } catch (error) {
      setValidationError('Failed to create orbital pool')
    } finally {
      setIsCreating(false)
    }
  }

  return (
    <div className="space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">Create Orbital Pool</h2>
        <p className="text-gray-400 max-w-2xl mx-auto">
          Design and deploy N-dimensional liquidity pools with spherical constraints and concentrated liquidity.
          Support for 3-1000 tokens with unprecedented capital efficiency.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Token Configuration */}
        <div className="lg:col-span-2 space-y-6">
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-semibold text-white flex items-center">
                <CircleDot className="w-5 h-5 mr-2 text-purple-400" />
                Token Configuration
              </h3>
              <Badge variant={poolConfig.tokens.length >= 3 ? 'success' : 'warning'}>
                {poolConfig.tokens.length}/1000 tokens
              </Badge>
            </div>

            <div className="space-y-4">
              {poolConfig.tokens.map((token, index) => (
                <motion.div
                  key={index}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -20 }}
                  className="p-4 bg-white/5 rounded-lg border border-white/10"
                >
                  <div className="flex items-center justify-between mb-4">
                    <span className="text-sm font-medium text-gray-400">Token {index + 1}</span>
                    <div className="flex items-center space-x-2">
                      <div className="relative">
                        <select
                          onChange={(e) => {
                            const sampleToken = SAMPLE_TOKENS[parseInt(e.target.value)]
                            if (sampleToken) loadSampleToken(index, sampleToken)
                          }}
                          className="bg-white/5 border border-white/20 rounded px-2 py-1 text-xs text-white"
                        >
                          <option value="">Quick Add</option>
                          {SAMPLE_TOKENS.map((sample, i) => (
                            <option key={i} value={i}>{sample.symbol}</option>
                          ))}
                        </select>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removeToken(index)}
                        className="text-red-400 hover:text-red-300"
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <Input
                      placeholder="Token Address"
                      value={token.address}
                      onChange={(e) => updateToken(index, { address: e.target.value })}
                      className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                    />
                    <Input
                      placeholder="Symbol (e.g., USDC)"
                      value={token.symbol}
                      onChange={(e) => updateToken(index, { symbol: e.target.value })}
                      className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                    />
                    <Input
                      placeholder="Token Name"
                      value={token.name}
                      onChange={(e) => updateToken(index, { name: e.target.value })}
                      className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                    />
                    <Input
                      type="number"
                      placeholder="Decimals"
                      value={token.decimals}
                      onChange={(e) => updateToken(index, { decimals: parseInt(e.target.value) || 18 })}
                      className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                    />
                  </div>

                  <div className="mt-4">
                    <Input
                      placeholder="Initial Reserve Amount"
                      value={token.initialReserve}
                      onChange={(e) => updateToken(index, { initialReserve: e.target.value })}
                      className="bg-white/5 border-white/20 text-white placeholder-gray-400 w-full"
                    />
                  </div>
                </motion.div>
              ))}

              <Button
                onClick={addToken}
                variant="outline"
                className="w-full border-dashed border-purple-400/50 text-purple-400 hover:bg-purple-400/10"
              >
                <Plus className="w-4 h-4 mr-2" />
                Add Token {poolConfig.tokens.length < 3 && `(${3 - poolConfig.tokens.length} more required)`}
              </Button>
            </div>
          </Card>
        </div>

        {/* Pool Configuration & Validation */}
        <div className="space-y-6">
          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
              <Settings className="w-5 h-5 mr-2 text-blue-400" />
              Pool Settings
            </h3>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Radius² (Sphere Constraint)
                </label>
                <Input
                  placeholder="Auto-calculated"
                  value={poolConfig.radiusSquared}
                  onChange={(e) => setPoolConfig(prev => ({ ...prev, radiusSquared: e.target.value }))}
                  className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                />
                <p className="text-xs text-gray-400 mt-1">Σ(r²) = R² constraint parameter</p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Superellipse u-parameter
                </label>
                <Input
                  type="number"
                  step="0.1"
                  min="2"
                  max="10"
                  value={poolConfig.superellipseU}
                  onChange={(e) => setPoolConfig(prev => ({ ...prev, superellipseU: e.target.value }))}
                  className="bg-white/5 border-white/20 text-white placeholder-gray-400"
                />
                <p className="text-xs text-gray-400 mt-1">2.0 = sphere, &gt;2 = flatter curve</p>
              </div>

              <div className="space-y-3">
                <label className="flex items-center space-x-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={poolConfig.concentratedLiquidity}
                    onChange={(e) => setPoolConfig(prev => ({ ...prev, concentratedLiquidity: e.target.checked }))}
                    className="w-4 h-4 text-purple-400 bg-white/5 border-white/20 rounded focus:ring-purple-400"
                  />
                  <span className="text-gray-300">Enable Concentrated Liquidity</span>
                </label>

                <label className="flex items-center space-x-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={poolConfig.mevProtection}
                    onChange={(e) => setPoolConfig(prev => ({ ...prev, mevProtection: e.target.checked }))}
                    className="w-4 h-4 text-blue-400 bg-white/5 border-white/20 rounded focus:ring-blue-400"
                  />
                  <span className="text-gray-300">MEV Protection</span>
                </label>
              </div>
            </div>
          </Card>

          <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
            <h3 className="text-xl font-semibold text-white mb-6 flex items-center">
              <Info className="w-5 h-5 mr-2 text-green-400" />
              Pool Validation
            </h3>

            <div className="space-y-4">
              <Button
                onClick={calculateSphereIntegrity}
                variant="outline"
                className="w-full border-green-400/50 text-green-400 hover:bg-green-400/10"
                disabled={poolConfig.tokens.length < 3}
              >
                <CheckCircle className="w-4 h-4 mr-2" />
                Validate Sphere Constraint
              </Button>

              {sphereIntegrity !== null && (
                <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium text-green-400">Sphere Integrity</span>
                    <span className="text-lg font-bold text-green-400">{sphereIntegrity.toFixed(2)}%</span>
                  </div>
                  <div className="w-full bg-green-500/20 rounded-full h-2">
                    <div 
                      className="bg-green-400 h-2 rounded-full transition-all duration-500"
                      style={{ width: `${sphereIntegrity}%` }}
                    />
                  </div>
                  <p className="text-xs text-green-300 mt-2">
                    {sphereIntegrity >= 95 ? 'Pool ready for creation' : 'Adjust reserves to improve integrity'}
                  </p>
                </div>
              )}

              {validationError && (
                <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
                  <div className="flex items-center">
                    <AlertTriangle className="w-4 h-4 text-red-400 mr-2" />
                    <span className="text-sm text-red-400">{validationError}</span>
                  </div>
                </div>
              )}

              <Button
                onClick={createPool}
                className="w-full bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-700 hover:to-blue-700"
                disabled={isCreating || sphereIntegrity === null || sphereIntegrity < 95}
              >
                {isCreating ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Creating Pool...
                  </>
                ) : (
                  'Create Orbital Pool'
                )}
              </Button>
            </div>
          </Card>
        </div>
      </div>
    </div>
  )
}