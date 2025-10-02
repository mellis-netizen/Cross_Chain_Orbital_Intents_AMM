'use client'

import { useState, useEffect } from 'react'
import { CheckCircle, XCircle, Clock, ExternalLink, Copy, AlertTriangle, Loader2 } from 'lucide-react'
import { Modal } from './Modal'
import { Button } from './Button'
import { Badge } from './Badge'
import { Progress } from './Progress'
import { useNetwork, useWaitForTransaction } from 'wagmi'
import { formatEther, formatUnits } from 'viem'
import { getExplorerUrl, copyToClipboard, truncateAddress } from '@/utils'
import { toast } from 'react-hot-toast'

export interface TransactionStep {
  id: string
  title: string
  description: string
  status: 'pending' | 'loading' | 'success' | 'error'
  txHash?: string
  errorMessage?: string
}

export interface TransactionModalProps {
  isOpen: boolean
  onClose: () => void
  title: string
  steps: TransactionStep[]
  currentStepIndex: number
  txHash?: string
  onRetry?: () => void
  showGasEstimate?: boolean
  gasEstimate?: {
    gasLimit: string
    gasPrice: string
    gasCost: string
  }
}

export function TransactionModal({
  isOpen,
  onClose,
  title,
  steps,
  currentStepIndex,
  txHash,
  onRetry,
  showGasEstimate = false,
  gasEstimate
}: TransactionModalProps) {
  const { chain } = useNetwork()
  const [copiedHash, setCopiedHash] = useState(false)

  // Monitor transaction status for the current transaction
  const { 
    data: txReceipt, 
    isError: txError, 
    isLoading: txLoading 
  } = useWaitForTransaction({
    hash: txHash as `0x${string}`,
    enabled: !!txHash,
  })

  const currentStep = steps[currentStepIndex]
  const isCompleted = currentStepIndex >= steps.length - 1 && currentStep?.status === 'success'
  const hasFailed = steps.some(step => step.status === 'error')
  const isProcessing = currentStep?.status === 'loading'

  // Calculate overall progress
  const completedSteps = steps.filter(step => step.status === 'success').length
  const progress = Math.round((completedSteps / steps.length) * 100)

  // Handle transaction hash copy
  const handleCopyHash = async (hash: string) => {
    const success = await copyToClipboard(hash)
    if (success) {
      setCopiedHash(true)
      toast.success('Transaction hash copied')
      setTimeout(() => setCopiedHash(false), 2000)
    } else {
      toast.error('Failed to copy transaction hash')
    }
  }

  // Handle view on explorer
  const handleViewOnExplorer = (hash: string) => {
    if (chain) {
      const url = getExplorerUrl(hash, 'tx')
      window.open(url, '_blank')
    }
  }

  // Update step status based on transaction receipt
  useEffect(() => {
    if (txReceipt && currentStep?.status === 'loading') {
      // Transaction confirmed - update step status in parent component
      console.log('Transaction confirmed:', txReceipt)
    }
    if (txError && currentStep?.status === 'loading') {
      // Transaction failed - update step status in parent component
      console.error('Transaction failed:', txError)
    }
  }, [txReceipt, txError, currentStep])

  const getStepIcon = (step: TransactionStep) => {
    switch (step.status) {
      case 'success':
        return <CheckCircle className="h-5 w-5 text-success-600" />
      case 'error':
        return <XCircle className="h-5 w-5 text-destructive" />
      case 'loading':
        return <Loader2 className="h-5 w-5 text-primary animate-spin" />
      default:
        return <Clock className="h-5 w-5 text-muted-foreground" />
    }
  }

  const getStatusBadge = () => {
    if (isCompleted) {
      return <Badge variant="success">Completed</Badge>
    }
    if (hasFailed) {
      return <Badge variant="destructive">Failed</Badge>
    }
    if (isProcessing) {
      return <Badge variant="info">Processing</Badge>
    }
    return <Badge variant="warning">Pending</Badge>
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={isCompleted || hasFailed ? onClose : undefined}
      title={title}
      size="md"
      showCloseButton={isCompleted || hasFailed}
    >
      <div className="space-y-6">
        {/* Status Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            {isCompleted ? (
              <div className="p-2 bg-success-100 rounded-full">
                <CheckCircle className="h-6 w-6 text-success-600" />
              </div>
            ) : hasFailed ? (
              <div className="p-2 bg-destructive/10 rounded-full">
                <XCircle className="h-6 w-6 text-destructive" />
              </div>
            ) : (
              <div className="p-2 bg-primary/10 rounded-full">
                <Loader2 className="h-6 w-6 text-primary animate-spin" />
              </div>
            )}
            <div>
              <h3 className="font-semibold text-lg">
                {isCompleted ? 'Transaction Completed' :
                 hasFailed ? 'Transaction Failed' :
                 'Processing Transaction'}
              </h3>
              <p className="text-sm text-muted-foreground">
                {isCompleted ? 'Your transaction has been confirmed on the blockchain' :
                 hasFailed ? 'There was an error processing your transaction' :
                 'Please wait while we process your transaction'}
              </p>
            </div>
          </div>
          {getStatusBadge()}
        </div>

        {/* Progress Bar */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Progress</span>
            <span className="font-medium">{progress}%</span>
          </div>
          <Progress value={progress} className="h-2" />
        </div>

        {/* Gas Estimate */}
        {showGasEstimate && gasEstimate && (
          <div className="p-3 bg-muted/20 rounded-lg space-y-2">
            <div className="text-sm font-medium text-muted-foreground">Gas Estimate</div>
            <div className="grid grid-cols-3 gap-4 text-xs">
              <div>
                <div className="text-muted-foreground">Gas Limit</div>
                <div className="font-medium">{parseInt(gasEstimate.gasLimit).toLocaleString()}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Gas Price</div>
                <div className="font-medium">{formatUnits(BigInt(gasEstimate.gasPrice), 9)} Gwei</div>
              </div>
              <div>
                <div className="text-muted-foreground">Total Cost</div>
                <div className="font-medium">{formatEther(BigInt(gasEstimate.gasCost))} ETH</div>
              </div>
            </div>
          </div>
        )}

        {/* Transaction Steps */}
        <div className="space-y-3">
          <div className="text-sm font-medium text-muted-foreground">Transaction Steps</div>
          {steps.map((step, index) => (
            <div
              key={step.id}
              className={`flex items-start space-x-3 p-3 rounded-lg border ${
                index === currentStepIndex
                  ? 'border-primary/20 bg-primary/5'
                  : step.status === 'success'
                  ? 'border-success-200 bg-success-50'
                  : step.status === 'error'
                  ? 'border-destructive/20 bg-destructive/5'
                  : 'border-border bg-muted/20'
              }`}
            >
              {getStepIcon(step)}
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <h4 className="text-sm font-medium">{step.title}</h4>
                  {step.txHash && (
                    <div className="flex items-center space-x-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleCopyHash(step.txHash!)}
                        className="h-6 w-6"
                      >
                        <Copy className="h-3 w-3" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleViewOnExplorer(step.txHash!)}
                        className="h-6 w-6"
                      >
                        <ExternalLink className="h-3 w-3" />
                      </Button>
                    </div>
                  )}
                </div>
                <p className="text-xs text-muted-foreground mt-1">{step.description}</p>
                {step.txHash && (
                  <div className="text-xs text-muted-foreground mt-1 font-mono">
                    Tx: {truncateAddress(step.txHash, 8, 6)}
                  </div>
                )}
                {step.status === 'error' && step.errorMessage && (
                  <div className="flex items-center space-x-1 mt-2 text-xs text-destructive">
                    <AlertTriangle className="h-3 w-3" />
                    <span>{step.errorMessage}</span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* Current Transaction Hash */}
        {txHash && (
          <div className="p-3 bg-muted/20 rounded-lg space-y-2">
            <div className="text-sm font-medium text-muted-foreground">Current Transaction</div>
            <div className="flex items-center justify-between">
              <code className="text-xs bg-muted px-2 py-1 rounded font-mono">
                {truncateAddress(txHash, 12, 8)}
              </code>
              <div className="flex items-center space-x-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleCopyHash(txHash)}
                  className="h-7"
                >
                  <Copy className="h-3 w-3 mr-1" />
                  {copiedHash ? 'Copied' : 'Copy'}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => handleViewOnExplorer(txHash)}
                  className="h-7"
                >
                  <ExternalLink className="h-3 w-3 mr-1" />
                  View
                </Button>
              </div>
            </div>
            {txLoading && (
              <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                <span>Waiting for confirmation...</span>
              </div>
            )}
          </div>
        )}

        {/* Action Buttons */}
        <div className="flex items-center justify-end space-x-3 pt-4 border-t">
          {hasFailed && onRetry && (
            <Button
              variant="outline"
              onClick={onRetry}
              className="flex items-center space-x-2"
            >
              <span>Retry Transaction</span>
            </Button>
          )}
          {(isCompleted || hasFailed) && (
            <Button
              variant="orbital"
              onClick={onClose}
            >
              {isCompleted ? 'Done' : 'Close'}
            </Button>
          )}
        </div>
      </div>
    </Modal>
  )
}

export default TransactionModal