import { useState, useEffect, useCallback } from 'react'
import { useContractRead, useContractWrite, usePrepareContractWrite, usePublicClient } from 'wagmi'
import { Address, formatUnits, parseUnits, Hash, parseEther } from 'viem'
import { 
  ORBITAL_AMM_ABI, 
  INTENTS_ENGINE_ABI, 
  MOCK_USDC_ABI,
  loadContractAddresses,
  getContractAddresses 
} from '@/lib/contracts'
import { useNetwork } from 'wagmi'
import { SwapParams, Pool, Intent, Solver } from '@/types'
import useTransactions from './useTransactions'
import toast from 'react-hot-toast'

// Hook to get contract addresses with network awareness
export function useContractAddresses() {
  const { chain } = useNetwork()
  const [addresses, setAddresses] = useState({
    orbitalAMM: '0x0000000000000000000000000000000000000000' as Address,
    intentsEngine: '0x0000000000000000000000000000000000000000' as Address,
    mockUSDC: '0x0000000000000000000000000000000000000000' as Address,
    weth: '0x0000000000000000000000000000000000000000' as Address,
  })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const loadAddresses = async () => {
      try {
        setLoading(true)
        setError(null)
        
        const chainId = chain?.id || 17000 // Default to Holesky
        const contracts = await loadContractAddresses(chainId as 1 | 17000 | 31337)
        
        setAddresses({
          orbitalAMM: contracts.ORBITAL_AMM,
          intentsEngine: contracts.INTENTS_ENGINE,
          mockUSDC: contracts.MOCK_USDC,
          weth: contracts.WETH || '0x0000000000000000000000000000000000000000' as Address,
        })
      } catch (err) {
        console.error('Failed to load contract addresses:', err)
        setError('Failed to load contract addresses')
        
        // Fallback to network defaults
        const fallback = getContractAddresses((chain?.id || 17000) as 1 | 17000 | 31337)
        setAddresses({
          orbitalAMM: fallback.ORBITAL_AMM,
          intentsEngine: fallback.INTENTS_ENGINE,
          mockUSDC: fallback.MOCK_USDC,
          weth: fallback.WETH || '0x0000000000000000000000000000000000000000' as Address,
        })
      } finally {
        setLoading(false)
      }
    }

    loadAddresses()
  }, [chain?.id])

  return { addresses, loading, error }
}

// Orbital AMM Hooks
export function usePoolById(poolId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_pool',
    args: [BigInt(poolId)],
    enabled: !!addresses.orbitalAMM && !!poolId,
  })

  return {
    pool: data as Pool | undefined,
    isError,
    isLoading,
    refetch,
  }
}

export function usePoolByTokens(token0: Address, token1: Address) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_pool_by_tokens',
    args: [token0, token1],
    enabled: !!addresses.orbitalAMM && !!token0 && !!token1,
  })

  const poolId = data?.toString()
  const { pool, refetch } = usePoolById(poolId || '0')

  return {
    poolId,
    pool,
    isError,
    isLoading,
    refetch,
  }
}

export function useSwapQuote(
  poolId: string,
  zeroForOne: boolean,
  amountIn: string,
  enabled: boolean = true
) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_amount_out',
    args: [BigInt(poolId), zeroForOne, BigInt(amountIn || 0)],
    enabled: !!addresses.orbitalAMM && !!poolId && !!amountIn && enabled,
    watch: true,
  })

  return {
    amountOut: data?.toString() || '0',
    isError,
    isLoading,
    refetch,
  }
}

