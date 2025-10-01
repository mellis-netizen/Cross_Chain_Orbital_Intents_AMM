'use client'

import { AlertTriangle, CheckCircle, Loader } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { useWallet } from '@/hooks/useWeb3'
import { HOLESKY_CONFIG } from '@/constants'

export function NetworkStatus() {
  const { 
    chain, 
    isConnected, 
    isCorrectNetwork, 
    switchToCorrectNetwork, 
    isSwitchLoading 
  } = useWallet()

  if (!isConnected) {
    return null
  }

  if (isCorrectNetwork) {
    return (
      <div className="flex items-center space-x-2">
        <div className="h-2 w-2 bg-success-500 rounded-full animate-pulse" />
        <span className="text-sm text-muted-foreground hidden sm:inline">
          {HOLESKY_CONFIG.name}
        </span>
      </div>
    )
  }

  return (
    <div className="flex items-center space-x-2">
      <Badge variant="warning" className="flex items-center space-x-1">
        <AlertTriangle className="h-3 w-3" />
        <span>Wrong Network</span>
      </Badge>
      <Button
        size="sm"
        variant="outline"
        onClick={switchToCorrectNetwork}
        loading={isSwitchLoading}
        className="hidden sm:flex"
      >
        Switch to {HOLESKY_CONFIG.name}
      </Button>
    </div>
  )
}

// Network indicator for mobile
export function MobileNetworkStatus() {
  const { 
    chain, 
    isConnected, 
    isCorrectNetwork, 
    switchToCorrectNetwork, 
    isSwitchLoading 
  } = useWallet()

  if (!isConnected) {
    return (
      <div className="flex items-center justify-center py-2">
        <Badge variant="outline">Not Connected</Badge>
      </div>
    )
  }

  if (isCorrectNetwork) {
    return (
      <div className="flex items-center justify-center py-2">
        <Badge variant="success" className="flex items-center space-x-2">
          <CheckCircle className="h-3 w-3" />
          <span>{HOLESKY_CONFIG.name}</span>
        </Badge>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center space-y-2 py-2">
      <Badge variant="warning" className="flex items-center space-x-2">
        <AlertTriangle className="h-3 w-3" />
        <span>Wrong Network: {chain?.name || 'Unknown'}</span>
      </Badge>
      <Button
        size="sm"
        variant="outline"
        onClick={switchToCorrectNetwork}
        loading={isSwitchLoading}
        className="w-full"
      >
        {isSwitchLoading ? (
          <>
            <Loader className="h-4 w-4 mr-2 animate-spin" />
            Switching...
          </>
        ) : (
          `Switch to ${HOLESKY_CONFIG.name}`
        )}
      </Button>
    </div>
  )
}

export default NetworkStatus