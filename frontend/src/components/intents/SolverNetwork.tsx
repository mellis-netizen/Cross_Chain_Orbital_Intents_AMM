'use client'

import React, { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import { 
  Network,
  Award,
  Zap,
  Shield,
  TrendingUp,
  TrendingDown,
  Clock,
  DollarSign,
  Activity,
  Users,
  Star,
  CheckCircle,
  AlertTriangle,
  ExternalLink
} from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { Progress } from '@/components/ui/Progress'

interface Solver {
  address: string
  name: string
  reputation: number
  stake: string
  totalIntents: number
  successfulIntents: number
  failedIntents: number
  avgExecutionTime: number
  totalVolume: string
  lastActive: number
  status: 'active' | 'inactive' | 'slashed'
  specialties: string[]
  profitGenerated: string
  gasOptimization: number
}

// Mock solver data
const MOCK_SOLVERS: Solver[] = [
  {
    address: '0x1234567890abcdef1234567890abcdef12345678',
    name: 'Alpha Solver',
    reputation: 9850,
    stake: '50000000000000000000', // 50 ETH
    totalIntents: 1247,
    successfulIntents: 1230,
    failedIntents: 17,
    avgExecutionTime: 42,
    totalVolume: '2500000000000000000000', // 2500 ETH
    lastActive: Date.now() - 5 * 60 * 1000, // 5 minutes ago
    status: 'active',
    specialties: ['ETH-USDC', 'Cross-chain', 'MEV Protection'],
    profitGenerated: '125000000000000000000', // 125 ETH
    gasOptimization: 95
  },
  {
    address: '0xabcdef1234567890abcdef1234567890abcdef12',
    name: 'Beta Solver',
    reputation: 8920,
    stake: '35000000000000000000', // 35 ETH
    totalIntents: 987,
    successfulIntents: 965,
    failedIntents: 22,
    avgExecutionTime: 38,
    totalVolume: '1800000000000000000000', // 1800 ETH
    lastActive: Date.now() - 2 * 60 * 1000, // 2 minutes ago
    status: 'active',
    specialties: ['Arbitrage', 'Fast Execution', 'Low Slippage'],
    profitGenerated: '89000000000000000000', // 89 ETH
    gasOptimization: 92
  },
  {
    address: '0x9876543210fedcba9876543210fedcba98765432',
    name: 'Gamma Solver',
    reputation: 7650,
    stake: '25000000000000000000', // 25 ETH
    totalIntents: 654,
    successfulIntents: 628,
    failedIntents: 26,
    avgExecutionTime: 55,
    totalVolume: '980000000000000000000', // 980 ETH
    lastActive: Date.now() - 15 * 60 * 1000, // 15 minutes ago
    status: 'active',
    specialties: ['Stable Swaps', 'High Volume', 'Multi-chain'],
    profitGenerated: '45000000000000000000', // 45 ETH
    gasOptimization: 88
  },
  {
    address: '0xfedcba0987654321fedcba0987654321fedcba09',
    name: 'Delta Solver',
    reputation: 6420,
    stake: '15000000000000000000', // 15 ETH
    totalIntents: 432,
    successfulIntents: 398,
    failedIntents: 34,
    avgExecutionTime: 67,
    totalVolume: '650000000000000000000', // 650 ETH
    lastActive: Date.now() - 60 * 60 * 1000, // 1 hour ago
    status: 'inactive',
    specialties: ['L2 Bridges', 'DeFi Integration'],
    profitGenerated: '28000000000000000000', // 28 ETH
    gasOptimization: 85
  },
  {
    address: '0x5555666677778888999900001111222233334444',
    name: 'Epsilon Solver',
    reputation: 3200,
    stake: '5000000000000000000', // 5 ETH
    totalIntents: 156,
    successfulIntents: 134,
    failedIntents: 22,
    avgExecutionTime: 89,
    totalVolume: '180000000000000000000', // 180 ETH
    lastActive: Date.now() - 3 * 24 * 60 * 60 * 1000, // 3 days ago
    status: 'slashed',
    specialties: ['Experimental Routes'],
    profitGenerated: '8000000000000000000', // 8 ETH
    gasOptimization: 72
  }
]

interface NetworkStats {
  totalSolvers: number
  activeSolvers: number
  totalStaked: string
  totalVolume: string
  avgReputationScore: number
  networkUptime: number
}

export function SolverNetwork() {
  const [solvers, setSolvers] = useState<Solver[]>([])
  const [sortBy, setSortBy] = useState<'reputation' | 'volume' | 'performance'>('reputation')
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'inactive' | 'slashed'>('all')

  useEffect(() => {
    setSolvers(MOCK_SOLVERS)
  }, [])

  const calculateNetworkStats = (): NetworkStats => {
    const activeSolvers = solvers.filter(s => s.status === 'active')
    const totalStaked = solvers.reduce((sum, s) => sum + parseFloat(s.stake), 0)
    const totalVolume = solvers.reduce((sum, s) => sum + parseFloat(s.totalVolume), 0)
    const avgReputationScore = solvers.length > 0 ? 
      solvers.reduce((sum, s) => sum + s.reputation, 0) / solvers.length : 0

    return {
      totalSolvers: solvers.length,
      activeSolvers: activeSolvers.length,
      totalStaked: totalStaked.toString(),
      totalVolume: totalVolume.toString(),
      avgReputationScore,
      networkUptime: 99.87 // Mock uptime percentage
    }
  }

  const sortSolvers = (solvers: Solver[]) => {
    return [...solvers].sort((a, b) => {
      switch (sortBy) {
        case 'reputation':
          return b.reputation - a.reputation
        case 'volume':
          return parseFloat(b.totalVolume) - parseFloat(a.totalVolume)
        case 'performance':
          const aSuccessRate = a.successfulIntents / (a.successfulIntents + a.failedIntents)
          const bSuccessRate = b.successfulIntents / (b.successfulIntents + b.failedIntents)
          return bSuccessRate - aSuccessRate
        default:
          return 0
      }
    })
  }

  const filterSolvers = (solvers: Solver[]) => {
    if (filterStatus === 'all') return solvers
    return solvers.filter(s => s.status === filterStatus)
  }

  const getReputationColor = (reputation: number) => {
    if (reputation >= 9000) return 'text-green-400'
    if (reputation >= 7000) return 'text-blue-400'
    if (reputation >= 5000) return 'text-yellow-400'
    return 'text-red-400'
  }

  const getStatusBadge = (status: Solver['status']) => {
    switch (status) {
      case 'active':
        return <Badge variant=\"success\">Active</Badge>
      case 'inactive':
        return <Badge variant=\"warning\">Inactive</Badge>
      case 'slashed':
        return <Badge variant=\"destructive\">Slashed</Badge>
    }
  }

  const formatTime = (timestamp: number) => {
    const diff = Date.now() - timestamp
    const minutes = Math.floor(diff / (60 * 1000))
    const hours = Math.floor(minutes / 60)
    const days = Math.floor(hours / 24)

    if (days > 0) return `${days}d ago`
    if (hours > 0) return `${hours}h ago`
    return `${minutes}m ago`
  }

  const networkStats = calculateNetworkStats()
  const filteredAndSortedSolvers = sortSolvers(filterSolvers(solvers))

  const SolverCard = ({ solver }: { solver: Solver }) => {
    const successRate = (solver.successfulIntents / (solver.successfulIntents + solver.failedIntents)) * 100

    return (
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className=\"p-6 bg-white/5 border border-white/10 rounded-lg backdrop-blur-md\"
      >
        <div className=\"flex items-start justify-between mb-4\">
          <div className=\"flex items-center space-x-3\">
            <div className=\"p-3 bg-gradient-to-br from-purple-500/20 to-blue-500/20 rounded-lg\">
              <Network className=\"w-6 h-6 text-purple-400\" />
            </div>
            <div>
              <h3 className=\"font-semibold text-white\">{solver.name}</h3>
              <p className=\"text-sm text-gray-400\">{solver.address.slice(0, 10)}...</p>
            </div>
          </div>
          <div className=\"flex items-center space-x-2\">
            {getStatusBadge(solver.status)}
            <Button size=\"sm\" variant=\"ghost\" className=\"text-blue-400 hover:text-blue-300\">
              <ExternalLink className=\"w-4 h-4\" />
            </Button>
          </div>
        </div>

        <div className=\"grid grid-cols-2 gap-4 mb-4\">
          <div className=\"space-y-3\">
            <div>
              <div className=\"text-sm text-gray-400\">Reputation</div>
              <div className={`text-lg font-bold ${getReputationColor(solver.reputation)}`}>
                {solver.reputation.toLocaleString()}
              </div>
            </div>
            
            <div>
              <div className=\"text-sm text-gray-400\">Success Rate</div>
              <div className=\"flex items-center space-x-2\">
                <div className=\"text-lg font-bold text-white\">{successRate.toFixed(1)}%</div>
                <Progress value={successRate} className=\"w-16 h-2\" />
              </div>
            </div>
            
            <div>
              <div className=\"text-sm text-gray-400\">Avg Execution</div>
              <div className=\"text-lg font-bold text-white\">{solver.avgExecutionTime}s</div>
            </div>
          </div>

          <div className=\"space-y-3\">
            <div>
              <div className=\"text-sm text-gray-400\">Stake</div>
              <div className=\"text-lg font-bold text-white\">
                {(parseFloat(solver.stake) / 1e18).toFixed(0)} ETH
              </div>
            </div>
            
            <div>
              <div className=\"text-sm text-gray-400\">Total Volume</div>
              <div className=\"text-lg font-bold text-white\">
                {(parseFloat(solver.totalVolume) / 1e18).toFixed(0)} ETH
              </div>
            </div>
            
            <div>
              <div className=\"text-sm text-gray-400\">Gas Optimization</div>
              <div className=\"flex items-center space-x-2\">
                <div className=\"text-lg font-bold text-white\">{solver.gasOptimization}%</div>
                <Progress value={solver.gasOptimization} className=\"w-16 h-2\" />
              </div>
            </div>
          </div>
        </div>

        <div className=\"mb-4\">
          <div className=\"text-sm text-gray-400 mb-2\">Specialties</div>
          <div className=\"flex flex-wrap gap-2\">
            {solver.specialties.map((specialty) => (
              <Badge key={specialty} variant=\"outline\" className=\"text-xs\">
                {specialty}
              </Badge>
            ))}
          </div>
        </div>

        <div className=\"flex items-center justify-between text-sm\">
          <div className=\"text-gray-400\">
            {solver.totalIntents} intents • {solver.successfulIntents} successful
          </div>
          <div className=\"text-gray-400\">
            Last active: {formatTime(solver.lastActive)}
          </div>
        </div>
      </motion.div>
    )
  }

  return (
    <div className=\"space-y-6\">
      <div className=\"text-center\">
        <h2 className=\"text-3xl font-bold text-white mb-4\">Solver Network</h2>
        <p className=\"text-gray-400 max-w-2xl mx-auto\">
          Explore the decentralized network of solvers competing to execute your intents with optimal efficiency.
        </p>
      </div>

      {/* Network Stats */}
      <div className=\"grid grid-cols-1 md:grid-cols-3 lg:grid-cols-6 gap-4\">
        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <Users className=\"w-5 h-5 text-blue-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">{networkStats.totalSolvers}</div>
              <div className=\"text-xs text-gray-400\">Total Solvers</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <Activity className=\"w-5 h-5 text-green-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">{networkStats.activeSolvers}</div>
              <div className=\"text-xs text-gray-400\">Active Now</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <Shield className=\"w-5 h-5 text-purple-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">
                {(parseFloat(networkStats.totalStaked) / 1e18).toFixed(0)}
              </div>
              <div className=\"text-xs text-gray-400\">ETH Staked</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <DollarSign className=\"w-5 h-5 text-orange-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">
                {(parseFloat(networkStats.totalVolume) / 1e18).toFixed(0)}
              </div>
              <div className=\"text-xs text-gray-400\">ETH Volume</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <Star className=\"w-5 h-5 text-yellow-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">
                {networkStats.avgReputationScore.toFixed(0)}
              </div>
              <div className=\"text-xs text-gray-400\">Avg Reputation</div>
            </div>
          </div>
        </Card>

        <Card className=\"p-4 bg-white/5 border-white/10 backdrop-blur-md\">
          <div className=\"flex items-center space-x-2\">
            <CheckCircle className=\"w-5 h-5 text-green-400\" />
            <div>
              <div className=\"text-lg font-bold text-white\">{networkStats.networkUptime}%</div>
              <div className=\"text-xs text-gray-400\">Uptime</div>
            </div>
          </div>
        </Card>
      </div>

      {/* Controls */}
      <Card className=\"p-6 bg-white/5 border-white/10 backdrop-blur-md\">
        <div className=\"flex flex-wrap items-center gap-4\">
          <div className=\"flex items-center space-x-2\">
            <span className=\"text-sm text-gray-400\">Sort by:</span>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className=\"bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white\"
            >
              <option value=\"reputation\">Reputation</option>
              <option value=\"volume\">Volume</option>
              <option value=\"performance\">Performance</option>
            </select>
          </div>

          <div className=\"flex items-center space-x-2\">
            <span className=\"text-sm text-gray-400\">Filter:</span>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as any)}
              className=\"bg-white/10 border border-white/20 rounded-lg px-3 py-2 text-white\"
            >
              <option value=\"all\">All Solvers</option>
              <option value=\"active\">Active Only</option>
              <option value=\"inactive\">Inactive</option>
              <option value=\"slashed\">Slashed</option>
            </select>
          </div>
        </div>
      </Card>

      {/* Solver List */}
      <div className=\"grid gap-6\">
        {filteredAndSortedSolvers.map((solver) => (
          <SolverCard key={solver.address} solver={solver} />
        ))}
      </div>

      {filteredAndSortedSolvers.length === 0 && (
        <Card className=\"p-8 bg-white/5 border-white/10 backdrop-blur-md text-center\">
          <div className=\"text-gray-400\">
            <Network className=\"w-12 h-12 mx-auto mb-4 opacity-50\" />
            <h3 className=\"text-lg font-medium text-white mb-2\">No Solvers Found</h3>
            <p>No solvers match your current filters.</p>
          </div>
        </Card>
      )}
    </div>
  )
}