export function useSwap() {
  const { addresses } = useContractAddresses()
  const { writeContract, estimateGas } = useTransactions()
  const [isLoading, setIsLoading] = useState(false)
  const [txHash, setTxHash] = useState<Hash | null>(null)
  const [isSuccess, setIsSuccess] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const swap = useCallback(async (params: SwapParams, value?: bigint) => {
    if (!addresses.orbitalAMM) {
      toast.error('Orbital AMM contract not found')
      return
    }

    try {
      setIsLoading(true)
      setError(null)
      setIsSuccess(false)
      
      // First estimate gas
      const gasEstimate = await estimateGas({
        to: addresses.orbitalAMM,
        data: '0x', // This would be encoded function call
        value: value || 0n,
      })
      
      console.log('Swap parameters:', {
        poolId: params.poolId,
        zeroForOne: params.zeroForOne,
        amountIn: params.amountIn,
        minAmountOut: params.minAmountOut,
        value: value?.toString(),
        gasEstimate: gasEstimate.gasLimit.toString(),
      })

      // Execute the swap
      const hash = await writeContract({
        address: addresses.orbitalAMM,
        abi: ORBITAL_AMM_ABI,
        functionName: 'swap',
        args: [
          BigInt(params.poolId),
          params.zeroForOne,
          BigInt(params.amountIn),
          BigInt(params.minAmountOut),
        ],
        value,
      }, {
        gasLimit: gasEstimate.gasLimit,
        maxFeePerGas: gasEstimate.maxFeePerGas,
        maxPriorityFeePerGas: gasEstimate.maxPriorityFeePerGas,
        onHash: (hash) => {
          setTxHash(hash)
          toast.loading('Swap transaction submitted...')
        },
        onConfirmation: (receipt) => {
          setIsSuccess(receipt.status === 'success')
          if (receipt.status === 'success') {
            toast.success('Swap completed successfully!')
          } else {
            toast.error('Swap transaction failed')
          }
        },
        onError: (error) => {
          setError(error)
          toast.error(`Swap failed: ${error.message}`)
        },
      })
      
      return hash
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Swap failed')
      setError(error)
      console.error('Swap failed:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [addresses.orbitalAMM, writeContract, estimateGas])

  return {
    swap,
    data: txHash,
    isLoading,
    isSuccess,
    error,
  }
}

export function usePoolState(poolId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_pool_state',
    args: [BigInt(poolId)],
    enabled: !!addresses.orbitalAMM && !!poolId,
    watch: true,
  })

  return {
    poolState: data ? {
      reserve0: data[0].toString(),
      reserve1: data[1].toString(),
      virtual0: data[2].toString(),
      virtual1: data[3].toString(),
      k: data[4].toString(),
      volume: data[5].toString(),
    } : undefined,
    isError,
    isLoading,
    refetch,
  }
}

export function useFeeState(poolId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_fee_state',
    args: [BigInt(poolId)],
    enabled: !!addresses.orbitalAMM && !!poolId,
    watch: true,
  })

  return {
    feeState: data ? {
      currentFee: data[0].toString(),
      baseFee: data[1].toString(),
      volatilityFactor: data[2].toString(),
      volume24h: data[3].toString(),
    } : undefined,
    isError,
    isLoading,
    refetch,
  }
}

export function useTWAP(poolId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_twap',
    args: [BigInt(poolId)],
    enabled: !!addresses.orbitalAMM && !!poolId,
    watch: true,
  })

  return {
    twap: data?.toString() || '0',
    isError,
    isLoading,
    refetch,
  }
}

export function useSpotPrice(poolId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'get_spot_price',
    args: [BigInt(poolId)],
    enabled: !!addresses.orbitalAMM && !!poolId,
    watch: true,
  })

  return {
    spotPrice: data?.toString() || '0',
    isError,
    isLoading,
    refetch,
  }
}

