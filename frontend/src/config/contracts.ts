import { Address } from 'viem'

// Real deployed contract addresses on Holesky testnet
// These would be updated after actual deployment
export const HOLESKY_CONTRACTS = {
  // Core Protocol Contracts
  ORBITAL_AMM: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6' as Address, // Mock deployment address
  INTENTS_ENGINE: '0x1234567890123456789012345678901234567890' as Address, // Mock deployment address
  
  // Token Contracts
  MOCK_USDC: '0x9876543210987654321098765432109876543210' as Address, // Mock USDC for testing
  WETH: '0x94373a4919B3240D86eA41593D5eBa789FEF3848' as Address, // Real Holesky WETH
  
  // Bridge Contracts
  CROSS_CHAIN_BRIDGE: '0xabcdefabcdefabcdefabcdefabcdefabcdefabcd' as Address, // Mock bridge
  
  // Utility Contracts
  MULTICALL: '0xca11bde05977b3631167028862be2a173976ca11' as Address, // Standard multicall
} as const

// Mainnet addresses (for production)
export const MAINNET_CONTRACTS = {
  ORBITAL_AMM: '0x0000000000000000000000000000000000000000' as Address,
  INTENTS_ENGINE: '0x0000000000000000000000000000000000000000' as Address,
  MOCK_USDC: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7' as Address, // Real USDC
  WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2' as Address, // Real WETH
  CROSS_CHAIN_BRIDGE: '0x0000000000000000000000000000000000000000' as Address,
  MULTICALL: '0xca11bde05977b3631167028862be2a173976ca11' as Address,
} as const

// Local development addresses
export const LOCAL_CONTRACTS = {
  ORBITAL_AMM: '0x5FbDB2315678afecb367f032d93F642f64180aa3' as Address,
  INTENTS_ENGINE: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512' as Address,
  MOCK_USDC: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0' as Address,
  WETH: '0x0000000000000000000000000000000000000000' as Address,
  CROSS_CHAIN_BRIDGE: '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9' as Address,
  MULTICALL: '0x0000000000000000000000000000000000000000' as Address,
} as const

// Chain-specific contract mapping
export const CONTRACTS_BY_CHAIN = {
  1: MAINNET_CONTRACTS,      // Ethereum Mainnet
  17000: HOLESKY_CONTRACTS,  // Holesky Testnet
  31337: LOCAL_CONTRACTS,    // Local Development
} as const

// Default to Holesky for development
export const DEFAULT_CONTRACTS = HOLESKY_CONTRACTS

// Contract verification status
export const CONTRACT_VERIFICATION = {
  [HOLESKY_CONTRACTS.ORBITAL_AMM]: {
    verified: false,
    deployedAt: 'TBD',
    deployer: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    gasUsed: '0',
  },
  [HOLESKY_CONTRACTS.INTENTS_ENGINE]: {
    verified: false,
    deployedAt: 'TBD',
    deployer: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    gasUsed: '0',
  },
  [HOLESKY_CONTRACTS.MOCK_USDC]: {
    verified: false,
    deployedAt: 'TBD',
    deployer: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6',
    gasUsed: '0',
  },
} as const

// Helper function to get contracts for a specific chain
export function getContractsForChain(chainId: number) {
  return CONTRACTS_BY_CHAIN[chainId as keyof typeof CONTRACTS_BY_CHAIN] || DEFAULT_CONTRACTS
}

// Helper function to validate contract addresses
export function validateContractAddress(address: Address): boolean {
  return address !== '0x0000000000000000000000000000000000000000' && /^0x[a-fA-F0-9]{40}$/.test(address)
}

// Token configurations for different chains
export const TOKEN_CONFIGS = {
  1: {
    ETH: { address: '0x0000000000000000000000000000000000000000' as Address, decimals: 18, symbol: 'ETH' },
    USDC: { address: MAINNET_CONTRACTS.MOCK_USDC, decimals: 6, symbol: 'USDC' },
    WETH: { address: MAINNET_CONTRACTS.WETH, decimals: 18, symbol: 'WETH' },
  },
  17000: {
    ETH: { address: '0x0000000000000000000000000000000000000000' as Address, decimals: 18, symbol: 'ETH' },
    USDC: { address: HOLESKY_CONTRACTS.MOCK_USDC, decimals: 6, symbol: 'USDC' },
    WETH: { address: HOLESKY_CONTRACTS.WETH, decimals: 18, symbol: 'WETH' },
  },
  31337: {
    ETH: { address: '0x0000000000000000000000000000000000000000' as Address, decimals: 18, symbol: 'ETH' },
    USDC: { address: LOCAL_CONTRACTS.MOCK_USDC, decimals: 6, symbol: 'USDC' },
    WETH: { address: LOCAL_CONTRACTS.WETH, decimals: 18, symbol: 'WETH' },
  },
} as const

export default {
  HOLESKY_CONTRACTS,
  MAINNET_CONTRACTS,
  LOCAL_CONTRACTS,
  CONTRACTS_BY_CHAIN,
  DEFAULT_CONTRACTS,
  getContractsForChain,
  validateContractAddress,
  TOKEN_CONFIGS,
}