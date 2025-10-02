'use client'

import React, { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  CheckCircle, 
  XCircle, 
  Clock, 
  Loader2, 
  ExternalLink,
  AlertTriangle,
  Copy,
  X,
  ChevronDown,
  ChevronUp,
  Zap,
  Shield
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { Progress } from '@/components/ui/Progress'
import { Hash, formatEther, formatGwei } from 'viem'
import useTransactionMonitor from '@/hooks/useTransactionMonitor'
import { formatTokenAmount, getExplorerUrl, copyToClipboard, truncateAddress } from '@/utils'
import toast from 'react-hot-toast'

interface TransactionStatusProps {
  hash?: Hash
  type?: string
  description?: string
  isVisible?: boolean
  onClose?: () => void
}

interface TransactionDetails {
  hash: Hash
  status: 'pending' | 'confirmed' | 'failed'
  confirmations: number
  requiredConfirmations: number
  blockNumber?: bigint
  gasUsed?: bigint
  gasPrice?: bigint
  value?: bigint
  from?: string
  to?: string
  timestamp?: number
  error?: string
}

export function TransactionStatus({ 
  hash, 
  type = 'transaction', 
  description,
  isVisible = true,
  onClose 
}: TransactionStatusProps) {
  const { getTransactionDetails, pendingTransactions } = useTransactionMonitor()
  const [details, setDetails] = useState<TransactionDetails | null>(null)
  const [isExpanded, setIsExpanded] = useState(false)
  const [isLoading, setIsLoading] = useState(false)

  // Fetch transaction details
  useEffect(() => {
    const fetchDetails = async () => {
      if (!hash) return

      setIsLoading(true)
      try {
        const txDetails = await getTransactionDetails(hash)
        
        if (txDetails) {
          setDetails({
            hash,
            status: txDetails.isPending ? 'pending' : 
                   txDetails.receipt?.status === 'success' ? 'confirmed' : 'failed',
            confirmations: txDetails.confirmations,
            requiredConfirmations: 2, // Default requirement
            blockNumber: txDetails.receipt?.blockNumber,
            gasUsed: txDetails.receipt?.gasUsed,
            gasPrice: txDetails.transaction.gasPrice,
            value: txDetails.transaction.value,
            from: txDetails.transaction.from,
            to: txDetails.transaction.to,
            timestamp: Date.now(), // Would get from block in real implementation
          })
        }
      } catch (error) {
        console.error('Failed to fetch transaction details:', error)
        setDetails({
          hash,
          status: 'failed',
          confirmations: 0,
          requiredConfirmations: 2,
          error: error instanceof Error ? error.message : 'Failed to fetch details',
        })
      } finally {
        setIsLoading(false)
      }
    }

    fetchDetails()
    
    // Refresh details every 30 seconds for pending transactions
    const interval = setInterval(fetchDetails, 30000)
    return () => clearInterval(interval)
  }, [hash, getTransactionDetails])

  // Get status display info
  const getStatusInfo = () => {
    if (!details) return { icon: Clock, color: 'gray', label: 'Loading...' }

    switch (details.status) {
      case 'pending':
        return { 
          icon: Loader2, 
          color: 'blue', 
          label: `Pending (${details.confirmations}/${details.requiredConfirmations})`,
          animated: true
        }
      case 'confirmed':
        return { 
          icon: CheckCircle, 
          color: 'green', 
          label: 'Confirmed',
          animated: false
        }
      case 'failed':
        return { 
          icon: XCircle, 
          color: 'red', 
          label: 'Failed',
          animated: false
        }
      default:
        return { icon: Clock, color: 'gray', label: 'Unknown', animated: false }
    }
  }

  const statusInfo = getStatusInfo()
  const StatusIcon = statusInfo.icon

  if (!isVisible || !hash) return null

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        className="fixed bottom-4 right-4 z-50 max-w-sm"
      >
        <Card className="bg-white/95 dark:bg-gray-900/95 backdrop-blur-md border shadow-xl">
          <div className="p-4">
            {/* Header */}
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center space-x-2">
                <StatusIcon 
                  className={`w-5 h-5 text-${statusInfo.color}-500 ${
                    statusInfo.animated ? 'animate-spin' : ''
                  }`} 
                />
                <span className="font-medium text-sm">{type.toUpperCase()}</span>
                <Badge 
                  variant={statusInfo.color === 'green' ? 'success' : 
                          statusInfo.color === 'red' ? 'destructive' : 'info'}
                  className="text-xs"
                >
                  {statusInfo.label}
                </Badge>
              </div>
              {onClose && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={onClose}
                  className="p-1 h-auto"
                >
                  <X className="w-4 h-4" />
                </Button>
              )}
            </div>

            {/* Description */}
            {description && (
              <p className="text-sm text-gray-600 dark:text-gray-300 mb-3">
                {description}
              </p>
            )}

            {/* Progress Bar */}
            {details && details.status === 'pending' && (
              <div className="mb-3">
                <Progress 
                  value={(details.confirmations / details.requiredConfirmations) * 100}
                  className="h-2"
                />
                <div className="flex justify-between text-xs text-gray-500 mt-1">
                  <span>{details.confirmations} confirmations</span>
                  <span>{details.requiredConfirmations} required</span>
                </div>
              </div>
            )}

            {/* Transaction Hash */}
            <div className="flex items-center justify-between mb-3">
              <span className="text-xs text-gray-500">Transaction:</span>
              <div className="flex items-center space-x-2">
                <code className="text-xs font-mono bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
                  {truncateAddress(hash, 6)}
                </code>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => copyToClipboard(hash)}
                  className="p-1 h-auto"
                >
                  <Copy className="w-3 h-3" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => window.open(getExplorerUrl(hash, 'tx'), '_blank')}
                  className="p-1 h-auto"
                >
                  <ExternalLink className="w-3 h-3" />
                </Button>
              </div>
            </div>

            {/* Error Message */}
            {details?.error && (
              <div className="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded mb-3">
                <div className="flex items-center space-x-2">
                  <AlertTriangle className="w-4 h-4 text-red-500" />
                  <span className="text-sm text-red-700 dark:text-red-300">
                    {details.error}
                  </span>
                </div>
              </div>
            )}

            {/* Expandable Details */}
            {details && (
              <div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="w-full flex items-center justify-between p-2 text-xs"
                >
                  <span>Transaction Details</span>
                  {isExpanded ? (
                    <ChevronUp className="w-3 h-3" />
                  ) : (
                    <ChevronDown className="w-3 h-3" />
                  )}
                </Button>

                <AnimatePresence>
                  {isExpanded && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: 'auto', opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      className="overflow-hidden"
                    >
                      <div className="pt-2 space-y-2 text-xs">
                        {details.blockNumber && (
                          <div className="flex justify-between">
                            <span className="text-gray-500">Block:</span>
                            <span className="font-mono">{details.blockNumber.toString()}</span>
                          </div>
                        )}
                        
                        {details.gasUsed && details.gasPrice && (
                          <div className="flex justify-between">
                            <span className="text-gray-500">Gas Used:</span>
                            <span className="font-mono">
                              {details.gasUsed.toString()} @ {formatGwei(details.gasPrice)} gwei
                            </span>
                          </div>
                        )}
                        
                        {details.value && details.value > 0n && (
                          <div className="flex justify-between">
                            <span className="text-gray-500">Value:</span>
                            <span className="font-mono">{formatEther(details.value)} ETH</span>
                          </div>
                        )}
                        
                        {details.from && (
                          <div className="flex justify-between">
                            <span className="text-gray-500">From:</span>
                            <code className="font-mono">{truncateAddress(details.from)}</code>
                          </div>
                        )}
                        
                        {details.to && (
                          <div className="flex justify-between">
                            <span className="text-gray-500">To:</span>
                            <code className="font-mono">{truncateAddress(details.to)}</code>
                          </div>
                        )}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
            )}

            {/* Action Buttons */}
            {details?.status === 'confirmed' && (
              <div className="flex space-x-2 mt-3">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => window.open(getExplorerUrl(hash, 'tx'), '_blank')}
                  className="flex-1 text-xs"
                >
                  <ExternalLink className="w-3 h-3 mr-1" />
                  View on Explorer
                </Button>
              </div>
            )}
          </div>
        </Card>
      </motion.div>
    </AnimatePresence>
  )
}

