import { useState, useEffect, useCallback } from 'react'
import { useAccount, useConnect, useDisconnect, useBalance as useWagmiBalance, useNetwork, useSwitchNetwork } from 'wagmi'
import { formatEther } from 'viem'
import { HOLESKY_CHAIN_ID } from '@/constants'
import { storage, formatNumber } from '@/utils'

export function useWallet() {
  const { address, isConnected, isConnecting } = useAccount()
  const { connect, connectors, error: connectError, isLoading: isConnectLoading } = useConnect()
  const { disconnect } = useDisconnect()
  const { chain } = useNetwork()
  const { switchNetwork, isLoading: isSwitchLoading } = useSwitchNetwork()

  const isCorrectNetwork = chain?.id === HOLESKY_CHAIN_ID

  const connectWallet = useCallback((connectorId?: string) => {
    const connector = connectorId 
      ? connectors.find(c => c.id === connectorId)
      : connectors[0]
    
    if (connector) {
      connect({ connector })
    }
  }, [connect, connectors])

  const disconnectWallet = useCallback(() => {
    disconnect()
    // Clear any cached data
    storage.remove('wallet-cache')
  }, [disconnect])

  const switchToCorrectNetwork = useCallback(() => {
    if (switchNetwork && !isCorrectNetwork) {
      switchNetwork(HOLESKY_CHAIN_ID)
    }
  }, [switchNetwork, isCorrectNetwork])

  return {
    address,
    isConnected,
    isConnecting: isConnecting || isConnectLoading,
    isCorrectNetwork,
    chain,
    connectors,
    connectWallet,
    disconnectWallet,
    switchToCorrectNetwork,
    isSwitchLoading,
    error: connectError,
  }
}

export function useBalance(address?: string) {
  const { data, isError, isLoading, refetch } = useWagmiBalance({
    address: address as `0x${string}`,
    enabled: !!address,
    watch: true,
  })

  const formattedBalance = data ? formatEther(data.value) : '0'
  const displayBalance = formatNumber(parseFloat(formattedBalance), { decimals: 4 })

  return {
    balance: data?.value.toString() || '0',
    formattedBalance,
    displayBalance,
    symbol: data?.symbol || 'ETH',
    decimals: data?.decimals || 18,
    isError,
    isLoading,
    refetch,
  }
}

export function useTokenBalances(address?: string, tokenAddresses: string[] = []) {
  const [balances, setBalances] = useState<Record<string, string>>({})
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchBalances = useCallback(async () => {
    if (!address || tokenAddresses.length === 0) return

    setLoading(true)
    setError(null)
    
    try {
      const { createPublicClient, http } = await import('viem')
      const { HOLESKY_CHAIN_ID } = await import('@/constants')
      const { MOCK_USDC_ABI } = await import('@/lib/contracts')
      
      // Create public client for reading
      const client = createPublicClient({
        chain: {
          id: HOLESKY_CHAIN_ID,
          name: 'Holesky',
          network: 'holesky',
          nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
          rpcUrls: {
            default: { http: ['https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/'] },
            public: { http: ['https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/'] },
          },
          blockExplorers: {
            default: { name: 'Etherscan', url: 'https://holesky.etherscan.io' },
          },
        },
        transport: http(),
      })
      
      const balancePromises = tokenAddresses.map(async (tokenAddress) => {
        try {
          if (tokenAddress === '0x0000000000000000000000000000000000000000') {
            // ETH balance
            const balance = await client.getBalance({ address: address as `0x${string}` })
            return { tokenAddress, balance: balance.toString() }
          } else {
            // ERC20 token balance
            const balance = await client.readContract({
              address: tokenAddress as `0x${string}`,
              abi: MOCK_USDC_ABI,
              functionName: 'balanceOf',
              args: [address as `0x${string}`],
            })
            return { tokenAddress, balance: (balance as bigint).toString() }
          }
        } catch (err) {
          console.warn(`Failed to fetch balance for token ${tokenAddress}:`, err)
          return { tokenAddress, balance: '0' }
        }
      })
      
      const results = await Promise.all(balancePromises)
      const newBalances: Record<string, string> = {}
      
      results.forEach(({ tokenAddress, balance }) => {
        newBalances[tokenAddress] = balance
      })
      
      setBalances(newBalances)
    } catch (err) {
      console.error('Failed to fetch token balances:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch balances')
    } finally {
      setLoading(false)
    }
  }, [address, tokenAddresses])

  useEffect(() => {
    fetchBalances()
  }, [fetchBalances])

  return {
    balances,
    loading,
    error,
    refetch: fetchBalances,
  }
}