// Intents Engine Hooks
export function useCreateIntent() {
  const { addresses } = useContractAddresses()
  const { writeContract, estimateGas } = useTransactions()
  const [isLoading, setIsLoading] = useState(false)
  const [txHash, setTxHash] = useState<Hash | null>(null)
  const [isSuccess, setIsSuccess] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [intentId, setIntentId] = useState<string | null>(null)

  const createIntent = useCallback(async (
    sourceChainId: number,
    destChainId: number,
    sourceToken: Address,
    destToken: Address,
    sourceAmount: string,
    minDestAmount: string,
    deadline: number,
    data: string = '0x',
    value?: bigint
  ): Promise<Hash> => {
    if (!addresses.intentsEngine) {
      throw new Error('Intents Engine contract not found')
    }

    try {
      setIsLoading(true)
      setError(null)
      setIsSuccess(false)
      setIntentId(null)
      
      // Validate parameters
      if (sourceChainId === destChainId) {
        throw new Error('Source and destination chains must be different')
      }
      
      if (BigInt(sourceAmount) <= 0n) {
        throw new Error('Source amount must be greater than 0')
      }
      
      if (BigInt(minDestAmount) <= 0n) {
        throw new Error('Minimum destination amount must be greater than 0')
      }
      
      if (deadline <= Math.floor(Date.now() / 1000)) {
        throw new Error('Deadline must be in the future')
      }

      // Estimate gas
      const gasEstimate = await estimateGas({
        to: addresses.intentsEngine,
        data: '0x', // This would be encoded function call
        value: value || 0n,
      })
      
      console.log('Intent creation parameters:', {
        sourceChainId,
        destChainId,
        sourceToken,
        destToken,
        sourceAmount,
        minDestAmount,
        deadline,
        data,
        value: value?.toString(),
        gasEstimate: gasEstimate.gasLimit.toString(),
      })

      // Execute intent creation
      const hash = await writeContract({
        address: addresses.intentsEngine,
        abi: INTENTS_ENGINE_ABI,
        functionName: 'create_intent',
        args: [
          BigInt(sourceChainId),
          BigInt(destChainId),
          sourceToken,
          destToken,
          BigInt(sourceAmount),
          BigInt(minDestAmount),
          BigInt(deadline),
          data as `0x${string}`,
        ],
        value,
      }, {
        gasLimit: gasEstimate.gasLimit,
        maxFeePerGas: gasEstimate.maxFeePerGas,
        maxPriorityFeePerGas: gasEstimate.maxPriorityFeePerGas,
        onHash: (hash) => {
          setTxHash(hash)
          toast.loading('Intent creation transaction submitted...')
        },
        onConfirmation: (receipt) => {
          setIsSuccess(receipt.status === 'success')
          
          if (receipt.status === 'success') {
            // Extract intent ID from logs
            try {
              const intentCreatedLog = receipt.logs.find(
                log => log.topics[0] === '0x...' // IntentCreated event signature
              )
              if (intentCreatedLog && intentCreatedLog.topics[1]) {
                setIntentId(intentCreatedLog.topics[1])
              }
            } catch (err) {
              console.warn('Failed to extract intent ID from logs:', err)
            }
            
            toast.success('Intent created successfully!')
          } else {
            toast.error('Intent creation failed')
          }
        },
        onError: (error) => {
          setError(error)
          toast.error(`Intent creation failed: ${error.message}`)
        },
      })
      
      return hash
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Intent creation failed')
      setError(error)
      console.error('Intent creation failed:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [addresses.intentsEngine, writeContract, estimateGas])

  return {
    createIntent,
    data: txHash,
    isLoading,
    isSuccess,
    error,
    intentId,
  }
}

export function useIntent(intentId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.intentsEngine,
    abi: INTENTS_ENGINE_ABI,
    functionName: 'get_intent',
    args: [intentId as `0x${string}`],
    enabled: !!addresses.intentsEngine && !!intentId,
  })

  return {
    intent: data as Intent | undefined,
    isError,
    isLoading,
    refetch,
  }
}

export function useIntentExecution(intentId: string) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.intentsEngine,
    abi: INTENTS_ENGINE_ABI,
    functionName: 'get_execution',
    args: [intentId as `0x${string}`],
    enabled: !!addresses.intentsEngine && !!intentId,
  })

  return {
    execution: data,
    isError,
    isLoading,
    refetch,
  }
}

export function useSolver(solverAddress: Address) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: addresses.intentsEngine,
    abi: INTENTS_ENGINE_ABI,
    functionName: 'get_solver',
    args: [solverAddress],
    enabled: !!addresses.intentsEngine && !!solverAddress,
  })

  return {
    solver: data as Solver | undefined,
    isError,
    isLoading,
    refetch,
  }
}

export function useCancelIntent() {
  const { addresses } = useContractAddresses()
  
  const { config, error: prepareError } = usePrepareContractWrite({
    address: addresses.intentsEngine,
    abi: INTENTS_ENGINE_ABI,
    functionName: 'cancel_intent',
  })

  const { 
    data, 
    isLoading, 
    isSuccess, 
    write,
    error: writeError 
  } = useContractWrite(config)

  const cancelIntent = (intentId: string) => {
    if (!write) return
    
    write({
      args: [intentId as `0x${string}`],
    })
  }

  return {
    cancelIntent,
    data,
    isLoading,
    isSuccess,
    error: prepareError || writeError,
  }
}

