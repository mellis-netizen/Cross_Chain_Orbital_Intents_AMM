'use client'

import { useState, useEffect, useCallback } from 'react'
import { Clock, CheckCircle, XCircle, ExternalLink, Copy, Filter, Search, ArrowUpDown, Loader2, RefreshCw } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from './Card'
import { Button } from './Button'
import { Input } from './Input'
import { Badge } from './Badge'
import { Modal } from './Modal'
import { useWallet, useTransactionStatus } from '@/hooks/useWeb3'
import { usePublicClient } from 'wagmi'
import { formatEther, formatUnits } from 'viem'
import { getExplorerUrl, copyToClipboard, truncateAddress, formatCurrency } from '@/utils'
import { toast } from 'react-hot-toast'

export interface Transaction {
  hash: string
  type: 'swap' | 'bridge' | 'intent' | 'approval' | 'liquidity'
  status: 'pending' | 'confirmed' | 'failed'
  timestamp: number
  blockNumber?: number
  from: string
  to?: string
  value: string
  gasUsed?: string
  gasPrice?: string
  gasCost?: string
  tokenIn?: {
    address: string
    symbol: string
    amount: string
    decimals: number
  }
  tokenOut?: {
    address: string
    symbol: string
    amount: string
    decimals: number
  }
  chainId: number
  confirmations?: number
  error?: string
}

interface TransactionHistoryProps {
  limit?: number
  showHeader?: boolean
  filterType?: 'all' | 'swap' | 'bridge' | 'intent' | 'approval' | 'liquidity'
}

