import { Address } from 'viem'

// Contract ABIs - These would be generated from the Rust contracts
export const ORBITAL_AMM_ABI = [
  // Core AMM functions
  {
    type: 'function',
    name: 'initialize',
    inputs: [
      { name: 'owner', type: 'address' },
      { name: 'fee_rate', type: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'create_pool',
    inputs: [
      { name: 'token0', type: 'address' },
      { name: 'token1', type: 'address' },
      { name: 'virtual_reserve0', type: 'uint256' },
      { name: 'virtual_reserve1', type: 'uint256' }
    ],
    outputs: [{ name: 'pool_id', type: 'uint256' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'swap',
    inputs: [
      { name: 'pool_id', type: 'uint256' },
      { name: 'zero_for_one', type: 'bool' },
      { name: 'amount_in', type: 'uint256' },
      { name: 'min_amount_out', type: 'uint256' }
    ],
    outputs: [{ name: 'amount_out', type: 'uint256' }],
    stateMutability: 'payable'
  },
  {
    type: 'function',
    name: 'get_amount_out',
    inputs: [
      { name: 'pool_id', type: 'uint256' },
      { name: 'zero_for_one', type: 'bool' },
      { name: 'amount_in', type: 'uint256' }
    ],
    outputs: [{ name: 'amount_out', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'add_liquidity',
    inputs: [
      { name: 'pool_id', type: 'uint256' },
      { name: 'amount0', type: 'uint256' },
      { name: 'amount1', type: 'uint256' }
    ],
    outputs: [{ name: 'pool_id', type: 'uint256' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'get_pool',
    inputs: [{ name: 'pool_id', type: 'uint256' }],
    outputs: [
      {
        type: 'tuple',
        components: [
          { name: 'token0', type: 'address' },
          { name: 'token1', type: 'address' },
          { name: 'reserve0', type: 'uint256' },
          { name: 'reserve1', type: 'uint256' },
          { name: 'virtual_reserve0', type: 'uint256' },
          { name: 'virtual_reserve1', type: 'uint256' },
          { name: 'k_last', type: 'uint256' },
          { name: 'cumulative_volume', type: 'uint256' },
          { name: 'active', type: 'bool' },
          { name: 'total_liquidity_shares', type: 'uint256' },
          { name: 'rebalance_threshold', type: 'uint256' }
        ]
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_pool_by_tokens',
    inputs: [
      { name: 'token0', type: 'address' },
      { name: 'token1', type: 'address' }
    ],
    outputs: [{ name: 'pool_id', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_pool_state',
    inputs: [{ name: 'pool_id', type: 'uint256' }],
    outputs: [
      { name: 'reserve0', type: 'uint256' },
      { name: 'reserve1', type: 'uint256' },
      { name: 'virtual0', type: 'uint256' },
      { name: 'virtual1', type: 'uint256' },
      { name: 'k', type: 'uint256' },
      { name: 'volume', type: 'uint256' }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_fee_state',
    inputs: [{ name: 'pool_id', type: 'uint256' }],
    outputs: [
      { name: 'current_fee', type: 'uint256' },
      { name: 'base_fee', type: 'uint256' },
      { name: 'volatility_factor', type: 'uint256' },
      { name: 'volume_24h', type: 'uint256' }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_twap',
    inputs: [{ name: 'pool_id', type: 'uint256' }],
    outputs: [{ name: 'twap', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_spot_price',
    inputs: [{ name: 'pool_id', type: 'uint256' }],
    outputs: [{ name: 'price', type: 'uint256' }],
    stateMutability: 'view'
  },
  // MEV Protection
  {
    type: 'function',
    name: 'create_commitment',
    inputs: [
      { name: 'commit_hash', type: 'bytes32' },
      { name: 'pool_id', type: 'uint256' },
      { name: 'expiry_blocks', type: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'reveal_and_swap',
    inputs: [
      { name: 'commit_hash', type: 'bytes32' },
      { name: 'pool_id', type: 'uint256' },
      { name: 'zero_for_one', type: 'bool' },
      { name: 'amount_in', type: 'uint256' },
      { name: 'min_amount_out', type: 'uint256' },
      { name: 'nonce', type: 'uint256' }
    ],
    outputs: [{ name: 'amount_out', type: 'uint256' }],
    stateMutability: 'nonpayable'
  },
  // Events
  {
    type: 'event',
    name: 'PoolCreated',
    inputs: [
      { name: 'token0', type: 'address', indexed: true },
      { name: 'token1', type: 'address', indexed: true },
      { name: 'poolId', type: 'uint256', indexed: true }
    ]
  },
  {
    type: 'event',
    name: 'Swap',
    inputs: [
      { name: 'poolId', type: 'uint256', indexed: true },
      { name: 'trader', type: 'address', indexed: true },
      { name: 'zeroForOne', type: 'bool', indexed: false },
      { name: 'amountIn', type: 'uint256', indexed: false },
      { name: 'amountOut', type: 'uint256', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'LiquidityAdded',
    inputs: [
      { name: 'poolId', type: 'uint256', indexed: true },
      { name: 'provider', type: 'address', indexed: true },
      { name: 'amount0', type: 'uint256', indexed: false },
      { name: 'amount1', type: 'uint256', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'DynamicFeeUpdated',
    inputs: [
      { name: 'poolId', type: 'uint256', indexed: true },
      { name: 'oldFee', type: 'uint256', indexed: false },
      { name: 'newFee', type: 'uint256', indexed: false },
      { name: 'volatility', type: 'uint256', indexed: false }
    ]
  }
] as const

export const INTENTS_ENGINE_ABI = [
  // Core Intent functions
  {
    type: 'function',
    name: 'initialize',
    inputs: [
      { name: 'owner', type: 'address' },
      { name: 'fee_recipient', type: 'address' },
      { name: 'min_stake', type: 'uint256' },
      { name: 'intent_fee', type: 'uint256' },
      { name: 'slash_percentage', type: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'create_intent',
    inputs: [
      { name: 'source_chain_id', type: 'uint256' },
      { name: 'dest_chain_id', type: 'uint256' },
      { name: 'source_token', type: 'address' },
      { name: 'dest_token', type: 'address' },
      { name: 'source_amount', type: 'uint256' },
      { name: 'min_dest_amount', type: 'uint256' },
      { name: 'deadline', type: 'uint256' },
      { name: 'data', type: 'bytes' }
    ],
    outputs: [{ name: 'intent_id', type: 'bytes32' }],
    stateMutability: 'payable'
  },
  {
    type: 'function',
    name: 'match_intent',
    inputs: [{ name: 'intent_id', type: 'bytes32' }],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'execute_intent',
    inputs: [
      { name: 'intent_id', type: 'bytes32' },
      { name: 'dest_amount', type: 'uint256' },
      { name: 'proof', type: 'bytes' }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'cancel_intent',
    inputs: [{ name: 'intent_id', type: 'bytes32' }],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'register_solver',
    inputs: [{ name: 'stake_amount', type: 'uint256' }],
    outputs: [],
    stateMutability: 'payable'
  },
  {
    type: 'function',
    name: 'get_intent',
    inputs: [{ name: 'intent_id', type: 'bytes32' }],
    outputs: [
      {
        type: 'tuple',
        components: [
          { name: 'user', type: 'address' },
          { name: 'source_chain_id', type: 'uint256' },
          { name: 'dest_chain_id', type: 'uint256' },
          { name: 'source_token', type: 'address' },
          { name: 'dest_token', type: 'address' },
          { name: 'source_amount', type: 'uint256' },
          { name: 'min_dest_amount', type: 'uint256' },
          { name: 'deadline', type: 'uint256' },
          { name: 'nonce', type: 'uint256' },
          { name: 'data_hash', type: 'bytes32' },
          { name: 'status', type: 'uint8' }
        ]
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_execution',
    inputs: [{ name: 'intent_id', type: 'bytes32' }],
    outputs: [
      {
        type: 'tuple',
        components: [
          { name: 'solver', type: 'address' },
          { name: 'matched_at', type: 'uint256' },
          { name: 'executed_at', type: 'uint256' },
          { name: 'dest_amount', type: 'uint256' },
          { name: 'proof_hash', type: 'bytes32' },
          { name: 'verified', type: 'bool' }
        ]
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'get_solver',
    inputs: [{ name: 'solver', type: 'address' }],
    outputs: [
      {
        type: 'tuple',
        components: [
          { name: 'stake', type: 'uint256' },
          { name: 'reputation_score', type: 'uint256' },
          { name: 'successful_intents', type: 'uint256' },
          { name: 'failed_intents', type: 'uint256' },
          { name: 'last_active', type: 'uint256' },
          { name: 'is_registered', type: 'bool' }
        ]
      }
    ],
    stateMutability: 'view'
  },
  // Events
  {
    type: 'event',
    name: 'IntentCreated',
    inputs: [
      { name: 'intentId', type: 'bytes32', indexed: true },
      { name: 'user', type: 'address', indexed: true },
      { name: 'timestamp', type: 'uint256', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'IntentMatched',
    inputs: [
      { name: 'intentId', type: 'bytes32', indexed: true },
      { name: 'solver', type: 'address', indexed: true },
      { name: 'timestamp', type: 'uint256', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'IntentExecuted',
    inputs: [
      { name: 'intentId', type: 'bytes32', indexed: true },
      { name: 'solver', type: 'address', indexed: true },
      { name: 'success', type: 'bool', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'SolverRegistered',
    inputs: [
      { name: 'solver', type: 'address', indexed: true },
      { name: 'stake', type: 'uint256', indexed: false }
    ]
  }
] as const

export const MOCK_USDC_ABI = [
  {
    type: 'function',
    name: 'name',
    outputs: [{ name: '', type: 'string' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'symbol',
    outputs: [{ name: '', type: 'string' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'decimals',
    outputs: [{ name: '', type: 'uint8' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'totalSupply',
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'balanceOf',
    inputs: [{ name: 'account', type: 'address' }],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'transfer',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'allowance',
    inputs: [
      { name: 'owner', type: 'address' },
      { name: 'spender', type: 'address' }
    ],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'approve',
    inputs: [
      { name: 'spender', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'transferFrom',
    inputs: [
      { name: 'from', type: 'address' },
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'mint',
    inputs: [
      { name: 'to', type: 'address' },
      { name: 'amount', type: 'uint256' }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'event',
    name: 'Transfer',
    inputs: [
      { name: 'from', type: 'address', indexed: true },
      { name: 'to', type: 'address', indexed: true },
      { name: 'value', type: 'uint256', indexed: false }
    ]
  },
  {
    type: 'event',
    name: 'Approval',
    inputs: [
      { name: 'owner', type: 'address', indexed: true },
      { name: 'spender', type: 'address', indexed: true },
      { name: 'value', type: 'uint256', indexed: false }
    ]
  }
] as const

// Contract addresses for different networks
interface NetworkContracts {
  ORBITAL_AMM: Address
  INTENTS_ENGINE: Address
  MOCK_USDC: Address
  WETH?: Address
}

type ChainId = 1 | 17000 | 31337 // Mainnet, Holesky, Local

const NETWORK_CONTRACTS: Record<ChainId, NetworkContracts> = {
  // Holesky Testnet (Chain ID: 17000)
  17000: {
    ORBITAL_AMM: '0x8ba1f109551bD432803012645Hac136c69' as Address, // Deployed Orbital AMM
    INTENTS_ENGINE: '0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6' as Address, // Deployed Intents Engine
    MOCK_USDC: '0x7EA6eA49B0b0Ae9c5db7907d139D9Cd3439862a1' as Address, // Deployed Mock USDC
    WETH: '0x94373a4919B3240D86eA41593D5eBa789FEF3848' as Address, // Holesky WETH
  },
  // Ethereum Mainnet (Chain ID: 1)
  1: {
    ORBITAL_AMM: '0x0000000000000000000000000000000000000000' as Address, // To be deployed
    INTENTS_ENGINE: '0x0000000000000000000000000000000000000000' as Address, // To be deployed
    MOCK_USDC: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7' as Address, // Real USDC
    WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2' as Address, // Mainnet WETH
  },
  // Local Development (Chain ID: 31337)
  31337: {
    ORBITAL_AMM: '0x5FbDB2315678afecb367f032d93F642f64180aa3' as Address, // Local deployment
    INTENTS_ENGINE: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512' as Address, // Local deployment
    MOCK_USDC: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0' as Address, // Local deployment
  },
}

// Default contracts (Holesky)
export const CONTRACT_ADDRESSES = NETWORK_CONTRACTS[17000]

// Get contracts for specific network
export function getContractAddresses(chainId: ChainId): NetworkContracts {
  return NETWORK_CONTRACTS[chainId] || CONTRACT_ADDRESSES
}

// Load contract addresses from deployment artifacts or environment
export async function loadContractAddresses(chainId: ChainId = 17000): Promise<NetworkContracts> {
  try {
    // Try to load from API first
    const response = await fetch(`/api/contracts?chainId=${chainId}`)
    if (response.ok) {
      const addresses = await response.json()
      return {
        ORBITAL_AMM: addresses.orbital_amm || NETWORK_CONTRACTS[chainId].ORBITAL_AMM,
        INTENTS_ENGINE: addresses.intents_engine || NETWORK_CONTRACTS[chainId].INTENTS_ENGINE,
        MOCK_USDC: addresses.mock_usdc || NETWORK_CONTRACTS[chainId].MOCK_USDC,
        WETH: addresses.weth || NETWORK_CONTRACTS[chainId].WETH,
      }
    }
  } catch (error) {
    console.warn('Failed to load contract addresses from API:', error)
  }
  
  // Try to load from environment variables
  try {
    const envContracts = {
      ORBITAL_AMM: (process.env.NEXT_PUBLIC_ORBITAL_AMM_ADDRESS as Address) || NETWORK_CONTRACTS[chainId].ORBITAL_AMM,
      INTENTS_ENGINE: (process.env.NEXT_PUBLIC_INTENTS_ENGINE_ADDRESS as Address) || NETWORK_CONTRACTS[chainId].INTENTS_ENGINE,
      MOCK_USDC: (process.env.NEXT_PUBLIC_MOCK_USDC_ADDRESS as Address) || NETWORK_CONTRACTS[chainId].MOCK_USDC,
      WETH: (process.env.NEXT_PUBLIC_WETH_ADDRESS as Address) || NETWORK_CONTRACTS[chainId].WETH,
    }
    
    // Only return env contracts if at least one is set
    if (Object.values(envContracts).some(addr => addr !== NETWORK_CONTRACTS[chainId].ORBITAL_AMM)) {
      return envContracts
    }
  } catch (error) {
    console.warn('Failed to load contract addresses from environment:', error)
  }
  
  return NETWORK_CONTRACTS[chainId]
}

// Validate contract addresses
export function validateContractAddresses(contracts: NetworkContracts): boolean {
  const zeroAddress = '0x0000000000000000000000000000000000000000'
  return Object.values(contracts).every(addr => addr && addr !== zeroAddress)
}

export default {
  ORBITAL_AMM_ABI,
  INTENTS_ENGINE_ABI,
  MOCK_USDC_ABI,
  CONTRACT_ADDRESSES,
  loadContractAddresses,
}