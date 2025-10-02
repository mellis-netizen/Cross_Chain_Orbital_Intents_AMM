'use client'

import { useState, useEffect, useCallback } from 'react'
import { ArrowUpDown, Settings, Info, Zap } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Badge } from '@/components/ui/Badge'
import { TokenSelector } from './TokenSelector'
import { SwapSettings } from './SwapSettings'
import { PriceImpactWarning } from './PriceImpactWarning'
import { useWallet, useTokenBalances, useGasPrice } from '@/hooks/useWeb3'
import { useSwapQuote, usePoolByTokens, useSwap, useTokenApprove, useTokenAllowance } from '@/hooks/useContracts'
import useTransactions from '@/hooks/useTransactions'
import useTransactionMonitor from '@/hooks/useTransactionMonitor'
import { Token, SwapFormData } from '@/types'
import { ETH_TOKEN, USDC_TOKEN, DEFAULT_SLIPPAGE } from '@/constants'
import { formatTokenAmount, parseTokenAmount, calculatePriceImpact, isValidAmount } from '@/utils'
import { toast } from 'react-hot-toast'

export function SwapInterface() {
  const [formData, setFormData] = useState<SwapFormData>({
    fromToken: ETH_TOKEN,
    toToken: USDC_TOKEN,
    fromAmount: '',
    slippage: DEFAULT_SLIPPAGE,
  })
  const [showSettings, setShowSettings] = useState(false)
  const [isReversed, setIsReversed] = useState(false)
  const [needsApproval, setNeedsApproval] = useState(false)
  const [isCheckingApproval, setIsCheckingApproval] = useState(false)

  const { address, isConnected, isCorrectNetwork } = useWallet()
  const { gasPrice, gasPriceGwei } = useGasPrice()
  const { estimateGas } = useTransactions()
  const { addPendingTransaction } = useTransactionMonitor()
  
  // Get pool information
  const { 
    poolId, 
    pool, 
    isLoading: poolLoading 
  } = usePoolByTokens(
    formData.fromToken?.address || '0x0',
    formData.toToken?.address || '0x0'
  )

  // Get swap quote
  const {
    amountOut,
    isLoading: quoteLoading,
    refetch: refetchQuote
  } = useSwapQuote(
    poolId || '0',
    !isReversed, // zeroForOne
    parseTokenAmount(formData.fromAmount, formData.fromToken?.decimals).toString(),
    !!formData.fromAmount && !!poolId && isValidAmount(formData.fromAmount)
  )

  // Swap execution
  const { swap, isLoading: swapLoading, isSuccess, error } = useSwap()
  const { approve, isLoading: approveLoading } = useTokenApprove()
  
  // Token balances
  const { balances } = useTokenBalances(
    address,
    [formData.fromToken?.address, formData.toToken?.address].filter(Boolean) as string[]
  )
  
  // Token allowance for approval checking
  const { allowance, refetch: refetchAllowance } = useTokenAllowance(
    formData.fromToken?.address || '0x0',
    address,
    '0x0000000000000000000000000000000000000000' // This would be the AMM contract address
  )

  // Calculate price impact
  const priceImpact = pool && formData.fromAmount && amountOut ? 
    calculatePriceImpact(
      parseTokenAmount(formData.fromAmount, formData.fromToken?.decimals).toString(),
      amountOut,
      pool.reserve0,
      pool.reserve1
    ) : 0

  const isHighPriceImpact = priceImpact > 0.05 // 5%

  // Handle token selection
  const handleTokenSelect = useCallback((token: Token, type: 'from' | 'to') => {
    setFormData(prev => ({
      ...prev,
      [type === 'from' ? 'fromToken' : 'toToken']: token
    }))
  }, [])

  // Handle amount input
  const handleAmountChange = useCallback((value: string) => {
    setFormData(prev => ({ ...prev, fromAmount: value }))
  }, [])

  // Handle token swap (reverse)
  const handleSwapTokens = useCallback(() => {
    setFormData(prev => ({
      ...prev,
      fromToken: prev.toToken,
      toToken: prev.fromToken,
      fromAmount: formatTokenAmount(amountOut, prev.toToken?.decimals),
    }))
    setIsReversed(!isReversed)
  }, [amountOut, isReversed])

  // Handle settings update
  const handleSettingsUpdate = useCallback((settings: { slippage: number }) => {
    setFormData(prev => ({ ...prev, slippage: settings.slippage }))
    setShowSettings(false)
  }, [])

  // Check if approval is needed
  const checkApprovalNeeded = useCallback(async () => {
    if (!formData.fromToken || !formData.fromAmount || !address || 
        formData.fromToken.address === ETH_TOKEN.address) {
      setNeedsApproval(false)
      return
    }

    setIsCheckingApproval(true)
    try {
      const amountToSwap = parseTokenAmount(formData.fromAmount, formData.fromToken.decimals)
      const currentAllowance = BigInt(allowance)
      setNeedsApproval(currentAllowance < amountToSwap)
    } catch (error) {
      console.error('Error checking approval:', error)
      setNeedsApproval(false)
    } finally {
      setIsCheckingApproval(false)
    }
  }, [formData.fromToken, formData.fromAmount, address, allowance])

  // Handle token approval
  const handleApproval = useCallback(async () => {
    if (!formData.fromToken || !formData.fromAmount || !address) {
      toast.error('Missing approval parameters')
      return
    }

    try {
      const amountToApprove = parseTokenAmount(formData.fromAmount, formData.fromToken.decimals)
      const spenderAddress = '0x0000000000000000000000000000000000000000' // AMM contract address
      
      toast.loading('Requesting token approval...')
      
      const hash = await approve(
        formData.fromToken.address,
        spenderAddress,
        amountToApprove.toString()
      )
      
      addPendingTransaction(hash, 'approval', `Approve ${formData.fromToken.symbol}`, 1)
      
      // Refresh allowance after approval
      setTimeout(() => {
        refetchAllowance()
        checkApprovalNeeded()
      }, 3000)
      
    } catch (error) {
      console.error('Approval failed:', error)
      toast.error('Token approval failed')
    }
  }, [formData.fromToken, formData.fromAmount, address, approve, addPendingTransaction, refetchAllowance, checkApprovalNeeded])

  // Execute swap with enhanced error handling and gas estimation
  const handleSwap = useCallback(async () => {
    if (!isConnected || !isCorrectNetwork || !poolId || !formData.fromAmount) {
      return
    }

    try {
      // Show loading toast
      toast.loading('Preparing swap transaction...')
      
      const minAmountOut = BigInt(amountOut) * BigInt(10000 - formData.slippage * 100) / BigInt(10000)
      
      const swapParams = {
        poolId,
        zeroForOne: !isReversed,
        amountIn: parseTokenAmount(formData.fromAmount, formData.fromToken?.decimals).toString(),
        minAmountOut: minAmountOut.toString(),
      }

      const value = formData.fromToken?.address === ETH_TOKEN.address 
        ? parseTokenAmount(formData.fromAmount, formData.fromToken.decimals)
        : undefined

      // Estimate gas before execution
      try {
        await estimateGas({
          to: '0x0000000000000000000000000000000000000000', // AMM contract address
          data: '0x',
          value: value || 0n,
        })
      } catch (gasError) {
        toast.error('Transaction would fail. Please check your parameters.')
        return
      }

      const hash = await swap(swapParams, value)
      
      if (hash) {
        addPendingTransaction(hash, 'swap', `Swap ${formData.fromToken?.symbol} for ${formData.toToken?.symbol}`, 2)
      }
      
    } catch (error) {
      console.error('Swap failed:', error)
      const errorMessage = error instanceof Error ? error.message : 'Swap failed. Please try again.'
      toast.error(errorMessage)
    }
  }, [
    isConnected,
    isCorrectNetwork,
    poolId,
    formData,
    amountOut,
    isReversed,
    swap,
    estimateGas,
    addPendingTransaction
  ])

  // Show success toast when swap completes
  useEffect(() => {
    if (isSuccess) {
      toast.success('Swap completed successfully!')
      setFormData(prev => ({ ...prev, fromAmount: '' }))
    }
  }, [isSuccess])

  // Check approval when form data changes
  useEffect(() => {
    checkApprovalNeeded()
  }, [checkApprovalNeeded])

  // Show error toast
  useEffect(() => {
    if (error) {
      toast.error(error.message || 'Swap failed')
    }
  }, [error])

  const canSwap = isConnected && 
    isCorrectNetwork && 
    formData.fromToken && 
    formData.toToken && 
    formData.fromAmount && 
    isValidAmount(formData.fromAmount) &&
    poolId &&
    amountOut !== '0' &&
    !needsApproval

  const outputAmount = formatTokenAmount(amountOut, formData.toToken?.decimals)

  return (
    <div className="w-full max-w-md mx-auto">
      <Card className="border-border/50 shadow-xl">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
          <CardTitle className="text-xl font-semibold">Swap</CardTitle>
          <div className="flex items-center space-x-2">
            <Badge variant="info" className="flex items-center space-x-1">
              <Zap className="h-3 w-3" />
              <span>Orbital AMM</span>
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setShowSettings(true)}
            >
              <Settings className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* From Token */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium text-muted-foreground">From</label>
              {formData.fromToken && (
                <div className="text-xs text-muted-foreground">
                  Balance: {formatTokenAmount(
                    balances[formData.fromToken.address] || '0',
                    formData.fromToken.decimals
                  )} {formData.fromToken.symbol}
                </div>
              )}
            </div>
            <div className="flex space-x-2">
              <div className="flex-1">
                <Input
                  type="number"
                  placeholder="0.0"
                  value={formData.fromAmount}
                  onChange={(e) => handleAmountChange(e.target.value)}
                  className="text-lg font-semibold border-0 bg-muted/20 focus-visible:ring-0"
                />
              </div>
              <TokenSelector
                selectedToken={formData.fromToken}
                onTokenSelect={(token) => handleTokenSelect(token, 'from')}
              />
            </div>
          </div>

          {/* Swap Direction Button */}
          <div className="flex justify-center">
            <Button
              variant="ghost"
              size="icon"
              onClick={handleSwapTokens}
              className="rounded-full border bg-background hover:bg-muted"
            >
              <ArrowUpDown className="h-4 w-4" />
            </Button>
          </div>

          {/* To Token */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-sm font-medium text-muted-foreground">To</label>
              {formData.toToken && (
                <div className="text-xs text-muted-foreground">
                  Balance: {formatTokenAmount(
                    balances[formData.toToken.address] || '0',
                    formData.toToken.decimals
                  )} {formData.toToken.symbol}
                </div>
              )}
            </div>
            <div className="flex space-x-2">
              <div className="flex-1">
                <Input
                  type="number"
                  placeholder="0.0"
                  value={outputAmount}
                  readOnly
                  className="text-lg font-semibold border-0 bg-muted/20 text-muted-foreground"
                />
              </div>
              <TokenSelector
                selectedToken={formData.toToken}
                onTokenSelect={(token) => handleTokenSelect(token, 'to')}
              />
            </div>
          </div>

          {/* Price Impact Warning */}
          {isHighPriceImpact && (
            <PriceImpactWarning priceImpact={priceImpact} />
          )}

          {/* Swap Details */}
          {formData.fromAmount && amountOut !== '0' && (
            <div className="p-3 bg-muted/20 rounded-lg space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Price Impact</span>
                <span className={priceImpact > 0.03 ? 'text-warning-600' : 'text-muted-foreground'}>
                  {(priceImpact * 100).toFixed(2)}%
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Slippage Tolerance</span>
                <span>{formData.slippage}%</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Network Fee</span>
                <span>{gasPriceGwei} gwei</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Minimum Received</span>
                <span>
                  {formatTokenAmount(
                    (BigInt(amountOut) * BigInt(10000 - formData.slippage * 100) / BigInt(10000)).toString(),
                    formData.toToken?.decimals
                  )} {formData.toToken?.symbol}
                </span>
              </div>
              {poolLoading && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Pool</span>
                  <span>Loading...</span>
                </div>
              )}
            </div>
          )}

          {/* Approval/Swap Buttons */}
          {needsApproval && formData.fromToken?.address !== ETH_TOKEN.address ? (
            <Button
              onClick={handleApproval}
              disabled={!isConnected || !isCorrectNetwork || approveLoading || isCheckingApproval}
              loading={approveLoading || isCheckingApproval}
              variant="orbital"
              size="lg"
              className="w-full"
            >
              {approveLoading ? 'Approving...' :
               isCheckingApproval ? 'Checking...' :
               `Approve ${formData.fromToken?.symbol}`}
            </Button>
          ) : (
            <Button
              onClick={handleSwap}
              disabled={!canSwap || needsApproval}
              loading={swapLoading || quoteLoading}
              variant="orbital"
              size="lg"
              className="w-full"
            >
              {!isConnected ? 'Connect Wallet' :
               !isCorrectNetwork ? 'Wrong Network' :
               !formData.fromToken || !formData.toToken ? 'Select Tokens' :
               !formData.fromAmount ? 'Enter Amount' :
               !isValidAmount(formData.fromAmount) ? 'Invalid Amount' :
               !poolId ? 'Pool Not Found' :
               needsApproval ? 'Approval Required' :
               isHighPriceImpact ? 'Swap Anyway' :
               'Swap'}
            </Button>
          )}

          {/* Pool Status */}
          {poolId && pool && (
            <div className="text-xs text-center text-muted-foreground">
              Pool: {formData.fromToken?.symbol}/{formData.toToken?.symbol} • 
              TVL: ${formatTokenAmount(pool.reserve0, formData.fromToken?.decimals)} • 
              Fee: 0.3%
            </div>
          )}
        </CardContent>
      </Card>

      {/* Settings Modal */}
      <SwapSettings
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
        currentSlippage={formData.slippage}
        onUpdate={handleSettingsUpdate}
      />

      {/* Transaction Modal */}
      <TransactionModal
        isOpen={showTxModal}
        onClose={handleCloseModal}
        title="Swap Transaction"
        steps={txSteps}
        currentStepIndex={currentStepIndex}
        txHash={swapTxData?.hash}
        onRetry={handleRetry}
        showGasEstimate={true}
        gasEstimate={gasEstimate}
      />
    </div>
  )
}

export default SwapInterface