export function TransactionHistory({ 
  limit = 10, 
  showHeader = true, 
  filterType = 'all' 
}: TransactionHistoryProps) {
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [filteredTransactions, setFilteredTransactions] = useState<Transaction[]>([])
  const [selectedTx, setSelectedTx] = useState<Transaction | null>(null)
  const [showDetailsModal, setShowDetailsModal] = useState(false)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'timestamp' | 'value' | 'gasUsed'>('timestamp')
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc')
  const [statusFilter, setStatusFilter] = useState<'all' | 'pending' | 'confirmed' | 'failed'>('all')

  const { address, isConnected } = useWallet()
  const { pendingTxs, confirmedTxs, failedTxs } = useTransactionStatus()
  const publicClient = usePublicClient()

  // Load transaction history
  const loadTransactions = useCallback(async () => {
    if (!isConnected || !address || !publicClient) return

    setLoading(true)
    try {
      // Get current block number
      const currentBlock = await publicClient.getBlockNumber()
      const fromBlock = currentBlock - BigInt(10000) // Last ~10000 blocks

      // Fetch transaction history from the blockchain
      // In a real implementation, you'd use an indexer service like The Graph or Alchemy
      const mockTransactions: Transaction[] = await fetchMockTransactions(address)
      
      // Merge with pending transactions
      const allTransactions = [
        ...mockTransactions,
        ...pendingTxs.map(hash => createPendingTransaction(hash)),
        ...confirmedTxs.map(hash => createConfirmedTransaction(hash)),
        ...failedTxs.map(hash => createFailedTransaction(hash))
      ]

      // Remove duplicates
      const uniqueTransactions = allTransactions.reduce((acc, tx) => {
        const existing = acc.find(t => t.hash === tx.hash)
        if (!existing) {
          acc.push(tx)
        } else if (tx.status !== 'pending' && existing.status === 'pending') {
          // Update pending transaction with confirmed/failed status
          Object.assign(existing, tx)
        }
        return acc
      }, [] as Transaction[])

      setTransactions(uniqueTransactions)
    } catch (error) {
      console.error('Failed to load transaction history:', error)
      toast.error('Failed to load transaction history')
    } finally {
      setLoading(false)
    }
  }, [isConnected, address, publicClient, pendingTxs, confirmedTxs, failedTxs])

  // Mock transaction data (replace with real blockchain data)
  const fetchMockTransactions = async (userAddress: string): Promise<Transaction[]> => {
    // Simulate API delay
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    return [
      {
        hash: '0xabc123def456789012345678901234567890abcdef123456789012345678901234',
        type: 'swap',
        status: 'confirmed',
        timestamp: Date.now() - 300000, // 5 minutes ago
        blockNumber: 12345678,
        from: userAddress,
        to: '0x1234567890123456789012345678901234567890',
        value: '0',
        gasUsed: '150000',
        gasPrice: '20000000000',
        gasCost: '0.003',
        tokenIn: {
          address: '0x0000000000000000000000000000000000000000',
          symbol: 'ETH',
          amount: '1.0',
          decimals: 18
        },
        tokenOut: {
          address: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7',
          symbol: 'USDC',
          amount: '3000.0',
          decimals: 6
        },
        chainId: 17000,
        confirmations: 12
      },
      {
        hash: '0xdef456abc789012345678901234567890123456def789012345678901234567',
        type: 'bridge',
        status: 'confirmed',
        timestamp: Date.now() - 600000, // 10 minutes ago
        blockNumber: 12345677,
        from: userAddress,
        to: '0x9876543210987654321098765432109876543210',
        value: '2000000000000000000', // 2 ETH
        gasUsed: '250000',
        gasPrice: '25000000000',
        gasCost: '0.00625',
        tokenIn: {
          address: '0x0000000000000000000000000000000000000000',
          symbol: 'ETH',
          amount: '2.0',
          decimals: 18
        },
        chainId: 17000,
        confirmations: 8
      },
      {
        hash: '0x789012def456abc123456789012345678901234567890123456789012345678',
        type: 'intent',
        status: 'pending',
        timestamp: Date.now() - 120000, // 2 minutes ago
        from: userAddress,
        to: '0x5555666677778888999900001111222233334444',
        value: '0',
        tokenIn: {
          address: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7',
          symbol: 'USDC',
          amount: '1000.0',
          decimals: 6
        },
        chainId: 17000
      }
    ]
  }

  const createPendingTransaction = (hash: string): Transaction => ({
    hash,
    type: 'swap', // Default type
    status: 'pending',
    timestamp: Date.now(),
    from: address || '',
    value: '0',
    chainId: 17000
  })

  const createConfirmedTransaction = (hash: string): Transaction => ({
    hash,
    type: 'swap',
    status: 'confirmed',
    timestamp: Date.now(),
    from: address || '',
    value: '0',
    chainId: 17000,
    confirmations: 1
  })

  const createFailedTransaction = (hash: string): Transaction => ({
    hash,
    type: 'swap',
    status: 'failed',
    timestamp: Date.now(),
    from: address || '',
    value: '0',
    chainId: 17000,
    error: 'Transaction failed'
  })

  // Filter and sort transactions
  useEffect(() => {
    let filtered = transactions

    // Apply type filter
    if (filterType !== 'all') {
      filtered = filtered.filter(tx => tx.type === filterType)
    }

    // Apply status filter
    if (statusFilter !== 'all') {
      filtered = filtered.filter(tx => tx.status === statusFilter)
    }

    // Apply search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(tx => 
        tx.hash.toLowerCase().includes(query) ||
        tx.type.toLowerCase().includes(query) ||
        tx.tokenIn?.symbol.toLowerCase().includes(query) ||
        tx.tokenOut?.symbol.toLowerCase().includes(query)
      )
    }

    // Sort transactions
    filtered.sort((a, b) => {
      let aValue: number, bValue: number
      
      switch (sortBy) {
        case 'timestamp':
          aValue = a.timestamp
          bValue = b.timestamp
          break
        case 'value':
          aValue = parseFloat(a.value || '0')
          bValue = parseFloat(b.value || '0')
          break
        case 'gasUsed':
          aValue = parseFloat(a.gasUsed || '0')
          bValue = parseFloat(b.gasUsed || '0')
          break
        default:
          aValue = a.timestamp
          bValue = b.timestamp
      }

      return sortOrder === 'desc' ? bValue - aValue : aValue - bValue
    })

    // Apply limit
    if (limit > 0) {
      filtered = filtered.slice(0, limit)
    }

    setFilteredTransactions(filtered)
  }, [transactions, filterType, statusFilter, searchQuery, sortBy, sortOrder, limit])

  // Load transactions on mount and when dependencies change
  useEffect(() => {
    loadTransactions()
  }, [loadTransactions])

  // Refresh transactions
  const handleRefresh = useCallback(async () => {
    setRefreshing(true)
    await loadTransactions()
    setRefreshing(false)
    toast.success('Transaction history refreshed')
  }, [loadTransactions])

  // Copy transaction hash
  const handleCopyHash = useCallback(async (hash: string) => {
    const success = await copyToClipboard(hash)
    if (success) {
      toast.success('Transaction hash copied')
    } else {
      toast.error('Failed to copy')
    }
  }, [])

  // View on block explorer
  const handleViewOnExplorer = useCallback((hash: string) => {
    const url = getExplorerUrl(hash, 'tx')
    window.open(url, '_blank')
  }, [])

  // Get transaction status icon
  const getStatusIcon = (status: Transaction['status']) => {
    switch (status) {
      case 'confirmed':
        return <CheckCircle className="h-4 w-4 text-success-600" />
      case 'failed':
        return <XCircle className="h-4 w-4 text-destructive" />
      case 'pending':
      default:
        return <Clock className="h-4 w-4 text-warning-600" />
    }
  }

  // Get transaction type badge variant
  const getTypeBadgeVariant = (type: Transaction['type']) => {
    switch (type) {
      case 'swap': return 'info'
      case 'bridge': return 'success'
      case 'intent': return 'warning'
      case 'approval': return 'secondary'
      case 'liquidity': return 'info'
      default: return 'secondary'
    }
  }

  // Format transaction value
  const formatTransactionValue = (tx: Transaction) => {
    if (tx.tokenIn && tx.tokenOut) {
      return `${tx.tokenIn.amount} ${tx.tokenIn.symbol} → ${tx.tokenOut.amount} ${tx.tokenOut.symbol}`
    }
    if (tx.tokenIn) {
      return `${tx.tokenIn.amount} ${tx.tokenIn.symbol}`
    }
    if (tx.value && tx.value !== '0') {
      return `${formatEther(BigInt(tx.value))} ETH`
    }
    return 'N/A'
  }

  if (!isConnected) {
    return (
      <Card className="p-8 text-center">
        <div className="text-muted-foreground">
          <Clock className="h-12 w-12 mx-auto mb-4 opacity-50" />
          <h3 className="text-lg font-medium mb-2">Connect Your Wallet</h3>
          <p>Connect your wallet to view transaction history.</p>
        </div>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {showHeader && (
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold">Transaction History</h2>
          <Button
            onClick={handleRefresh}
            disabled={refreshing}
            variant="outline"
            size="sm"
            className="flex items-center space-x-2"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
            <span>Refresh</span>
          </Button>
        </div>
      )}

      {/* Filters */}
      <Card className="p-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search transactions..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>

          {/* Status Filter */}
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as any)}
            className="px-3 py-2 border rounded-lg bg-background text-foreground"
          >
            <option value="all">All Status</option>
            <option value="pending">Pending</option>
            <option value="confirmed">Confirmed</option>
            <option value="failed">Failed</option>
          </select>

          {/* Sort By */}
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="px-3 py-2 border rounded-lg bg-background text-foreground"
          >
            <option value="timestamp">Sort by Time</option>
            <option value="value">Sort by Value</option>
            <option value="gasUsed">Sort by Gas</option>
          </select>

          {/* Sort Order */}
          <Button
            variant="outline"
            onClick={() => setSortOrder(prev => prev === 'desc' ? 'asc' : 'desc')}
            className="flex items-center space-x-2"
          >
            <ArrowUpDown className="h-4 w-4" />
            <span>{sortOrder === 'desc' ? 'Newest' : 'Oldest'}</span>
          </Button>
        </div>
      </Card>

      {/* Transaction List */}
      <Card>
        <CardContent className="p-0">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              <span className="ml-2 text-muted-foreground">Loading transactions...</span>
            </div>
          ) : filteredTransactions.length === 0 ? (
            <div className="text-center py-12">
              <Clock className="h-12 w-12 mx-auto mb-4 opacity-50 text-muted-foreground" />
              <h3 className="text-lg font-medium mb-2">No Transactions Found</h3>
              <p className="text-muted-foreground">
                {searchQuery || statusFilter !== 'all' ? 'No transactions match your filters.' : 'No transactions yet.'}
              </p>
            </div>
          ) : (
            <div className="divide-y divide-border">
              {filteredTransactions.map((tx) => (
                <div
                  key={tx.hash}
                  className="p-4 hover:bg-muted/50 cursor-pointer transition-colors"
                  onClick={() => {
                    setSelectedTx(tx)
                    setShowDetailsModal(true)
                  }}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      {getStatusIcon(tx.status)}
                      <div>
                        <div className="flex items-center space-x-2">
                          <Badge variant={getTypeBadgeVariant(tx.type)} className="text-xs">
                            {tx.type}
                          </Badge>
                          <code className="text-xs bg-muted px-1 py-0.5 rounded">
                            {truncateAddress(tx.hash, 8, 6)}
                          </code>
                        </div>
                        <div className="text-sm text-muted-foreground mt-1">
                          {formatTransactionValue(tx)}
                        </div>
                      </div>
                    </div>
                    
                    <div className="text-right">
                      <div className="text-sm font-medium">
                        {new Date(tx.timestamp).toLocaleTimeString()}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {tx.gasCost && `Gas: ${tx.gasCost} ETH`}
                        {tx.confirmations && ` • ${tx.confirmations} conf.`}
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Transaction Details Modal */}
      <Modal
        isOpen={showDetailsModal}
        onClose={() => setShowDetailsModal(false)}
        title="Transaction Details"
        size="lg"
      >
        {selectedTx && (
          <div className="space-y-6">
            {/* Status */}
            <div className="flex items-center space-x-3">
              {getStatusIcon(selectedTx.status)}
              <div>
                <div className="font-semibold capitalize">{selectedTx.status}</div>
                <div className="text-sm text-muted-foreground">
                  {selectedTx.confirmations ? `${selectedTx.confirmations} confirmations` : 'Waiting for confirmation'}
                </div>
              </div>
            </div>

            {/* Transaction Info */}
            <div className="grid grid-cols-2 gap-6">
              <div className="space-y-4">
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Transaction Hash</div>
                  <div className="flex items-center space-x-2">
                    <code className="text-sm bg-muted px-2 py-1 rounded flex-1">
                      {selectedTx.hash}
                    </code>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleCopyHash(selectedTx.hash)}
                      className="h-8 w-8"
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleViewOnExplorer(selectedTx.hash)}
                      className="h-8 w-8"
                    >
                      <ExternalLink className="h-3 w-3" />
                    </Button>
                  </div>
                </div>

                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Type</div>
                  <Badge variant={getTypeBadgeVariant(selectedTx.type)}>
                    {selectedTx.type}
                  </Badge>
                </div>

                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">Timestamp</div>
                  <div className="text-sm">{new Date(selectedTx.timestamp).toLocaleString()}</div>
                </div>

                {selectedTx.blockNumber && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">Block Number</div>
                    <div className="text-sm">{selectedTx.blockNumber.toLocaleString()}</div>
                  </div>
                )}
              </div>

              <div className="space-y-4">
                <div>
                  <div className="text-sm font-medium text-muted-foreground mb-1">From</div>
                  <code className="text-sm bg-muted px-2 py-1 rounded block">
                    {truncateAddress(selectedTx.from, 12, 8)}
                  </code>
                </div>

                {selectedTx.to && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">To</div>
                    <code className="text-sm bg-muted px-2 py-1 rounded block">
                      {truncateAddress(selectedTx.to, 12, 8)}
                    </code>
                  </div>
                )}

                {selectedTx.gasUsed && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">Gas Used</div>
                    <div className="text-sm">{parseInt(selectedTx.gasUsed).toLocaleString()}</div>
                  </div>
                )}

                {selectedTx.gasCost && (
                  <div>
                    <div className="text-sm font-medium text-muted-foreground mb-1">Gas Cost</div>
                    <div className="text-sm">{selectedTx.gasCost} ETH</div>
                  </div>
                )}
              </div>
            </div>

            {/* Token Information */}
            {(selectedTx.tokenIn || selectedTx.tokenOut) && (
              <div className="space-y-3">
                <div className="text-sm font-medium text-muted-foreground">Token Transfer</div>
                <div className="flex items-center space-x-4">
                  {selectedTx.tokenIn && (
                    <div className="flex-1 p-3 bg-muted/20 rounded-lg">
                      <div className="text-xs text-muted-foreground mb-1">From</div>
                      <div className="font-medium">
                        {selectedTx.tokenIn.amount} {selectedTx.tokenIn.symbol}
                      </div>
                    </div>
                  )}
                  
                  {selectedTx.tokenIn && selectedTx.tokenOut && (
                    <div className="text-muted-foreground">
                      →
                    </div>
                  )}
                  
                  {selectedTx.tokenOut && (
                    <div className="flex-1 p-3 bg-muted/20 rounded-lg">
                      <div className="text-xs text-muted-foreground mb-1">To</div>
                      <div className="font-medium">
                        {selectedTx.tokenOut.amount} {selectedTx.tokenOut.symbol}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* Error Message */}
            {selectedTx.error && (
              <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
                <div className="text-sm font-medium text-destructive mb-1">Error</div>
                <div className="text-sm text-destructive/80">{selectedTx.error}</div>
              </div>
            )}
          </div>
        )}
      </Modal>
    </div>
  )
}

export default TransactionHistory