'use client'

import React, { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { 
  Eye, 
  Clock, 
  CheckCircle, 
  XCircle, 
  AlertTriangle,
  ExternalLink,
  RefreshCw,
  X,
  Activity,
  User,
  DollarSign,
  Shield,
  Plus
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { Modal } from '@/components/ui/Modal'
import { IntentCreationModal } from './IntentCreationModal'
import { useWallet } from '@/hooks/useWeb3'
import { useIntent, useIntentExecution, useCancelIntent } from '@/hooks/useContracts'
import { formatUnits } from 'viem'
import { getExplorerUrl } from '@/utils'
import toast from 'react-hot-toast'

interface Intent {
  id: string
  user: string
  sourceChain: string
  destChain: string
  sourceToken: string
  destToken: string
  sourceAmount: string
  minDestAmount: string
  deadline: number
  status: 'pending' | 'matched' | 'executing' | 'completed' | 'failed' | 'cancelled'
  createdAt: number
  matchedAt?: number
  executedAt?: number
  solver?: string
  actualOutput?: string
  txHash?: string
}

interface Execution {
  solver: string
  matchedAt: number
  executedAt?: number
  destAmount?: string
  proofHash?: string
  verified: boolean
}

// Mock data for demonstration
const MOCK_INTENTS: Intent[] = [
  {
    id: '0x1234567890abcdef',
    user: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    sourceChain: 'Ethereum',
    destChain: 'Holesky',
    sourceToken: 'ETH',
    destToken: 'USDC',
    sourceAmount: '1000000000000000000', // 1 ETH
    minDestAmount: '1800000000', // 1800 USDC
    deadline: Date.now() + 30 * 60 * 1000, // 30 minutes from now
    status: 'executing',
    createdAt: Date.now() - 2 * 60 * 1000, // 2 minutes ago
    matchedAt: Date.now() - 1 * 60 * 1000, // 1 minute ago
    solver: '0xabcdef1234567890',
    txHash: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
  },
  {
    id: '0xabcdef1234567890',
    user: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    sourceChain: 'Holesky',
    destChain: 'Ethereum',
    sourceToken: 'USDC',
    destToken: 'ETH',
    sourceAmount: '2000000000', // 2000 USDC
    minDestAmount: '1100000000000000000', // 1.1 ETH
    deadline: Date.now() + 45 * 60 * 1000, // 45 minutes from now
    status: 'pending',
    createdAt: Date.now() - 5 * 60 * 1000, // 5 minutes ago
  },
  {
    id: '0x9876543210fedcba',
    user: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    sourceChain: 'Ethereum',
    destChain: 'Arbitrum',
    sourceToken: 'USDC',
    destToken: 'USDC',
    sourceAmount: '5000000000', // 5000 USDC
    minDestAmount: '4950000000', // 4950 USDC (accounting for fees)
    deadline: Date.now() - 10 * 60 * 1000, // Expired 10 minutes ago
    status: 'failed',
    createdAt: Date.now() - 70 * 60 * 1000, // 70 minutes ago
    matchedAt: Date.now() - 65 * 60 * 1000, // 65 minutes ago
    solver: '0x1234567890abcdef',
  }
]

export function IntentManager() {
  const { address, isConnected } = useWallet()
  const { cancelIntent, isLoading: isCancelling } = useCancelIntent()
  
  const [userIntents, setUserIntents] = useState<Intent[]>([])
  const [selectedIntent, setSelectedIntent] = useState<Intent | null>(null)
  const [showDetailsModal, setShowDetailsModal] = useState(false)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [refreshing, setRefreshing] = useState(false)

  // Load user intents (mock data for now)
  useEffect(() => {
    if (isConnected && address) {
      // Filter intents for current user
      const filteredIntents = MOCK_INTENTS.filter(
        intent => intent.user.toLowerCase() === address.toLowerCase()
      )
      setUserIntents(filteredIntents)
    } else {
      setUserIntents([])
    }
  }, [address, isConnected])

  const refreshIntents = async () => {
    setRefreshing(true)
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 1000))
    setRefreshing(false)
    toast.success('Intents refreshed')
  }

  const handleCancelIntent = async (intentId: string) => {
    try {
      await cancelIntent(intentId)
      
      // Update local state
      setUserIntents(prev => 
        prev.map(intent => 
          intent.id === intentId 
            ? { ...intent, status: 'cancelled' as const }
            : intent
        )
      )
      
      toast.success('Intent cancelled successfully')
    } catch (error) {
      console.error('Cancel intent error:', error)
      toast.error('Failed to cancel intent')
    }
  }

  const getStatusColor = (status: Intent['status']) => {
    switch (status) {
      case 'pending': return 'warning'
      case 'matched': return 'info'
      case 'executing': return 'info'
      case 'completed': return 'success'
      case 'failed': return 'destructive'
      case 'cancelled': return 'destructive'
      default: return 'warning'
    }
  }

  const getStatusIcon = (status: Intent['status']) => {
    switch (status) {
      case 'pending': return <Clock className=\"w-4 h-4\" />
      case 'matched': return <Eye className=\"w-4 h-4\" />
      case 'executing': return <RefreshCw className=\"w-4 h-4 animate-spin\" />
      case 'completed': return <CheckCircle className=\"w-4 h-4\" />
      case 'failed': return <XCircle className=\"w-4 h-4\" />
      case 'cancelled': return <X className=\"w-4 h-4\" />
      default: return <AlertTriangle className=\"w-4 h-4\" />
    }
  }

  const formatTimeRemaining = (deadline: number) => {
    const now = Date.now()
    const remaining = deadline - now
    
    if (remaining <= 0) return 'Expired'
    
    const minutes = Math.floor(remaining / (60 * 1000))
    const hours = Math.floor(minutes / 60)
    
    if (hours > 0) {
      return `${hours}h ${minutes % 60}m`
    }
    return `${minutes}m`
  }

  const IntentCard = ({ intent }: { intent: Intent }) => (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className=\"p-6 bg-white/5 border border-white/10 rounded-lg backdrop-blur-md hover:bg-white/10 transition-all cursor-pointer\"
      onClick={() => {
        setSelectedIntent(intent)
        setShowDetailsModal(true)
      }}
    >
      <div className=\"flex items-center justify-between mb-4\">
        <div className=\"flex items-center space-x-3\">
          <Badge variant={getStatusColor(intent.status)} className=\"flex items-center space-x-1\">
            {getStatusIcon(intent.status)}
            <span className=\"capitalize\">{intent.status}</span>
          </Badge>
          <span className=\"text-sm text-gray-400\">
            {intent.sourceChain} → {intent.destChain}
          </span>
        </div>
        <div className=\"text-sm text-gray-400\">
          {formatTimeRemaining(intent.deadline)}
        </div>
      </div>

      <div className=\"grid grid-cols-2 gap-4 mb-4\">
        <div>
          <div className=\"text-sm text-gray-400\">From</div>
          <div className=\"font-medium text-white\">
            {formatUnits(BigInt(intent.sourceAmount), 18)} {intent.sourceToken}
          </div>
        </div>
        <div>
          <div className=\"text-sm text-gray-400\">Min Receive</div>
          <div className=\"font-medium text-white\">
            {formatUnits(BigInt(intent.minDestAmount), intent.destToken === 'USDC' ? 6 : 18)} {intent.destToken}
          </div>
        </div>
      </div>

      <div className=\"flex items-center justify-between\">
        <div className=\"text-xs text-gray-500\">
          ID: {intent.id.slice(0, 10)}...
        </div>
        <div className=\"flex items-center space-x-2\">
          {intent.status === 'pending' && (
            <Button
              size=\"sm\"
              variant=\"destructive\"
              onClick={(e) => {
                e.stopPropagation()
                handleCancelIntent(intent.id)
              }}
              disabled={isCancelling}
            >
              Cancel
            </Button>
          )}
          <Button size=\"sm\" variant=\"ghost\">
            <Eye className=\"w-4 h-4\" />
          </Button>
        </div>
      </div>
    </motion.div>
  )

  if (!isConnected) {
    return (
      <Card className=\"p-8 bg-white/5 border-white/10 backdrop-blur-md text-center\">
        <div className=\"text-gray-400 mb-4\">
          <User className=\"w-12 h-12 mx-auto mb-4 opacity-50\" />
          <h3 className=\"text-lg font-medium text-white mb-2\">Connect Your Wallet</h3>
          <p>Connect your wallet to view and manage your intents.</p>
        </div>
      </Card>
    )
  }

  return (
    <div className=\"space-y-6\">
      <div className=\"flex items-center justify-between\">
        <h2 className=\"text-2xl font-bold text-white\">Your Intents</h2>
        <Button
          onClick={refreshIntents}
          disabled={refreshing}
          variant=\"outline\"
          className=\"flex items-center space-x-2\"
        >
          <RefreshCw className={`w-4 h-4 ${refreshing ? 'animate-spin' : ''}`} />
          <span>Refresh</span>
        </Button>
      </div>

      {userIntents.length === 0 ? (
        <Card className=\"p-8 bg-white/5 border-white/10 backdrop-blur-md text-center\">
          <div className=\"text-gray-400\">
            <Activity className=\"w-12 h-12 mx-auto mb-4 opacity-50\" />
            <h3 className=\"text-lg font-medium text-white mb-2\">No Intents Found</h3>
            <p>You haven't created any intents yet. Create your first intent to get started.</p>
          </div>
        </Card>
      ) : (
        <div className=\"space-y-4\">
          <AnimatePresence>
            {userIntents.map(intent => (
              <IntentCard key={intent.id} intent={intent} />
            ))}
          </AnimatePresence>
        </div>
      )}

      {/* Intent Details Modal */}
      <Modal
        isOpen={showDetailsModal}
        onClose={() => setShowDetailsModal(false)}
        title=\"Intent Details\"
      >
        {selectedIntent && (
          <div className=\"space-y-6\">
            <div className=\"flex items-center space-x-3\">
              <Badge variant={getStatusColor(selectedIntent.status)} className=\"flex items-center space-x-1\">
                {getStatusIcon(selectedIntent.status)}
                <span className=\"capitalize\">{selectedIntent.status}</span>
              </Badge>
              <span className=\"text-gray-400\">
                {selectedIntent.sourceChain} → {selectedIntent.destChain}
              </span>
            </div>

            <div className=\"grid grid-cols-2 gap-6\">
              <div className=\"space-y-4\">
                <div>
                  <div className=\"text-sm text-gray-400 mb-1\">Intent ID</div>
                  <div className=\"font-mono text-sm text-white\">{selectedIntent.id}</div>
                </div>
                
                <div>
                  <div className=\"text-sm text-gray-400 mb-1\">Source Amount</div>
                  <div className=\"font-medium text-white\">
                    {formatUnits(BigInt(selectedIntent.sourceAmount), 18)} {selectedIntent.sourceToken}
                  </div>
                </div>
                
                <div>
                  <div className=\"text-sm text-gray-400 mb-1\">Min Receive</div>
                  <div className=\"font-medium text-white\">
                    {formatUnits(BigInt(selectedIntent.minDestAmount), selectedIntent.destToken === 'USDC' ? 6 : 18)} {selectedIntent.destToken}
                  </div>
                </div>
              </div>

              <div className=\"space-y-4\">
                <div>
                  <div className=\"text-sm text-gray-400 mb-1\">Deadline</div>
                  <div className=\"font-medium text-white\">
                    {formatTimeRemaining(selectedIntent.deadline)}
                  </div>
                </div>
                
                <div>
                  <div className=\"text-sm text-gray-400 mb-1\">Created</div>
                  <div className=\"font-medium text-white\">
                    {new Date(selectedIntent.createdAt).toLocaleString()}
                  </div>
                </div>
                
                {selectedIntent.solver && (
                  <div>
                    <div className=\"text-sm text-gray-400 mb-1\">Solver</div>
                    <div className=\"font-mono text-sm text-white\">
                      {selectedIntent.solver.slice(0, 10)}...
                    </div>
                  </div>
                )}
              </div>
            </div>

            {selectedIntent.txHash && (
              <div className=\"p-4 bg-blue-500/10 border border-blue-500/20 rounded-lg\">
                <div className=\"flex items-center justify-between\">
                  <div>
                    <div className=\"text-sm text-blue-400 mb-1\">Transaction Hash</div>
                    <div className=\"font-mono text-sm text-white\">
                      {selectedIntent.txHash.slice(0, 20)}...
                    </div>
                  </div>
                  <Button
                    size=\"sm\"
                    variant=\"ghost\"
                    className=\"text-blue-400 hover:text-blue-300\"
                  >
                    <ExternalLink className=\"w-4 h-4\" />
                  </Button>
                </div>
              </div>
            )}

            <div className=\"flex justify-end space-x-3\">
              {selectedIntent.status === 'pending' && (
                <Button
                  variant=\"destructive\"
                  onClick={() => {
                    handleCancelIntent(selectedIntent.id)
                    setShowDetailsModal(false)
                  }}
                  disabled={isCancelling}
                >
                  Cancel Intent
                </Button>
              )}
              <Button variant=\"outline\" onClick={() => setShowDetailsModal(false)}>
                Close
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  )
}