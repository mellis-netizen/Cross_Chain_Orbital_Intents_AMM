'use client'

import React, { useState } from 'react'
import { Shield, Lock, Clock, CheckCircle } from 'lucide-react'
import { Card } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'

export function MEVProtectionPanel() {
  const [commitRevealEnabled, setCommitRevealEnabled] = useState(true)
  const [batchExecution, setBatchExecution] = useState(true)

  return (
    <div className="space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">MEV Protection Center</h2>
        <p className="text-gray-400 max-w-2xl mx-auto">
          Advanced MEV protection mechanisms to secure your trades from front-running and sandwich attacks.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center">
            <Shield className="w-5 h-5 mr-2 text-blue-400" />
            Protection Settings
          </h3>
          
          <div className="space-y-4">
            <label className="flex items-center justify-between cursor-pointer">
              <div className="flex items-center space-x-3">
                <Lock className="w-4 h-4 text-green-400" />
                <span className="text-gray-300">Commit-Reveal Scheme</span>
              </div>
              <input
                type="checkbox"
                checked={commitRevealEnabled}
                onChange={(e) => setCommitRevealEnabled(e.target.checked)}
                className="w-4 h-4 text-blue-400 bg-white/5 border-white/20 rounded focus:ring-blue-400"
              />
            </label>

            <label className="flex items-center justify-between cursor-pointer">
              <div className="flex items-center space-x-3">
                <Clock className="w-4 h-4 text-yellow-400" />
                <span className="text-gray-300">Batch Execution</span>
              </div>
              <input
                type="checkbox"
                checked={batchExecution}
                onChange={(e) => setBatchExecution(e.target.checked)}
                className="w-4 h-4 text-blue-400 bg-white/5 border-white/20 rounded focus:ring-blue-400"
              />
            </label>
          </div>

          <div className="mt-6">
            <Badge variant={commitRevealEnabled && batchExecution ? 'success' : 'warning'}>
              {commitRevealEnabled && batchExecution ? 'Fully Protected' : 'Partially Protected'}
            </Badge>
          </div>
        </Card>

        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center">
            <CheckCircle className="w-5 h-5 mr-2 text-green-400" />
            Protection Status
          </h3>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Sandwich Attacks</span>
              <Badge variant="success">Blocked</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Front-running</span>
              <Badge variant="success">Protected</Badge>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-gray-400">Price Manipulation</span>
              <Badge variant="success">Detected</Badge>
            </div>
          </div>
        </Card>
      </div>
    </div>
  )
}