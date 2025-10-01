import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'

export const metadata = {
  title: 'Intents - Orbital AMM',
  description: 'Create and manage cross-chain intents',
}

export default function IntentsPage() {
  return (
    <div className="max-w-6xl mx-auto">
      <div className="text-center mb-8">
        <h1 className="text-4xl font-bold mb-4">
          Cross-Chain <span className="gradient-text">Intents</span>
        </h1>
        <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
          Express your trading intentions across multiple chains and let solvers compete to execute them.
        </p>
      </div>

      <div className="grid gap-6">
        {/* Create Intent Card */}
        <Card className="border-orbital-200 dark:border-orbital-800">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2">
              <span>Create New Intent</span>
              <Badge variant="info">Coming Soon</Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-muted-foreground mb-4">
              Intent creation interface is currently under development. This will allow you to:
            </p>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li>• Specify source and destination chains</li>
              <li>• Set token amounts and minimum output requirements</li>
              <li>• Define execution deadlines and conditions</li>
              <li>• Monitor solver competition and execution</li>
            </ul>
          </CardContent>
        </Card>

        {/* Intent History */}
        <Card>
          <CardHeader>
            <CardTitle>Intent History</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-center py-8 text-muted-foreground">
              <p>No intents found. Create your first intent to get started.</p>
            </div>
          </CardContent>
        </Card>

        {/* System Overview */}
        <div className="grid md:grid-cols-3 gap-4">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Total Intents</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">1,247</div>
              <p className="text-xs text-muted-foreground">+12% from last week</p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Success Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-success-600">98.5%</div>
              <p className="text-xs text-muted-foreground">Last 30 days</p>
            </CardContent>
          </Card>
          
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-base">Avg Execution Time</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">45s</div>
              <p className="text-xs text-muted-foreground">Including MEV protection</p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}