// Token Hooks
export function useTokenBalance(tokenAddress: Address, userAddress?: Address) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: tokenAddress === '0x0000000000000000000000000000000000000000' 
      ? addresses.mockUSDC 
      : tokenAddress,
    abi: MOCK_USDC_ABI,
    functionName: 'balanceOf',
    args: userAddress ? [userAddress] : undefined,
    enabled: !!tokenAddress && !!userAddress,
    watch: true,
  })

  return {
    balance: data?.toString() || '0',
    isError,
    isLoading,
    refetch,
  }
}

export function useTokenAllowance(
  tokenAddress: Address, 
  ownerAddress?: Address, 
  spenderAddress?: Address
) {
  const { addresses } = useContractAddresses()
  
  const { data, isError, isLoading, refetch } = useContractRead({
    address: tokenAddress === '0x0000000000000000000000000000000000000000' 
      ? addresses.mockUSDC 
      : tokenAddress,
    abi: MOCK_USDC_ABI,
    functionName: 'allowance',
    args: ownerAddress && spenderAddress ? [ownerAddress, spenderAddress] : undefined,
    enabled: !!tokenAddress && !!ownerAddress && !!spenderAddress,
    watch: true,
  })

  return {
    allowance: data?.toString() || '0',
    isError,
    isLoading,
    refetch,
  }
}

export function useTokenApprove() {
  const { writeContract, estimateGas } = useTransactions()
  const [isLoading, setIsLoading] = useState(false)
  const [txHash, setTxHash] = useState<Hash | null>(null)
  const [isSuccess, setIsSuccess] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  const approve = useCallback(async (
    tokenAddress: Address, 
    spenderAddress: Address, 
    amount: string
  ): Promise<Hash> => {
    try {
      setIsLoading(true)
      setError(null)
      setIsSuccess(false)
      
      // Validate parameters
      if (!tokenAddress || tokenAddress === '0x0000000000000000000000000000000000000000') {
        throw new Error('Invalid token address')
      }
      
      if (!spenderAddress || spenderAddress === '0x0000000000000000000000000000000000000000') {
        throw new Error('Invalid spender address')
      }
      
      if (BigInt(amount) < 0n) {
        throw new Error('Amount cannot be negative')
      }

      // Estimate gas
      const gasEstimate = await estimateGas({
        to: tokenAddress,
        data: '0x', // This would be encoded approve function call
        value: 0n,
      })
      
      console.log('Token approval parameters:', {
        tokenAddress,
        spenderAddress,
        amount,
        gasEstimate: gasEstimate.gasLimit.toString(),
      })

      // Execute approval
      const hash = await writeContract({
        address: tokenAddress,
        abi: MOCK_USDC_ABI,
        functionName: 'approve',
        args: [spenderAddress, BigInt(amount)],
      }, {
        gasLimit: gasEstimate.gasLimit,
        maxFeePerGas: gasEstimate.maxFeePerGas,
        maxPriorityFeePerGas: gasEstimate.maxPriorityFeePerGas,
        onHash: (hash) => {
          setTxHash(hash)
          toast.loading('Approval transaction submitted...')
        },
        onConfirmation: (receipt) => {
          setIsSuccess(receipt.status === 'success')
          
          if (receipt.status === 'success') {
            toast.success('Token approval successful!')
          } else {
            toast.error('Token approval failed')
          }
        },
        onError: (error) => {
          setError(error)
          toast.error(`Approval failed: ${error.message}`)
        },
      })
      
      return hash
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Token approval failed')
      setError(error)
      console.error('Token approval failed:', error)
      throw error
    } finally {
      setIsLoading(false)
    }
  }, [writeContract, estimateGas])

  return {
    approve,
    data: txHash,
    isLoading,
    isSuccess,
    error,
  }
}

export default {
  useContractAddresses,
  usePoolById,
  usePoolByTokens,
  useSwapQuote,
  useSwap,
  usePoolState,
  useFeeState,
  useTWAP,
  useSpotPrice,
  useCreateIntent,
  useIntent,
  useIntentExecution,
  useSolver,
  useCancelIntent,
  useTokenBalance,
  useTokenAllowance,
  useTokenApprove,
}