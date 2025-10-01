'use client'

import React from 'react'
import { BarChart3, TrendingUp, DollarSign, Zap } from 'lucide-react'
import { Card } from '@/components/ui/Card'

export function OrbitalAnalytics() {
  return (
    <div className="space-y-8">
      <div className="text-center">
        <h2 className="text-3xl font-bold text-white mb-4">Orbital Analytics Dashboard</h2>
        <p className="text-gray-400 max-w-2xl mx-auto">
          Real-time analytics and performance metrics for N-dimensional orbital pools.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Sphere Integrity</p>
              <p className="text-2xl font-bold text-green-400">99.97%</p>
            </div>
            <BarChart3 className="w-8 h-8 text-green-400" />
          </div>
        </Card>

        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Capital Efficiency</p>
              <p className="text-2xl font-bold text-purple-400">127x</p>
            </div>
            <Zap className="w-8 h-8 text-purple-400" />
          </div>
        </Card>

        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">24h Volume</p>
              <p className="text-2xl font-bold text-blue-400">$5.2M</p>
            </div>
            <TrendingUp className="w-8 h-8 text-blue-400" />
          </div>
        </Card>

        <Card className="p-6 bg-white/5 border-white/10 backdrop-blur-md">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-gray-400">Fees Earned</p>
              <p className="text-2xl font-bold text-yellow-400">$15.6K</p>
            </div>
            <DollarSign className="w-8 h-8 text-yellow-400" />
          </div>
        </Card>
      </div>
    </div>
  )
}