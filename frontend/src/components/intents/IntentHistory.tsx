'use client'

import React, { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  History,
  Filter,
  Download,
  ExternalLink,
  TrendingUp,
  TrendingDown,
  Clock,
  CheckCircle,
  XCircle,
  DollarSign,
  BarChart3,
  Calendar
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { Input } from '@/components/ui/Input'
import { useWallet } from '@/hooks/useWeb3'
import { formatUnits } from 'viem'

interface HistoricalIntent {
  id: string
  sourceChain: string
  destChain: string
  sourceToken: string
  destToken: string
  sourceAmount: string
  actualOutput: string
  fee: string
  executionTime: number // in seconds
  solver: string
  timestamp: number
  txHash: string
  status: 'completed' | 'failed'
  profit?: string
}

// Mock historical data
const MOCK_HISTORY: HistoricalIntent[] = [
  {
    id: '0x1234567890abcdef',
    sourceChain: 'Ethereum',
    destChain: 'Holesky',
    sourceToken: 'ETH',
    destToken: 'USDC',
    sourceAmount: '1000000000000000000', // 1 ETH
    actualOutput: '1835000000', // 1835 USDC
    fee: '9175000', // 9.175 USDC
    executionTime: 42,
    solver: '0xabcdef1234567890',
    timestamp: Date.now() - 2 * 60 * 60 * 1000, // 2 hours ago
    txHash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
    status: 'completed',
    profit: '35000000' // 35 USDC profit
  },
  {
    id: '0x9876543210fedcba',
    sourceChain: 'Holesky',
    destChain: 'Ethereum',
    sourceToken: 'USDC',
    destToken: 'ETH',
    sourceAmount: '2000000000', // 2000 USDC
    actualOutput: '1089000000000000000', // 1.089 ETH
    fee: '5450000000000000', // 0.00545 ETH
    executionTime: 67,
    solver: '0x1234567890abcdef',
    timestamp: Date.now() - 6 * 60 * 60 * 1000, // 6 hours ago
    txHash: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    status: 'completed',
  },
  {
    id: '0xfedcba0987654321',
    sourceChain: 'Ethereum',
    destChain: 'Arbitrum',
    sourceToken: 'USDC',
    destToken: 'USDC',
    sourceAmount: '5000000000', // 5000 USDC
    actualOutput: '0', // Failed
    fee: '25000000', // 25 USDC (gas fee)
    executionTime: 0,
    solver: '0xfedcba0987654321',
    timestamp: Date.now() - 24 * 60 * 60 * 1000, // 24 hours ago
    txHash: '0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321',
    status: 'failed'
  },
  {
    id: '0xabcd1234efgh5678',
    sourceChain: 'Ethereum',
    destChain: 'Polygon',
    sourceToken: 'ETH',
    destToken: 'MATIC',
    sourceAmount: '500000000000000000', // 0.5 ETH
    actualOutput: '1250000000000000000000', // 1250 MATIC
    fee: '6250000000000000000', // 6.25 MATIC
    executionTime: 38,
    solver: '0xabcd1234efgh5678',
    timestamp: Date.now() - 48 * 60 * 60 * 1000, // 48 hours ago
    txHash: '0xabcd1234efgh5678abcd1234efgh5678abcd1234efgh5678abcd1234efgh5678',
    status: 'completed'
  }
]

export function IntentHistory() {
  const { address, isConnected } = useWallet()
  const [history, setHistory] = useState<HistoricalIntent[]>([])
  const [filteredHistory, setFilteredHistory] = useState<HistoricalIntent[]>([])
  const [filter, setFilter] = useState('')
  const [statusFilter, setStatusFilter] = useState<'all' | 'completed' | 'failed'>('all')
  const [timeFilter, setTimeFilter] = useState<'all' | '24h' | '7d' | '30d'>('all')

  // Load history
  useEffect(() => {
    if (isConnected && address) {
      setHistory(MOCK_HISTORY)
    } else {
      setHistory([])
    }
  }, [address, isConnected])

  // Apply filters
  useEffect(() => {
    let filtered = history

    // Text filter
    if (filter) {
      filtered = filtered.filter(intent => 
        intent.id.toLowerCase().includes(filter.toLowerCase()) ||
        intent.sourceChain.toLowerCase().includes(filter.toLowerCase()) ||
        intent.destChain.toLowerCase().includes(filter.toLowerCase()) ||
        intent.sourceToken.toLowerCase().includes(filter.toLowerCase()) ||
        intent.destToken.toLowerCase().includes(filter.toLowerCase())
      )
    }

    // Status filter
    if (statusFilter !== 'all') {
      filtered = filtered.filter(intent => intent.status === statusFilter)
    }

    // Time filter
    if (timeFilter !== 'all') {
      const now = Date.now()
      const timeRange = {
        '24h': 24 * 60 * 60 * 1000,
        '7d': 7 * 24 * 60 * 60 * 1000,
        '30d': 30 * 24 * 60 * 60 * 1000
      }[timeFilter]
      
      filtered = filtered.filter(intent => 
        now - intent.timestamp <= timeRange
      )
    }

    setFilteredHistory(filtered)
  }, [history, filter, statusFilter, timeFilter])

  const calculateStats = () => {
    const completed = history.filter(h => h.status === 'completed')
    const failed = history.filter(h => h.status === 'failed')
    
    const successRate = history.length > 0 ? (completed.length / history.length) * 100 : 0
    const avgExecutionTime = completed.length > 0 ? 
      completed.reduce((sum, h) => sum + h.executionTime, 0) / completed.length : 0
    
    const totalVolume = completed.reduce((sum, h) => {
      // Convert everything to ETH equivalent for simplicity
      const amount = parseFloat(formatUnits(BigInt(h.sourceAmount), h.sourceToken === 'USDC' ? 6 : 18))
      return sum + (h.sourceToken === 'ETH' ? amount : amount / 1800) // Assuming 1 ETH = 1800 USDC
    }, 0)

    return {
      totalIntents: history.length,
      successRate,
      avgExecutionTime,
      totalVolume,
      completedCount: completed.length,
      failedCount: failed.length
    }
  }

  const stats = calculateStats()

  const exportHistory = () => {
    const csvContent = [
      ['Intent ID', 'Source Chain', 'Dest Chain', 'Source Token', 'Dest Token', 'Source Amount', 'Output', 'Status', 'Timestamp'].join(','),
      ...filteredHistory.map(h => [
        h.id,
        h.sourceChain,
        h.destChain,
        h.sourceToken,
        h.destToken,
        formatUnits(BigInt(h.sourceAmount), h.sourceToken === 'USDC' ? 6 : 18),
        h.status === 'completed' ? formatUnits(BigInt(h.actualOutput), h.destToken === 'USDC' ? 6 : 18) : '0',
        h.status,
        new Date(h.timestamp).toISOString()
      ].join(','))
    ].join('\\n')

    const blob = new Blob([csvContent], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'intent-history.csv'
    a.click()
    URL.revokeObjectURL(url)
  }

  const HistoryCard = ({ intent }: { intent: HistoricalIntent }) => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className=\"p-6 bg-white/5 border border-white/10 rounded-lg backdrop-blur-md\"
    >
      <div className=\"flex items-center justify-between mb-4\">
        <div className=\"flex items-center space-x-3\">
          <Badge variant={intent.status === 'completed' ? 'success' : 'destructive'}>
            {intent.status === 'completed' ? (
              <CheckCircle className=\"w-4 h-4 mr-1\" />
            ) : (
              <XCircle className=\"w-4 h-4 mr-1\" />
            )}
            {intent.status}
          </Badge>
          <span className=\"text-sm text-gray-400\">
            {intent.sourceChain} → {intent.destChain}
          </span>
        </div>
        <div className=\"text-sm text-gray-400\">
          {new Date(intent.timestamp).toLocaleDateString()}
        </div>
      </div>

      <div className=\"grid grid-cols-3 gap-4 mb-4\">
        <div>
          <div className=\"text-sm text-gray-400\">Input</div>
          <div className=\"font-medium text-white\">
            {formatUnits(BigInt(intent.sourceAmount), intent.sourceToken === 'USDC' ? 6 : 18)} {intent.sourceToken}
          </div>
        </div>
        <div>
          <div className=\"text-sm text-gray-400\">Output</div>
          <div className=\"font-medium text-white\">
            {intent.status === 'completed' ? (
              `${formatUnits(BigInt(intent.actualOutput), intent.destToken === 'USDC' ? 6 : 18)} ${intent.destToken}`
            ) : (
              'Failed'
            )}
          </div>
        </div>
        <div>
          <div className=\"text-sm text-gray-400\">Execution Time</div>
          <div className=\"font-medium text-white\">
            {intent.status === 'completed' ? `${intent.executionTime}s` : 'N/A'}
          </div>
        </div>
      </div>

      {intent.profit && (
        <div className=\"mb-4 p-3 bg-green-500/10 border border-green-500/20 rounded-lg\">
          <div className=\"flex items-center space-x-2\">
            <TrendingUp className=\"w-4 h-4 text-green-400\" />
            <span className=\"text-sm text-green-400\">
              Profit: {formatUnits(BigInt(intent.profit), intent.destToken === 'USDC' ? 6 : 18)} {intent.destToken}
            </span>
          </div>
        </div>
      )}

      <div className=\"flex items-center justify-between\">
        <div className=\"text-xs text-gray-500\">
          ID: {intent.id.slice(0, 10)}...
        </div>
        <div className=\"flex items-center space-x-2\">
          <span className=\"text-xs text-gray-500\">
            Fee: {formatUnits(BigInt(intent.fee), intent.destToken === 'USDC' ? 6 : 18)} {intent.destToken}
          </span>
          <Button size=\"sm\" variant=\"ghost\" className=\"text-blue-400 hover:text-blue-300\">
            <ExternalLink className=\"w-4 h-4\" />
          </Button>
        </div>
      </div>
    </motion.div>
  )

  if (!isConnected) {
    return (
      <Card className=\"p-8 bg-white/5 border-white/10 backdrop-blur-md text-center\">
        <div className=\"text-gray-400 mb-4\">
          <History className=\"w-12 h-12 mx-auto mb-4 opacity-50\" />
          <h3 className=\"text-lg font-medium text-white mb-2\">Connect Your Wallet</h3>
          <p>Connect your wallet to view your intent history and analytics.</p>
        </div>
      </Card>
    )
  }

  return (
    <div className=\"space-y-6\">
      {/* Stats Overview */}
      <div className=\"grid grid-cols-1 md:grid-cols-4 gap-4\">
        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-3\">
            <div className=\"p-2 bg-blue-500/20 rounded-lg\">
              <BarChart3 className=\"w-5 h-5 text-blue-400\" />
            </div>
            <div>
              <div className=\"text-lg font-bold text-white\">{stats.totalIntents}</div>
              <div className=\"text-sm text-gray-400\">Total Intents</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-3\">
            <div className=\"p-2 bg-green-500/20 rounded-lg\">
              <CheckCircle className=\"w-5 h-5 text-green-400\" />
            </div>
            <div>
              <div className=\"text-lg font-bold text-white\">{stats.successRate.toFixed(1)}%</div>
              <div className=\"text-sm text-gray-400\">Success Rate</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-3\">
            <div className=\"p-2 bg-purple-500/20 rounded-lg\">
              <Clock className=\"w-5 h-5 text-purple-400\" />
            </div>
            <div>
              <div className=\"text-lg font-bold text-white\">{stats.avgExecutionTime.toFixed(0)}s</div>
              <div className=\"text-sm text-gray-400\">Avg Execution</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-3\">
            <div className=\"p-2 bg-orange-500/20 rounded-lg\">
              <DollarSign className=\"w-5 h-5 text-orange-400\" />
            </div>
            <div>
              <div className=\"text-lg font-bold text-white\">{stats.totalVolume.toFixed(2)} ETH</div>
              <div className=\"text-sm text-gray-400\">Total Volume</div>
            </div>
          </div>
        </Card>
      </div>

      {/* Filters */}
      <Card className=\"p-6 bg-white/5 border-white/10 backdrop-blur-md\">
        <div className=\"flex flex-wrap items-center gap-4\">
          <div className=\"flex-1 min-w-64\">
            <Input
              placeholder=\"Search by ID, chain, or token...\"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              className=\"bg-white/10 border-white/20\"
            />
          </div>
          
          <select
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as any)}
            className=\"bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white\"
          >
            <option value=\"all\">All Status</option>
            <option value=\"completed\">Completed</option>
            <option value=\"failed\">Failed</option>
          </select>

          <select
            value={timeFilter}
            onChange={(e) => setTimeFilter(e.target.value as any)}
            className=\"bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white\"
          >
            <option value=\"all\">All Time</option>
            <option value=\"24h\">Last 24h</option>
            <option value=\"7d\">Last 7 days</option>
            <option value=\"30d\">Last 30 days</option>
          </select>

          <Button
            onClick={exportHistory}
            variant=\"outline\"
            className=\"flex items-center space-x-2\"
          >
            <Download className=\"w-4 h-4\" />
            <span>Export</span>
          </Button>
        </div>
      </Card>

      {/* History List */}
      {filteredHistory.length === 0 ? (
        <Card className=\"p-8 bg-white/5 border-white/10 backdrop-blur-md text-center\">
          <div className=\"text-gray-400\">
            <History className=\"w-12 h-12 mx-auto mb-4 opacity-50\" />
            <h3 className=\"text-lg font-medium text-white mb-2\">No History Found</h3>
            <p>No intents match your current filters.</p>
          </div>
        </Card>
      ) : (
        <div className=\"space-y-4\">
          {filteredHistory.map(intent => (
            <HistoryCard key={intent.id} intent={intent} />
          ))}
        </div>
      )}
    </div>
  )
}