export function useWalletConnection() {
  const [isInitialized, setIsInitialized] = useState(false)
  const { isConnected, address } = useAccount()

  useEffect(() => {
    // Check if wallet was previously connected
    const wasConnected = storage.get('wallet-connected', false)
    if (wasConnected && !isConnected) {
      // Auto-connect if previously connected
      // This would be handled by wagmi's autoConnect
    }
    setIsInitialized(true)
  }, [isConnected])

  useEffect(() => {
    // Store connection state
    storage.set('wallet-connected', isConnected)
    if (isConnected && address) {
      storage.set('wallet-address', address)
    }
  }, [isConnected, address])

  return {
    isInitialized,
    isConnected,
    address,
  }
}

export function useTransactionStatus() {
  const [pendingTxs, setPendingTxs] = useState<string[]>([])
  const [confirmedTxs, setConfirmedTxs] = useState<string[]>([])
  const [failedTxs, setFailedTxs] = useState<string[]>([])

  const addPendingTx = useCallback((hash: string) => {
    setPendingTxs(prev => [...prev, hash])
  }, [])

  const confirmTx = useCallback((hash: string) => {
    setPendingTxs(prev => prev.filter(tx => tx !== hash))
    setConfirmedTxs(prev => [...prev, hash])
  }, [])

  const failTx = useCallback((hash: string) => {
    setPendingTxs(prev => prev.filter(tx => tx !== hash))
    setFailedTxs(prev => [...prev, hash])
  }, [])

  const clearTx = useCallback((hash: string) => {
    setPendingTxs(prev => prev.filter(tx => tx !== hash))
    setConfirmedTxs(prev => prev.filter(tx => tx !== hash))
    setFailedTxs(prev => prev.filter(tx => tx !== hash))
  }, [])

  const hasPendingTxs = pendingTxs.length > 0

  return {
    pendingTxs,
    confirmedTxs,
    failedTxs,
    hasPendingTxs,
    addPendingTx,
    confirmTx,
    failTx,
    clearTx,
  }
}

export function useGasPrice() {
  const [gasPrice, setGasPrice] = useState<string>('0')
  const [gasPriceGwei, setGasPriceGwei] = useState<string>('0')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchGasPrice = useCallback(async () => {
    setLoading(true)
    setError(null)
    
    try {
      const { createPublicClient, http, formatGwei } = await import('viem')
      const { HOLESKY_CHAIN_ID } = await import('@/constants')
      
      // Create public client for reading
      const client = createPublicClient({
        chain: {
          id: HOLESKY_CHAIN_ID,
          name: 'Holesky',
          network: 'holesky',
          nativeCurrency: { name: 'Ether', symbol: 'ETH', decimals: 18 },
          rpcUrls: {
            default: { http: ['https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/'] },
            public: { http: ['https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/'] },
          },
          blockExplorers: {
            default: { name: 'Etherscan', url: 'https://holesky.etherscan.io' },
          },
        },
        transport: http(),
      })
      
      // Get current gas price from the network
      const gasPriceResult = await client.getGasPrice()
      const gasPriceWei = gasPriceResult.toString()
      const gasPriceGweiValue = formatGwei(gasPriceResult)
      
      setGasPrice(gasPriceWei)
      setGasPriceGwei(gasPriceGweiValue)
    } catch (err) {
      console.error('Failed to fetch gas price:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch gas price')
      // Fallback to reasonable default for Holesky
      setGasPrice('20000000000') // 20 gwei
      setGasPriceGwei('20')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchGasPrice()
    const interval = setInterval(fetchGasPrice, 30000) // Update every 30 seconds
    return () => clearInterval(interval)
  }, [fetchGasPrice])

  return {
    gasPrice,
    gasPriceGwei,
    loading,
    error,
    refetch: fetchGasPrice,
  }
}

export function useEnsName(address?: string) {
  const [ensName, setEnsName] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!address) return

    setLoading(true)
    // In a real implementation, resolve ENS name
    // For now, return null as Holesky doesn't have ENS
    setEnsName(null)
    setLoading(false)
  }, [address])

  return {
    ensName,
    loading,
  }
}

export function useWalletModal() {
  const [isOpen, setIsOpen] = useState(false)

  const openModal = useCallback(() => setIsOpen(true), [])
  const closeModal = useCallback(() => setIsOpen(false), [])

  return {
    isOpen,
    openModal,
    closeModal,
  }
}

export default {
  useWallet,
  useBalance,
  useTokenBalances,
  useWalletConnection,
  useTransactionStatus,
  useGasPrice,
  useEnsName,
  useWalletModal,
}