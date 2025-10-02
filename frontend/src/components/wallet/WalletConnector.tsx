'use client'

import { useState } from 'react'
import { Wallet, LogOut, Copy, ExternalLink } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Modal } from '@/components/ui/Modal'
import { Badge } from '@/components/ui/Badge'
import { useWallet, useBalance } from '@/hooks/useWeb3'
import { truncateAddress, copyToClipboard, getExplorerUrl } from '@/utils'
import { toast } from 'react-hot-toast'

export function WalletConnector() {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const { 
    address, 
    isConnected, 
    isConnecting, 
    connectors, 
    connectWallet, 
    disconnectWallet,
    error 
  } = useWallet()
  
  const { displayBalance } = useBalance(address)

  const handleConnect = (connectorId: string) => {
    connectWallet(connectorId)
    setIsModalOpen(false)
  }

  const handleCopyAddress = async () => {
    if (address) {
      const success = await copyToClipboard(address)
      if (success) {
        toast.success('Address copied to clipboard')
      } else {
        toast.error('Failed to copy address')
      }
    }
  }

  const handleViewOnExplorer = () => {
    if (address) {
      window.open(getExplorerUrl(address, 'address'), '_blank')
    }
  }

  if (isConnected && address) {
    return (
      <div className="flex items-center space-x-2">
        <div className="hidden sm:flex items-center space-x-2 px-3 py-2 bg-muted rounded-lg">
          <div className="text-sm font-medium">
            {displayBalance} ETH
          </div>
        </div>
        <Button
          variant="outline"
          onClick={() => setIsModalOpen(true)}
          className="flex items-center space-x-2"
        >
          <Wallet className="h-4 w-4" />
          <span className="hidden sm:inline">
            {truncateAddress(address)}
          </span>
        </Button>

        {/* Account Modal */}
        <Modal
          isOpen={isModalOpen}
          onClose={() => setIsModalOpen(false)}
          title="Account"
          size="sm"
        >
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-muted rounded-lg">
              <div>
                <div className="text-sm text-muted-foreground">Connected with</div>
                <div className="flex items-center space-x-2">
                  <div className="font-medium">
                    {connectors.find(c => c.ready)?.name || 'Unknown Wallet'}
                  </div>
                  {isCorrectNetwork ? (
                    <CheckCircle className="h-4 w-4 text-success-600" />
                  ) : (
                    <AlertTriangle className="h-4 w-4 text-warning-600" />
                  )}
                </div>
              </div>
              <Button
                variant="ghost"
                size="sm"
                onClick={disconnectWallet}
                className="text-destructive hover:text-destructive/90"
              >
                <LogOut className="h-4 w-4 mr-2" />
                Disconnect
              </Button>
            </div>

            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Address</span>
                <div className="flex items-center space-x-2">
                  <code className="text-sm bg-muted px-2 py-1 rounded">
                    {truncateAddress(address, 8, 6)}
                  </code>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleCopyAddress}
                    className="h-8 w-8"
                  >
                    <Copy className="h-3 w-3" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={handleViewOnExplorer}
                    className="h-8 w-8"
                  >
                    <ExternalLink className="h-3 w-3" />
                  </Button>
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Balance</span>
                <span className="font-medium">{displayBalance} ETH</span>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-muted-foreground">Network</span>
                <Badge variant="success">Holesky</Badge>
              </div>
            </div>
          </div>
        </Modal>
      </div>
    )
  }

  return (
    <>
      <Button
        onClick={() => setIsModalOpen(true)}
        loading={isConnecting}
        variant="orbital"
        className="flex items-center space-x-2"
      >
        <Wallet className="h-4 w-4" />
        <span>Connect Wallet</span>
      </Button>

      {/* Connection Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="Connect Wallet"
        description="Choose how you want to connect your wallet"
        size="sm"
      >
        <div className="space-y-3">
          {connectors.map((connector) => (
            <Button
              key={connector.id}
              variant="outline"
              className="w-full justify-start p-4 h-auto"
              onClick={() => handleConnect(connector.id)}
              disabled={isConnecting}
            >
              <div className="flex items-center space-x-3">
                <div className="h-8 w-8 rounded-full bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                  <Wallet className="h-4 w-4 text-white" />
                </div>
                <div className="text-left">
                  <div className="font-medium">{connector.name}</div>
                  <div className="text-xs text-muted-foreground">
                    Connect using {connector.name}
                  </div>
                </div>
              </div>
            </Button>
          ))}
          
          {error && (
            <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg">
              <p className="text-sm text-destructive">
                {error.message || 'Failed to connect wallet'}
              </p>
            </div>
          )}
          
          <div className="text-xs text-muted-foreground text-center pt-2">
            By connecting, you agree to our Terms of Service and Privacy Policy
          </div>
        </div>
      </Modal>
    </>
  )
}

export default WalletConnector