export function TransactionStatusList() {
  const { pendingTransactions, recentTransactions } = useTransactionMonitor()
  const [showAll, setShowAll] = useState(false)

  const allTransactions = [
    ...pendingTransactions.map(tx => ({ ...tx, isPending: true })),
    ...recentTransactions.slice(0, showAll ? undefined : 5)
  ]

  if (allTransactions.length === 0) {
    return (
      <Card className="p-6 text-center">
        <Clock className="w-8 h-8 mx-auto text-gray-400 mb-2" />
        <p className="text-gray-500">No recent transactions</p>
      </Card>
    )
  }

  return (
    <Card className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold">Recent Transactions</h3>
        <Badge variant="info">{allTransactions.length}</Badge>
      </div>

      <div className="space-y-3">
        {allTransactions.map((tx, index) => (
          <div
            key={tx.hash || index}
            className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg"
          >
            <div className="flex items-center space-x-3">
              {'isPending' in tx && tx.isPending ? (
                <Loader2 className="w-4 h-4 text-blue-500 animate-spin" />
              ) : tx.status === 'confirmed' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : (
                <XCircle className="w-4 h-4 text-red-500" />
              )}
              
              <div>
                <p className="font-medium text-sm">
                  {'description' in tx ? tx.description : tx.type.replace('_', ' ').toUpperCase()}
                </p>
                <p className="text-xs text-gray-500">
                  {new Date(tx.timestamp).toLocaleTimeString()}
                </p>
              </div>
            </div>

            <div className="flex items-center space-x-2">
              {'confirmations' in tx && (
                <Badge variant="outline" className="text-xs">
                  {tx.confirmations}/{tx.requiredConfirmations}
                </Badge>
              )}
              
              <Button
                variant="ghost"
                size="sm"
                onClick={() => window.open(getExplorerUrl(tx.hash, 'tx'), '_blank')}
                className="p-1"
              >
                <ExternalLink className="w-3 h-3" />
              </Button>
            </div>
          </div>
        ))}
      </div>

      {recentTransactions.length > 5 && (
        <Button
          variant="ghost"
          onClick={() => setShowAll(!showAll)}
          className="w-full mt-3 text-sm"
        >
          {showAll ? 'Show Less' : `Show All (${recentTransactions.length})`}
        </Button>
      )}
    </Card>
  )
}

export default TransactionStatus