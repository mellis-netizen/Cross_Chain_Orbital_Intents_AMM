'use client'

import { useState, useEffect, useCallback } from 'react'
import { ArrowRight, Clock, DollarSign, Shield, AlertTriangle, Zap, ChevronDown } from 'lucide-react'
import { Modal } from '@/components/ui/Modal'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Card, CardContent } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'
import { Progress } from '@/components/ui/Progress'
import { TokenSelector } from '@/components/swap/TokenSelector'
import { TransactionModal, TransactionStep } from '@/components/ui/TransactionModal'
import { useWallet, useTransactionStatus, useGasPrice } from '@/hooks/useWeb3'
import { useCreateIntent, useTokenBalance, useTokenAllowance, useTokenApprove } from '@/hooks/useContracts'
import { Token } from '@/types'
import { ETH_TOKEN, USDC_TOKEN, SUPPORTED_CHAINS } from '@/constants'
import { formatTokenAmount, parseTokenAmount, isValidAmount, formatCurrency } from '@/utils'
import { toast } from 'react-hot-toast'

export interface IntentCreationModalProps {
  isOpen: boolean
  onClose: () => void
  onSuccess?: (intentId: string) => void
}

interface IntentFormData {
  sourceChainId: number
  destChainId: number
  sourceToken: Token | null
  destToken: Token | null
  sourceAmount: string
  minDestAmount: string
  deadline: number
  maxFee: string
}

interface ChainOption {
  id: number
  name: string
  nativeCurrency: string
  icon: string
}

const CHAIN_OPTIONS: ChainOption[] = [
  { id: 1, name: 'Ethereum', nativeCurrency: 'ETH', icon: '🔷' },
  { id: 17000, name: 'Holesky', nativeCurrency: 'ETH', icon: '🧪' },
  { id: 137, name: 'Polygon', nativeCurrency: 'MATIC', icon: '🟣' },
  { id: 42161, name: 'Arbitrum', nativeCurrency: 'ETH', icon: '🔵' },
]

