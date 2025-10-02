import { useState, useEffect } from 'react'
import { useContractRead, useContractWrite, usePrepareContractWrite } from 'wagmi'
import { Address, formatUnits, parseUnits } from 'viem'
import { 
  ORBITAL_AMM_ABI, 
  INTENTS_ENGINE_ABI, 
  MOCK_USDC_ABI,
  loadContractAddresses,
  getContractAddresses 
} from '@/lib/contracts'
import { useNetwork } from 'wagmi'
import { SwapParams, Pool, Intent, Solver } from '@/types'

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
  
  const { config, error: prepareError } = usePrepareContractWrite({
    address: addresses.orbitalAMM,
    abi: ORBITAL_AMM_ABI,
    functionName: 'swap',
  })

  const { 
    data, 
    isLoading, 
    isSuccess, 
    write,
    error: writeError 
  } = useContractWrite(config)

  const swap = (params: SwapParams, value?: bigint) => {
    if (!write) return
    
    write({
      args: [
        BigInt(params.poolId),
        params.zeroForOne,
        BigInt(params.amountIn),
        BigInt(params.minAmountOut),
      ],
      value,
    })
  }

  return {
    swap,
    data,
    isLoading,
    isSuccess,
    error: prepareError || writeError,
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
  
  const { config, error: prepareError } = usePrepareContractWrite({
    address: addresses.intentsEngine,
    abi: INTENTS_ENGINE_ABI,
    functionName: 'create_intent',
  })

  const { 
    data, 
    isLoading, 
    isSuccess, 
    write,
    error: writeError 
  } = useContractWrite(config)

  const createIntent = (
    sourceChainId: number,
    destChainId: number,
    sourceToken: Address,
    destToken: Address,
    sourceAmount: string,
    minDestAmount: string,
    deadline: number,
    data: string = '0x',
    value?: bigint
  ) => {
    if (!write) return
    
    write({
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
    })
  }

  return {
    createIntent,
    data,
    isLoading,
    isSuccess,
    error: prepareError || writeError,
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
  const { config, error: prepareError } = usePrepareContractWrite({
    abi: MOCK_USDC_ABI,
    functionName: 'approve',
  })

  const { 
    data, 
    isLoading, 
    isSuccess, 
    write,
    error: writeError 
  } = useContractWrite(config)

  const approve = (tokenAddress: Address, spenderAddress: Address, amount: string) => {
    if (!write) return
    
    write({
      address: tokenAddress,
      args: [spenderAddress, BigInt(amount)],
    })
  }

  return {
    approve,
    data,
    isLoading,
    isSuccess,
    error: prepareError || writeError,
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