import { useState, useCallback, useEffect } from 'react'
import { useAccount, usePublicClient, useWalletClient } from 'wagmi'
import { 
  Hash, 
  TransactionReceipt, 
  parseGwei, 
  formatEther,
  EstimateGasParameters,
  SendTransactionParameters,
  WriteContractParameters
} from 'viem'
import toast from 'react-hot-toast'

export interface TransactionOptions {
  gasLimit?: bigint
  gasPrice?: bigint
  maxFeePerGas?: bigint
  maxPriorityFeePerGas?: bigint
  value?: bigint
  onHash?: (hash: Hash) => void
  onConfirmation?: (receipt: TransactionReceipt) => void
  onError?: (error: Error) => void
}

export interface TransactionStatus {
  hash?: Hash
  status: 'idle' | 'preparing' | 'pending' | 'confirming' | 'confirmed' | 'failed'
  receipt?: TransactionReceipt
  error?: Error
  gasUsed?: bigint
  effectiveGasPrice?: bigint
}

export interface BatchTransactionRequest {
  id: string
  type: 'contract' | 'transfer' | 'approval'
  description: string
  params: any
  gasEstimate?: bigint
}

export function useTransactions() {
  const { address } = useAccount()
  const publicClient = usePublicClient()
  const { data: walletClient } = useWalletClient()
  
  const [transactions, setTransactions] = useState<Record<string, TransactionStatus>>({})
  const [batchQueue, setBatchQueue] = useState<BatchTransactionRequest[]>([])
  const [isProcessingBatch, setIsProcessingBatch] = useState(false)

  // Gas estimation with optimization
  const estimateGas = useCallback(async (
    params: EstimateGasParameters,
    optimization: 'speed' | 'standard' | 'economy' = 'standard'
  ): Promise<{ gasLimit: bigint; gasPrice: bigint; maxFeePerGas?: bigint; maxPriorityFeePerGas?: bigint }> => {
    if (!publicClient) {
      throw new Error('Public client not available')
    }

    try {
      // Estimate gas limit with 20% buffer
      const gasLimit = await publicClient.estimateGas(params)
      const bufferedGasLimit = (gasLimit * 120n) / 100n

      // Get current gas price
      const gasPrice = await publicClient.getGasPrice()
      
      // For EIP-1559 networks, also get fee data
      let maxFeePerGas: bigint | undefined
      let maxPriorityFeePerGas: bigint | undefined
      
      try {
        const feeData = await publicClient.estimateFeesPerGas()
        
        // Adjust fees based on optimization preference
        const multiplier = optimization === 'speed' ? 150n : optimization === 'economy' ? 80n : 100n
        
        maxFeePerGas = (feeData.maxFeePerGas * multiplier) / 100n
        maxPriorityFeePerGas = (feeData.maxPriorityFeePerGas * multiplier) / 100n
      } catch (error) {
        console.warn('EIP-1559 fee estimation failed, falling back to legacy gas price:', error)
      }

      return {
        gasLimit: bufferedGasLimit,
        gasPrice,
        maxFeePerGas,
        maxPriorityFeePerGas,
      }
    } catch (error) {
      console.error('Gas estimation failed:', error)
      throw new Error(`Gas estimation failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
    }
  }, [publicClient])

  // Enhanced transaction sending with retry logic
  const sendTransaction = useCallback(async (
    params: SendTransactionParameters,
    options: TransactionOptions = {},
    maxRetries: number = 3
  ): Promise<Hash> => {
    if (!walletClient) {
      throw new Error('Wallet not connected')
    }

    const txId = `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    
    setTransactions(prev => ({
      ...prev,
      [txId]: { status: 'preparing' }
    }))

    try {
      // Estimate gas if not provided
      let gasConfig = {}
      if (!options.gasLimit || !options.gasPrice) {
        const estimated = await estimateGas(params as EstimateGasParameters)
        gasConfig = {
          gas: options.gasLimit || estimated.gasLimit,
          gasPrice: options.gasPrice || estimated.gasPrice,
          maxFeePerGas: options.maxFeePerGas || estimated.maxFeePerGas,
          maxPriorityFeePerGas: options.maxPriorityFeePerGas || estimated.maxPriorityFeePerGas,
        }
      }

      const finalParams = {
        ...params,
        ...gasConfig,
        value: options.value || params.value,
      }

      let attempt = 0
      let lastError: Error

      while (attempt < maxRetries) {
        try {
          setTransactions(prev => ({
            ...prev,
            [txId]: { ...prev[txId], status: 'pending' }
          }))

          const hash = await walletClient.sendTransaction(finalParams)
          
          setTransactions(prev => ({
            ...prev,
            [txId]: { ...prev[txId], hash, status: 'confirming' }
          }))

          options.onHash?.(hash)
          
          // Wait for confirmation
          if (publicClient) {
            const receipt = await publicClient.waitForTransactionReceipt({ 
              hash,
              confirmations: 1,
              timeout: 300_000, // 5 minutes
            })
            
            setTransactions(prev => ({
              ...prev,
              [txId]: {
                ...prev[txId],
                status: receipt.status === 'success' ? 'confirmed' : 'failed',
                receipt,
                gasUsed: receipt.gasUsed,
                effectiveGasPrice: receipt.effectiveGasPrice,
              }
            }))

            options.onConfirmation?.(receipt)

            if (receipt.status === 'success') {
              toast.success('Transaction confirmed!')
            } else {
              throw new Error('Transaction failed')
            }
          }

          return hash
        } catch (error) {
          lastError = error instanceof Error ? error : new Error('Transaction failed')
          attempt++
          
          if (attempt < maxRetries) {
            console.warn(`Transaction attempt ${attempt} failed, retrying...`, error)
            // Exponential backoff
            await new Promise(resolve => setTimeout(resolve, Math.pow(2, attempt) * 1000))
          }
        }
      }

      // All retries failed
      setTransactions(prev => ({
        ...prev,
        [txId]: { ...prev[txId], status: 'failed', error: lastError }
      }))

      options.onError?.(lastError)
      toast.error(`Transaction failed: ${lastError.message}`)
      throw lastError

    } catch (error) {
      const err = error instanceof Error ? error : new Error('Transaction failed')
      
      setTransactions(prev => ({
        ...prev,
        [txId]: { ...prev[txId], status: 'failed', error: err }
      }))

      options.onError?.(err)
      throw err
    }
  }, [walletClient, publicClient, estimateGas])

  // Write contract with enhanced error handling
  const writeContract = useCallback(async (
    params: WriteContractParameters,
    options: TransactionOptions = {}
  ): Promise<Hash> => {
    if (!walletClient) {
      throw new Error('Wallet not connected')
    }

    try {
      // Simulate the contract call first to catch any revert reasons
      if (publicClient) {
        try {
          await publicClient.simulateContract(params)
        } catch (simulationError) {
          console.error('Contract simulation failed:', simulationError)
          throw new Error(`Contract call would fail: ${simulationError instanceof Error ? simulationError.message : 'Unknown error'}`)
        }
      }

      // Send the transaction
      const hash = await walletClient.writeContract(params)
      
      if (publicClient && options.onConfirmation) {
        // Wait for confirmation in background
        publicClient.waitForTransactionReceipt({ hash }).then(options.onConfirmation)
      }

      return hash
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Contract write failed')
      console.error('Contract write failed:', err)
      throw err
    }
  }, [walletClient, publicClient])

  // Batch transaction processing
  const addToBatch = useCallback((request: BatchTransactionRequest) => {
    setBatchQueue(prev => [...prev, request])
  }, [])

  const processBatch = useCallback(async (): Promise<Hash[]> => {
    if (batchQueue.length === 0 || isProcessingBatch) {
      return []
    }

    setIsProcessingBatch(true)
    const results: Hash[] = []
    const failures: string[] = []

    try {
      for (const request of batchQueue) {
        try {
          toast.loading(`Processing: ${request.description}`)
          
          let hash: Hash
          if (request.type === 'contract') {
            hash = await writeContract(request.params)
          } else {
            hash = await sendTransaction(request.params)
          }
          
          results.push(hash)
          toast.success(`Completed: ${request.description}`)
        } catch (error) {
          console.error(`Batch transaction ${request.id} failed:`, error)
          failures.push(request.description)
          toast.error(`Failed: ${request.description}`)
        }
      }

      if (failures.length > 0) {
        toast.error(`${failures.length} transaction(s) failed: ${failures.join(', ')}`)
      } else {
        toast.success(`All ${results.length} transactions completed successfully!`)
      }

      setBatchQueue([]) // Clear the batch queue
      return results
    } finally {
      setIsProcessingBatch(false)
    }
  }, [batchQueue, isProcessingBatch, writeContract, sendTransaction])

  const clearBatch = useCallback(() => {
    setBatchQueue([])
  }, [])

  // Get transaction status
  const getTransactionStatus = useCallback((hash: Hash): TransactionStatus | undefined => {
    return Object.values(transactions).find(tx => tx.hash === hash)
  }, [transactions])

  // Clean up old transactions
  useEffect(() => {
    const cleanup = setInterval(() => {
      const cutoff = Date.now() - 24 * 60 * 60 * 1000 // 24 hours ago
      setTransactions(prev => {
        const filtered = Object.entries(prev).filter(([key]) => {
          const timestamp = parseInt(key.split('_')[1])
          return timestamp > cutoff
        })
        return Object.fromEntries(filtered)
      })
    }, 60 * 60 * 1000) // Every hour

    return () => clearInterval(cleanup)
  }, [])

  return {
    // Core functions
    sendTransaction,
    writeContract,
    estimateGas,
    
    // Batch processing
    addToBatch,
    processBatch,
    clearBatch,
    batchQueue,
    isProcessingBatch,
    
    // Status tracking
    transactions,
    getTransactionStatus,
    
    // Utilities
    isConnected: !!address && !!walletClient,
  }
}

export default useTransactions