export function IntentCreationModal({ isOpen, onClose, onSuccess }: IntentCreationModalProps) {
  const [step, setStep] = useState<'form' | 'preview' | 'transaction'>('form')
  const [formData, setFormData] = useState<IntentFormData>({
    sourceChainId: 17000, // Default to Holesky
    destChainId: 1, // Default to Ethereum
    sourceToken: ETH_TOKEN,
    destToken: USDC_TOKEN,
    sourceAmount: '',
    minDestAmount: '',
    deadline: Math.floor(Date.now() / 1000) + 3600, // 1 hour from now
    maxFee: '0.01', // 1% default fee
  })
  const [priceEstimate, setPriceEstimate] = useState<string>('0')
  const [isEstimating, setIsEstimating] = useState(false)
  const [showTxModal, setShowTxModal] = useState(false)
  const [txSteps, setTxSteps] = useState<TransactionStep[]>([])
  const [currentStepIndex, setCurrentStepIndex] = useState(0)
  const [needsApproval, setNeedsApproval] = useState(false)

  const { address, isConnected, isCorrectNetwork } = useWallet()
  const { addPendingTx, confirmTx, failTx } = useTransactionStatus()
  const { gasPrice } = useGasPrice()

  // Get user's token balance
  const { balance: sourceBalance } = useTokenBalance(
    formData.sourceToken?.address as `0x${string}`,
    address
  )

  // Check token allowance for non-ETH tokens
  const { allowance, refetch: refetchAllowance } = useTokenAllowance(
    formData.sourceToken?.address === ETH_TOKEN.address ? undefined : formData.sourceToken?.address as `0x${string}`,
    address,
    '0x0000000000000000000000000000000000000000' as `0x${string}` // Intent engine address
  )

  const { approve, isLoading: approveLoading } = useTokenApprove()
  const { createIntent, isLoading: createLoading, isSuccess, data: intentTxData, error } = useCreateIntent()

  // Check if approval is needed
  useEffect(() => {
    if (
      formData.sourceToken?.address !== ETH_TOKEN.address && 
      formData.sourceAmount && 
      allowance !== undefined
    ) {
      const amountBigInt = parseTokenAmount(formData.sourceAmount, formData.sourceToken?.decimals)
      const allowanceBigInt = BigInt(allowance)
      setNeedsApproval(allowanceBigInt < amountBigInt)
    } else {
      setNeedsApproval(false)
    }
  }, [formData.sourceToken, formData.sourceAmount, allowance])

  // Estimate price when form data changes
  useEffect(() => {
    if (formData.sourceAmount && formData.sourceToken && formData.destToken) {
      estimatePrice()
    }
  }, [formData.sourceAmount, formData.sourceToken, formData.destToken, formData.sourceChainId, formData.destChainId])

  // Mock price estimation (in real implementation, this would call pricing APIs)
  const estimatePrice = useCallback(async () => {
    if (!formData.sourceAmount || !formData.sourceToken || !formData.destToken) return

    setIsEstimating(true)
    try {
      // Simulate API call delay
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Mock calculation: 1 ETH = 3000 USDC, with some slippage
      const sourceAmountNum = parseFloat(formData.sourceAmount)
      let estimatedAmount = 0

      if (formData.sourceToken.symbol === 'ETH' && formData.destToken.symbol === 'USDC') {
        estimatedAmount = sourceAmountNum * 3000 * 0.98 // 2% slippage
      } else if (formData.sourceToken.symbol === 'USDC' && formData.destToken.symbol === 'ETH') {
        estimatedAmount = sourceAmountNum / 3000 * 0.98
      } else {
        estimatedAmount = sourceAmountNum * 0.98 // Default 2% slippage
      }

      setPriceEstimate(estimatedAmount.toString())
      setFormData(prev => ({ ...prev, minDestAmount: (estimatedAmount * 0.95).toString() }))
    } catch (error) {
      console.error('Price estimation failed:', error)
    } finally {
      setIsEstimating(false)
    }
  }, [formData])

  // Handle form input changes
  const handleInputChange = useCallback((field: keyof IntentFormData, value: any) => {
    setFormData(prev => ({ ...prev, [field]: value }))
  }, [])

  // Handle chain selection
  const handleChainSelect = useCallback((type: 'source' | 'dest', chainId: number) => {
    if (type === 'source') {
      setFormData(prev => ({ ...prev, sourceChainId: chainId }))
    } else {
      setFormData(prev => ({ ...prev, destChainId: chainId }))
    }
  }, [])

  // Initialize transaction steps
  const initializeSteps = useCallback(() => {
    const steps: TransactionStep[] = []
    
    if (needsApproval) {
      steps.push({
        id: 'approve',
        title: 'Approve Token',
        description: `Allow the intent engine to spend your ${formData.sourceToken?.symbol}`,
        status: 'pending'
      })
    }
    
    steps.push({
      id: 'create-intent',
      title: 'Create Intent',
      description: `Create cross-chain intent for ${formData.sourceAmount} ${formData.sourceToken?.symbol}`,
      status: 'pending'
    })
    
    setTxSteps(steps)
    setCurrentStepIndex(0)
  }, [needsApproval, formData])

  // Handle token approval
  const handleApproval = useCallback(async () => {
    if (!formData.sourceToken || !formData.sourceAmount) return

    try {
      const amount = parseTokenAmount(formData.sourceAmount, formData.sourceToken.decimals).toString()
      const spenderAddress = '0x0000000000000000000000000000000000000000' as `0x${string}` // Intent engine
      
      setTxSteps(prev => prev.map((step, index) => 
        index === 0 ? { ...step, status: 'loading' } : step
      ))

      approve(formData.sourceToken.address as `0x${string}`, spenderAddress, amount)
    } catch (error) {
      console.error('Approval failed:', error)
      
      setTxSteps(prev => prev.map((step, index) => 
        index === 0 ? { 
          ...step, 
          status: 'error', 
          errorMessage: 'Token approval failed'
        } : step
      ))
      
      toast.error('Token approval failed')
    }
  }, [formData.sourceToken, formData.sourceAmount, approve])

  // Handle intent creation
  const handleCreateIntent = useCallback(async () => {
    if (!isConnected || !isCorrectNetwork) {
      toast.error('Please connect your wallet and switch to the correct network')
      return
    }

    if (!formData.sourceToken || !formData.destToken || !formData.sourceAmount) {
      toast.error('Please fill in all required fields')
      return
    }

    // Show transaction modal
    initializeSteps()
    setShowTxModal(true)
    setStep('transaction')

    try {
      // Handle approval if needed
      if (needsApproval) {
        await handleApproval()
        return // Approval will trigger intent creation in useEffect
      }

      // Create intent directly
      const sourceAmount = parseTokenAmount(formData.sourceAmount, formData.sourceToken.decimals).toString()
      const minDestAmount = parseTokenAmount(formData.minDestAmount, formData.destToken.decimals).toString()
      
      const intentStepIndex = needsApproval ? 1 : 0
      setCurrentStepIndex(intentStepIndex)
      setTxSteps(prev => prev.map((step, index) => 
        index === intentStepIndex ? { ...step, status: 'loading' } : step
      ))

      const value = formData.sourceToken.address === ETH_TOKEN.address 
        ? parseTokenAmount(formData.sourceAmount, formData.sourceToken.decimals)
        : undefined

      createIntent(
        formData.sourceChainId,
        formData.destChainId,
        formData.sourceToken.address as `0x${string}`,
        formData.destToken.address as `0x${string}`,
        sourceAmount,
        minDestAmount,
        formData.deadline,
        '0x', // Additional data
        value
      )
    } catch (error) {
      console.error('Intent creation failed:', error)
      
      const intentStepIndex = needsApproval ? 1 : 0
      setTxSteps(prev => prev.map((step, index) => 
        index === intentStepIndex ? { 
          ...step, 
          status: 'error', 
          errorMessage: 'Intent creation failed'
        } : step
      ))
      
      toast.error('Intent creation failed')
    }
  }, [formData, isConnected, isCorrectNetwork, needsApproval, createIntent, initializeSteps, handleApproval])

  // Handle successful intent creation
  useEffect(() => {
    if (isSuccess && intentTxData?.hash) {
      const intentStepIndex = needsApproval ? 1 : 0
      
      setTxSteps(prev => prev.map((step, index) => 
        index === intentStepIndex ? { 
          ...step, 
          status: 'success',
          txHash: intentTxData.hash 
        } : step
      ))
      
      addPendingTx(intentTxData.hash)
      confirmTx(intentTxData.hash)
      toast.success('Intent created successfully!')
      
      // Call success callback with mock intent ID
      const intentId = intentTxData.hash
      onSuccess?.(intentId)
      
      // Close modal after delay
      setTimeout(() => {
        handleClose()
      }, 3000)
    }
  }, [isSuccess, intentTxData, needsApproval, addPendingTx, confirmTx, onSuccess])

  // Handle error
  useEffect(() => {
    if (error) {
      const intentStepIndex = needsApproval ? 1 : 0
      setTxSteps(prev => prev.map((step, index) => 
        index === intentStepIndex ? { 
          ...step, 
          status: 'error', 
          errorMessage: error.message || 'Intent creation failed'
        } : step
      ))
      toast.error(error.message || 'Intent creation failed')
    }
  }, [error, needsApproval])

  // Reset form and close modal
  const handleClose = useCallback(() => {
    setStep('form')
    setShowTxModal(false)
    setTxSteps([])
    setCurrentStepIndex(0)
    onClose()
  }, [onClose])

  // Form validation
  const isFormValid = formData.sourceToken && 
    formData.destToken && 
    formData.sourceAmount && 
    isValidAmount(formData.sourceAmount) &&
    formData.minDestAmount &&
    formData.sourceChainId !== formData.destChainId

  // Calculate fees and estimates
  const estimatedFee = formData.sourceAmount ? 
    (parseFloat(formData.sourceAmount) * parseFloat(formData.maxFee)).toString() : '0'
  
  const gasEstimate = gasPrice ? {
    gasLimit: '200000', // Estimated gas limit for intent creation
    gasPrice,
    gasCost: (BigInt('200000') * BigInt(gasPrice)).toString()
  } : undefined

  const renderChainSelector = (label: string, selectedChainId: number, onChange: (chainId: number) => void) => (
    <div className="space-y-2">
      <label className="text-sm font-medium text-muted-foreground">{label}</label>
      <div className="relative">
        <select
          value={selectedChainId}
          onChange={(e) => onChange(parseInt(e.target.value))}
          className="w-full p-3 pr-10 border rounded-lg bg-background text-foreground appearance-none focus:ring-2 focus:ring-primary focus:border-transparent"
        >
          {CHAIN_OPTIONS.map((chain) => (
            <option key={chain.id} value={chain.id}>
              {chain.icon} {chain.name}
            </option>
          ))}
        </select>
        <ChevronDown className="absolute right-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
      </div>
    </div>
  )

  return (
    <>
      <Modal
        isOpen={isOpen && !showTxModal}
        onClose={handleClose}
        title="Create Cross-Chain Intent"
        size="lg"
      >
        <div className="space-y-6">
          {step === 'form' && (
            <>
              {/* Chain Selection */}
              <div className="grid grid-cols-2 gap-4">
                {renderChainSelector(
                  'Source Chain', 
                  formData.sourceChainId, 
                  (chainId) => handleChainSelect('source', chainId)
                )}
                {renderChainSelector(
                  'Destination Chain', 
                  formData.destChainId, 
                  (chainId) => handleChainSelect('dest', chainId)
                )}
              </div>

              {/* Token Selection */}
              <div className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-muted-foreground">From Token</label>
                  <div className="flex space-x-2">
                    <Input
                      type="number"
                      placeholder="0.0"
                      value={formData.sourceAmount}
                      onChange={(e) => handleInputChange('sourceAmount', e.target.value)}
                      className="flex-1"
                    />
                    <TokenSelector
                      selectedToken={formData.sourceToken}
                      onTokenSelect={(token) => handleInputChange('sourceToken', token)}
                    />
                  </div>
                  {sourceBalance && formData.sourceToken && (
                    <div className="text-xs text-muted-foreground">
                      Balance: {formatTokenAmount(sourceBalance, formData.sourceToken.decimals)} {formData.sourceToken.symbol}
                    </div>
                  )}
                </div>

                <div className="flex justify-center">
                  <div className="p-2 border rounded-full bg-muted/20">
                    <ArrowRight className="h-4 w-4 text-muted-foreground" />
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-muted-foreground">To Token (Minimum)</label>
                  <div className="flex space-x-2">
                    <Input
                      type="number"
                      placeholder="0.0"
                      value={formData.minDestAmount}
                      onChange={(e) => handleInputChange('minDestAmount', e.target.value)}
                      className="flex-1"
                    />
                    <TokenSelector
                      selectedToken={formData.destToken}
                      onTokenSelect={(token) => handleInputChange('destToken', token)}
                    />
                  </div>
                  {isEstimating ? (
                    <div className="text-xs text-muted-foreground">Estimating price...</div>
                  ) : priceEstimate !== '0' ? (
                    <div className="text-xs text-muted-foreground">
                      Estimated: {formatTokenAmount(priceEstimate, formData.destToken?.decimals)} {formData.destToken?.symbol}
                    </div>
                  ) : null}
                </div>
              </div>

              {/* Advanced Settings */}
              <Card>
                <CardContent className="p-4 space-y-4">
                  <h4 className="font-medium">Advanced Settings</h4>
                  
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-muted-foreground">Max Fee (%)</label>
                      <Input
                        type="number"
                        step="0.01"
                        min="0"
                        max="5"
                        value={formData.maxFee}
                        onChange={(e) => handleInputChange('maxFee', e.target.value)}
                      />
                    </div>
                    
                    <div className="space-y-2">
                      <label className="text-sm font-medium text-muted-foreground">Deadline (hours)</label>
                      <Input
                        type="number"
                        min="1"
                        max="24"
                        value={Math.round((formData.deadline - Date.now() / 1000) / 3600)}
                        onChange={(e) => handleInputChange('deadline', Math.floor(Date.now() / 1000) + parseInt(e.target.value) * 3600)}
                      />
                    </div>
                  </div>
                </CardContent>
              </Card>

              {/* Action Buttons */}
              <div className="flex items-center justify-end space-x-3 pt-4 border-t">
                <Button variant="outline" onClick={handleClose}>
                  Cancel
                </Button>
                <Button
                  variant="orbital"
                  onClick={() => setStep('preview')}
                  disabled={!isFormValid}
                >
                  Review Intent
                </Button>
              </div>
            </>
          )}

          {step === 'preview' && (
            <>
              {/* Intent Summary */}
              <Card>
                <CardContent className="p-6">
                  <h3 className="font-semibold text-lg mb-4">Intent Summary</h3>
                  
                  <div className="space-y-4">
                    {/* Route */}
                    <div className="flex items-center justify-between p-4 bg-muted/20 rounded-lg">
                      <div className="flex items-center space-x-3">
                        <div className="text-center">
                          <div className="font-medium">{CHAIN_OPTIONS.find(c => c.id === formData.sourceChainId)?.icon}</div>
                          <div className="text-xs text-muted-foreground">{CHAIN_OPTIONS.find(c => c.id === formData.sourceChainId)?.name}</div>
                        </div>
                        <ArrowRight className="h-5 w-5 text-muted-foreground" />
                        <div className="text-center">
                          <div className="font-medium">{CHAIN_OPTIONS.find(c => c.id === formData.destChainId)?.icon}</div>
                          <div className="text-xs text-muted-foreground">{CHAIN_OPTIONS.find(c => c.id === formData.destChainId)?.name}</div>
                        </div>
                      </div>
                      <Badge variant="info">
                        <Zap className="h-3 w-3 mr-1" />
                        Cross-Chain
                      </Badge>
                    </div>

                    {/* Amounts */}
                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-3 border rounded-lg">
                        <div className="text-sm text-muted-foreground">You Send</div>
                        <div className="font-semibold text-lg">
                          {formData.sourceAmount} {formData.sourceToken?.symbol}
                        </div>
                      </div>
                      <div className="p-3 border rounded-lg">
                        <div className="text-sm text-muted-foreground">You Receive (Min)</div>
                        <div className="font-semibold text-lg">
                          {formData.minDestAmount} {formData.destToken?.symbol}
                        </div>
                      </div>
                    </div>

                    {/* Fees & Details */}
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Estimated Fee</span>
                        <span>{estimatedFee} {formData.sourceToken?.symbol} ({formData.maxFee}%)</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Deadline</span>
                        <span>{new Date(formData.deadline * 1000).toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Status</span>
                        <Badge variant="warning">
                          <Clock className="h-3 w-3 mr-1" />
                          Pending
                        </Badge>
                      </div>
                    </div>

                    {/* Warnings */}
                    {needsApproval && (
                      <div className="flex items-center space-x-2 p-3 bg-warning-50 border border-warning-200 rounded-lg">
                        <AlertTriangle className="h-4 w-4 text-warning-600" />
                        <span className="text-sm text-warning-700">
                          This transaction requires token approval before creating the intent.
                        </span>
                      </div>
                    )}
                  </div>
                </CardContent>
              </Card>

              {/* Action Buttons */}
              <div className="flex items-center justify-between pt-4 border-t">
                <Button variant="outline" onClick={() => setStep('form')}>
                  Back to Edit
                </Button>
                <Button
                  variant="orbital"
                  onClick={handleCreateIntent}
                  disabled={!isConnected || !isCorrectNetwork}
                  loading={createLoading || approveLoading}
                >
                  {!isConnected ? 'Connect Wallet' :
                   !isCorrectNetwork ? 'Wrong Network' :
                   needsApproval ? 'Approve & Create Intent' :
                   'Create Intent'}
                </Button>
              </div>
            </>
          )}
        </div>
      </Modal>

      {/* Transaction Modal */}
      <TransactionModal
        isOpen={showTxModal}
        onClose={handleClose}
        title="Create Cross-Chain Intent"
        steps={txSteps}
        currentStepIndex={currentStepIndex}
        txHash={intentTxData?.hash}
        onRetry={() => {
          setShowTxModal(false)
          setTimeout(() => handleCreateIntent(), 500)
        }}
        showGasEstimate={true}
        gasEstimate={gasEstimate}
      />
    </>
  )
}

export default IntentCreationModal