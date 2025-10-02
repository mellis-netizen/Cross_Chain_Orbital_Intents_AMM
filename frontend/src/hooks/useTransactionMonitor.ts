import { useState, useEffect, useCallback, useRef } from 'react'
import { usePublicClient, useAccount } from 'wagmi'
import { Hash, TransactionReceipt, Block, Log } from 'viem'
import { ORBITAL_AMM_ABI, INTENTS_ENGINE_ABI } from '@/lib/contracts'
import toast from 'react-hot-toast'

export interface TransactionEvent {
  type: 'swap' | 'intent_created' | 'intent_matched' | 'intent_executed' | 'approval' | 'transfer'
  hash: Hash
  blockNumber: bigint
  timestamp: number
  from: string
  to?: string
  value?: bigint
  data?: any
  status: 'pending' | 'confirmed' | 'failed'
}

export interface PendingTransaction {
  hash: Hash
  type: string
  description: string
  timestamp: number
  confirmations: number
  requiredConfirmations: number
}

export function useTransactionMonitor() {
  const { address } = useAccount()
  const publicClient = usePublicClient()
  
  const [recentTransactions, setRecentTransactions] = useState<TransactionEvent[]>([])
  const [pendingTransactions, setPendingTransactions] = useState<PendingTransaction[]>([])
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [latestBlock, setLatestBlock] = useState<bigint | null>(null)
  
  const intervalRef = useRef<NodeJS.Timeout | null>(null)
  const watchedHashes = useRef<Set<Hash>>(new Set())

  // Start monitoring
  const startMonitoring = useCallback(async () => {
    if (!publicClient || isMonitoring) return

    setIsMonitoring(true)
    
    try {
      // Get initial block number
      const currentBlock = await publicClient.getBlockNumber()
      setLatestBlock(currentBlock)

      // Set up polling for new blocks
      intervalRef.current = setInterval(async () => {
        try {
          const newBlock = await publicClient.getBlockNumber()
          
          if (newBlock > (latestBlock || 0n)) {
            setLatestBlock(newBlock)
            await checkForNewTransactions(newBlock)
          }
        } catch (error) {
          console.error('Block polling error:', error)
        }
      }, 12000) // Poll every 12 seconds (Ethereum block time)

    } catch (error) {
      console.error('Failed to start transaction monitoring:', error)
      setIsMonitoring(false)
    }
  }, [publicClient, isMonitoring, latestBlock])

  // Stop monitoring
  const stopMonitoring = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
    setIsMonitoring(false)
  }, [])

  // Check for new transactions in a block
  const checkForNewTransactions = useCallback(async (blockNumber: bigint) => {
    if (!publicClient || !address) return

    try {
      const block = await publicClient.getBlock({ 
        blockNumber,
        includeTransactions: true
      })

      // Filter transactions involving the current user
      const userTransactions = (block.transactions as any[]).filter(tx => 
        tx.from?.toLowerCase() === address.toLowerCase() || 
        tx.to?.toLowerCase() === address.toLowerCase()
      )

      for (const tx of userTransactions) {
        if (!watchedHashes.current.has(tx.hash)) {
          await processTransaction(tx, block)
          watchedHashes.current.add(tx.hash)
        }
      }
    } catch (error) {
      console.error('Error checking for new transactions:', error)
    }
  }, [publicClient, address])

  // Process individual transaction
  const processTransaction = useCallback(async (tx: any, block: Block) => {
    if (!publicClient) return

    try {
      // Get transaction receipt
      const receipt = await publicClient.getTransactionReceipt({ hash: tx.hash })
      
      // Determine transaction type and extract relevant data
      const event = await categorizeTransaction(tx, receipt, block)
      
      if (event) {
        setRecentTransactions(prev => [event, ...prev.slice(0, 49)]) // Keep last 50
        
        // Show notification
        if (event.status === 'confirmed') {
          toast.success(`${event.type.replace('_', ' ').toUpperCase()} confirmed!`)
        } else if (event.status === 'failed') {
          toast.error(`${event.type.replace('_', ' ').toUpperCase()} failed!`)
        }
      }
    } catch (error) {
      console.error('Error processing transaction:', error)
    }
  }, [publicClient])

  // Categorize transaction based on logs and data
  const categorizeTransaction = useCallback(async (
    tx: any, 
    receipt: TransactionReceipt, 
    block: Block
  ): Promise<TransactionEvent | null> => {
    const timestamp = Number(block.timestamp)
    
    // Check for contract interaction logs
    for (const log of receipt.logs) {
      // Orbital AMM Swap event
      if (log.topics[0] === '0x...') { // Swap event signature (to be replaced with actual)
        return {
          type: 'swap',
          hash: tx.hash,
          blockNumber: receipt.blockNumber,
          timestamp,
          from: tx.from,
          to: tx.to,
          value: tx.value,
          status: receipt.status === 'success' ? 'confirmed' : 'failed',
          data: await parseSwapLog(log),
        }
      }
      
      // Intent Created event
      if (log.topics[0] === '0x...') { // IntentCreated event signature
        return {
          type: 'intent_created',
          hash: tx.hash,
          blockNumber: receipt.blockNumber,
          timestamp,
          from: tx.from,
          to: tx.to,
          value: tx.value,
          status: receipt.status === 'success' ? 'confirmed' : 'failed',
          data: await parseIntentCreatedLog(log),
        }
      }
      
      // Intent Matched event
      if (log.topics[0] === '0x...') { // IntentMatched event signature
        return {
          type: 'intent_matched',
          hash: tx.hash,
          blockNumber: receipt.blockNumber,
          timestamp,
          from: tx.from,
          to: tx.to,
          value: tx.value,
          status: receipt.status === 'success' ? 'confirmed' : 'failed',
          data: await parseIntentMatchedLog(log),
        }
      }
      
      // ERC20 Transfer/Approval
      if (log.topics[0] === '0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef') {
        return {
          type: 'transfer',
          hash: tx.hash,
          blockNumber: receipt.blockNumber,
          timestamp,
          from: tx.from,
          to: tx.to,
          value: tx.value,
          status: receipt.status === 'success' ? 'confirmed' : 'failed',
          data: await parseTransferLog(log),
        }
      }
    }

    // Fallback for regular ETH transfers
    if (tx.value && BigInt(tx.value) > 0n) {
      return {
        type: 'transfer',
        hash: tx.hash,
        blockNumber: receipt.blockNumber,
        timestamp,
        from: tx.from,
        to: tx.to,
        value: tx.value,
        status: receipt.status === 'success' ? 'confirmed' : 'failed',
      }
    }

    return null
  }, [])

  // Parse log data (these would be implemented with actual ABI decoding)
  const parseSwapLog = useCallback(async (log: Log) => {
    // Decode swap event data
    return {
      poolId: log.topics[1],
      trader: log.topics[2],
      // ... other swap data
    }
  }, [])

  const parseIntentCreatedLog = useCallback(async (log: Log) => {
    // Decode intent created event data
    return {
      intentId: log.topics[1],
      user: log.topics[2],
      // ... other intent data
    }
  }, [])

  const parseIntentMatchedLog = useCallback(async (log: Log) => {
    // Decode intent matched event data
    return {
      intentId: log.topics[1],
      solver: log.topics[2],
      // ... other match data
    }
  }, [])

  const parseTransferLog = useCallback(async (log: Log) => {
    // Decode transfer event data
    return {
      from: log.topics[1],
      to: log.topics[2],
      amount: log.data,
    }
  }, [])

  // Add transaction to pending list
  const addPendingTransaction = useCallback((
    hash: Hash, 
    type: string, 
    description: string,
    requiredConfirmations: number = 1
  ) => {
    setPendingTransactions(prev => [
      ...prev,
      {
        hash,
        type,
        description,
        timestamp: Date.now(),
        confirmations: 0,
        requiredConfirmations,
      }
    ])
    
    // Watch this transaction
    watchedHashes.current.add(hash)
  }, [])

  // Update pending transaction confirmations
  const updateConfirmations = useCallback(async () => {
    if (!publicClient || pendingTransactions.length === 0) return

    const currentBlock = await publicClient.getBlockNumber()
    
    setPendingTransactions(prev => 
      prev.map(async tx => {
        try {
          const receipt = await publicClient.getTransactionReceipt({ hash: tx.hash })
          const confirmations = Number(currentBlock - receipt.blockNumber)
          
          return {
            ...tx,
            confirmations,
          }
        } catch {
          return tx // Transaction not yet mined
        }
      }).filter(tx => (tx as any).confirmations < (tx as any).requiredConfirmations)
    )
  }, [publicClient, pendingTransactions])

  // Get transaction history for current user
  const getTransactionHistory = useCallback(async (limit: number = 10) => {
    if (!publicClient || !address) return []

    try {
      // This would typically query an indexing service or local database
      // For now, return recent transactions from state
      return recentTransactions.slice(0, limit)
    } catch (error) {
      console.error('Error fetching transaction history:', error)
      return []
    }
  }, [publicClient, address, recentTransactions])

  // Get transaction details
  const getTransactionDetails = useCallback(async (hash: Hash) => {
    if (!publicClient) return null

    try {
      const [transaction, receipt] = await Promise.all([
        publicClient.getTransaction({ hash }),
        publicClient.getTransactionReceipt({ hash }).catch(() => null),
      ])

      return {
        transaction,
        receipt,
        isPending: !receipt,
        confirmations: receipt ? 
          Number((latestBlock || 0n) - receipt.blockNumber) : 0,
      }
    } catch (error) {
      console.error('Error fetching transaction details:', error)
      return null
    }
  }, [publicClient, latestBlock])

  // Start monitoring when wallet connects
  useEffect(() => {
    if (address && publicClient) {
      startMonitoring()
    } else {
      stopMonitoring()
    }

    return () => stopMonitoring()
  }, [address, publicClient, startMonitoring, stopMonitoring])

  // Update confirmations periodically
  useEffect(() => {
    if (pendingTransactions.length > 0) {
      const interval = setInterval(updateConfirmations, 30000) // Every 30 seconds
      return () => clearInterval(interval)
    }
  }, [pendingTransactions.length, updateConfirmations])

  return {
    // State
    recentTransactions,
    pendingTransactions,
    isMonitoring,
    latestBlock,
    
    // Actions
    startMonitoring,
    stopMonitoring,
    addPendingTransaction,
    getTransactionHistory,
    getTransactionDetails,
    
    // Utils
    isConnected: !!address && !!publicClient,
  }
}

export default useTransactionMonitor