'use client'

import { useState } from 'react'
import { Card } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'
import { Button } from '@/components/ui/Button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs'
import { CrossChainBridge } from '@/components/bridge/CrossChainBridge'
import { IntentManager } from '@/components/intents/IntentManager'
import { IntentHistory } from '@/components/intents/IntentHistory'
import { SolverNetwork } from '@/components/intents/SolverNetwork'
import { 
  Zap, 
  History, 
  Network, 
  TrendingUp,
  Users,
  Clock,
  CheckCircle
} from 'lucide-react'

export default function IntentsPage() {
  const [activeTab, setActiveTab] = useState('create')

  return (
    <div className="max-w-7xl mx-auto space-y-8">
      <div className="text-center">
        <h1 className="text-4xl font-bold text-white mb-4">
          Cross-Chain <span className="bg-gradient-to-r from-purple-400 to-blue-400 bg-clip-text text-transparent">Intents</span>
        </h1>
        <p className="text-xl text-gray-400 max-w-3xl mx-auto">
          Express your trading intentions across multiple chains and let our solver network compete to execute them with optimal routing and MEV protection.
        </p>
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-blue-500/20 rounded-lg">
              <TrendingUp className="w-6 h-6 text-blue-400" />
            </div>
            <div>
              <div className="text-2xl font-bold text-white">1,247</div>
              <div className="text-sm text-gray-400">Total Intents</div>
            </div>
          </div>
          <div className="mt-2 text-xs text-green-400">+12% from last week</div>
        </Card>
        
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-green-500/20 rounded-lg">
              <CheckCircle className="w-6 h-6 text-green-400" />
            </div>
            <div>
              <div className="text-2xl font-bold text-white">98.5%</div>
              <div className="text-sm text-gray-400">Success Rate</div>
            </div>
          </div>
          <div className="mt-2 text-xs text-gray-400">Last 30 days</div>
        </Card>
        
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-purple-500/20 rounded-lg">
              <Clock className="w-6 h-6 text-purple-400" />
            </div>
            <div>
              <div className="text-2xl font-bold text-white">45s</div>
              <div className="text-sm text-gray-400">Avg Execution</div>
            </div>
          </div>
          <div className="mt-2 text-xs text-gray-400">Including MEV protection</div>
        </Card>
        
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center space-x-3">
            <div className="p-3 bg-orange-500/20 rounded-lg">
              <Users className="w-6 h-6 text-orange-400" />
            </div>
            <div>
              <div className="text-2xl font-bold text-white">23</div>
              <div className="text-sm text-gray-400">Active Solvers</div>
            </div>
          </div>
          <div className="mt-2 text-xs text-gray-400">Network participants</div>
        </Card>
      </div>

      {/* Main Interface */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
        <TabsList className="grid w-full grid-cols-4 bg-white/5 border border-white/10">
          <TabsTrigger 
            value="create" 
            className="flex items-center space-x-2 data-[state=active]:bg-white/10"
          >
            <Zap className="w-4 h-4" />
            <span>Create Intent</span>
          </TabsTrigger>
          <TabsTrigger 
            value="manage" 
            className="flex items-center space-x-2 data-[state=active]:bg-white/10"
          >
            <History className="w-4 h-4" />
            <span>Manage</span>
          </TabsTrigger>
          <TabsTrigger 
            value="history" 
            className="flex items-center space-x-2 data-[state=active]:bg-white/10"
          >
            <History className="w-4 h-4" />
            <span>History</span>
          </TabsTrigger>
          <TabsTrigger 
            value="solvers" 
            className="flex items-center space-x-2 data-[state=active]:bg-white/10"
          >
            <Network className="w-4 h-4" />
            <span>Solver Network</span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="create" className="mt-6">
          <CrossChainBridge />
        </TabsContent>

        <TabsContent value="manage" className="mt-6">
          <IntentManager />
        </TabsContent>

        <TabsContent value="history" className="mt-6">
          <IntentHistory />
        </TabsContent>

        <TabsContent value="solvers" className="mt-6">
          <SolverNetwork />
        </TabsContent>
      </Tabs>
    </div>
  )
}