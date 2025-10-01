import { SwapInterface } from '@/components/swap/SwapInterface'

export const metadata = {
  title: 'Swap - Orbital AMM',
  description: 'Swap tokens using the Cross-Chain Orbital AMM with MEV protection',
}

export default function SwapPage() {
  return (
    <div className="max-w-6xl mx-auto">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold mb-4">
          <span className="gradient-text">Orbital AMM</span>
        </h1>
        <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
          Experience next-generation trading with cross-chain intent execution, 
          virtual liquidity pools, and built-in MEV protection.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
        {/* Main Swap Interface */}
        <div className="lg:col-span-1 lg:col-start-2">
          <SwapInterface />
        </div>

        {/* Left Sidebar - Features */}
        <div className="lg:col-span-1 lg:col-start-1 lg:row-start-1 space-y-4">
          <div className="p-6 rounded-xl bg-gradient-to-br from-orbital-50 to-orbital-100 dark:from-orbital-900/20 dark:to-orbital-800/20 border border-orbital-200 dark:border-orbital-800">
            <h3 className="font-semibold text-orbital-800 dark:text-orbital-200 mb-3">
              🚀 MEV Protection
            </h3>
            <p className="text-sm text-orbital-700 dark:text-orbital-300">
              Built-in protection against front-running and sandwich attacks with commit-reveal schemes.
            </p>
          </div>

          <div className="p-6 rounded-xl bg-gradient-to-br from-success-50 to-success-100 dark:from-success-900/20 dark:to-success-800/20 border border-success-200 dark:border-success-800">
            <h3 className="font-semibold text-success-800 dark:text-success-200 mb-3">
              💎 Virtual Liquidity
            </h3>
            <p className="text-sm text-success-700 dark:text-success-300">
              Access deep liquidity from multiple chains through virtual pool aggregation.
            </p>
          </div>

          <div className="p-6 rounded-xl bg-gradient-to-br from-warning-50 to-warning-100 dark:from-warning-900/20 dark:to-warning-800/20 border border-warning-200 dark:border-warning-800">
            <h3 className="font-semibold text-warning-800 dark:text-warning-200 mb-3">
              ⚡ Dynamic Fees
            </h3>
            <p className="text-sm text-warning-700 dark:text-warning-300">
              Fees adjust automatically based on market volatility and trading volume.
            </p>
          </div>
        </div>

        {/* Right Sidebar - Stats */}
        <div className="lg:col-span-1 space-y-4">
          <div className="p-6 rounded-xl glass border">
            <h3 className="font-semibold mb-4">Pool Statistics</h3>
            <div className="space-y-3">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Value Locked</span>
                <span className="font-semibold">$2.4M</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">24h Volume</span>
                <span className="font-semibold text-success-600">$148K</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Intents</span>
                <span className="font-semibold">1,247</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Active Solvers</span>
                <span className="font-semibold">23</span>
              </div>
            </div>
          </div>

          <div className="p-6 rounded-xl glass border">
            <h3 className="font-semibold mb-4">Recent Activity</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">0.1 ETH → USDC</span>
                <span className="text-success-600">✓</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">0.5 ETH → USDC</span>
                <span className="text-success-600">✓</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">2.0 ETH → USDC</span>
                <span className="text-warning-600">⏳</span>
              </div>
            </div>
          </div>

          <div className="p-6 rounded-xl glass border">
            <h3 className="font-semibold mb-4">Network Status</h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Network</span>
                <div className="flex items-center space-x-2">
                  <div className="h-2 w-2 bg-success-500 rounded-full"></div>
                  <span className="text-sm">Holesky</span>
                </div>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Gas Price</span>
                <span className="font-semibold">20 gwei</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Block Number</span>
                <span className="font-semibold">1,234,567</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}