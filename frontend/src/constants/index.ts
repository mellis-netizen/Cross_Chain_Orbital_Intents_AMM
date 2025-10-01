import { Token, NetworkConfig, ContractAddresses } from '@/types'

// Network Configuration
export const HOLESKY_CHAIN_ID = 17000

export const HOLESKY_CONFIG: NetworkConfig = {
  chainId: HOLESKY_CHAIN_ID,
  name: 'Holesky Testnet',
  rpcUrl: 'https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/',
  blockExplorer: 'https://holesky.etherscan.io',
  nativeCurrency: {
    name: 'Ethereum',
    symbol: 'ETH',
    decimals: 18,
  },
  contracts: {
    // These will be populated from deployment artifacts
    intentsEngine: '0x0000000000000000000000000000000000000000',
    orbitalAMM: '0x0000000000000000000000000000000000000000',
    mockUSDC: '0x0000000000000000000000000000000000000000',
  } as ContractAddresses,
}

// Token Definitions
export const ETH_TOKEN: Token = {
  address: '0x0000000000000000000000000000000000000000',
  symbol: 'ETH',
  name: 'Ethereum',
  decimals: 18,
  chainId: HOLESKY_CHAIN_ID,
  logoURI: '/icons/eth.svg',
}

export const USDC_TOKEN: Token = {
  address: '0x0000000000000000000000000000000000000000', // Will be updated from deployment
  symbol: 'USDC',
  name: 'USD Coin',
  decimals: 6,
  chainId: HOLESKY_CHAIN_ID,
  logoURI: '/icons/usdc.svg',
}

export const SUPPORTED_TOKENS: Token[] = [ETH_TOKEN, USDC_TOKEN]

// Trading Configuration
export const DEFAULT_SLIPPAGE = 0.5 // 0.5%
export const MAX_SLIPPAGE = 5 // 5%
export const MIN_SLIPPAGE = 0.1 // 0.1%

export const SLIPPAGE_OPTIONS = [0.1, 0.5, 1.0, 2.0]

export const DEFAULT_DEADLINE = 20 // 20 minutes
export const MAX_DEADLINE = 60 // 60 minutes
export const MIN_DEADLINE = 1 // 1 minute

// Fee Configuration
export const BASE_FEE_RATE = 30 // 0.3% in basis points
export const MEV_PROTECTION_DELAY = 2 // blocks
export const TWAP_WINDOW = 1800 // 30 minutes in seconds

// UI Configuration
export const REFRESH_INTERVAL = 10000 // 10 seconds
export const CHART_UPDATE_INTERVAL = 30000 // 30 seconds
export const TRANSACTION_TIMEOUT = 300000 // 5 minutes

// Animation Configuration
export const ANIMATION_DURATION = {
  fast: 150,
  normal: 300,
  slow: 500,
}

// Toast Configuration
export const TOAST_DURATION = {
  success: 4000,
  error: 6000,
  info: 3000,
  warning: 5000,
}

// Validation Constants
export const MIN_TRADE_AMOUNT = '0.001' // ETH
export const MAX_TRADE_AMOUNT = '100' // ETH
export const MIN_INTENT_DEADLINE = 60 // 1 minute in seconds
export const MAX_INTENT_DEADLINE = 86400 // 24 hours in seconds

// Pool Configuration
export const POOL_FEE_TIERS = [500, 3000, 10000] // 0.05%, 0.3%, 1%
export const REBALANCE_THRESHOLD = 500 // 5% in basis points
export const ARBITRAGE_THRESHOLD = 50 // 0.5% in basis points

// Chart Configuration
export const CHART_COLORS = {
  primary: '#0ea5e9',
  secondary: '#22c55e',
  accent: '#f59e0b',
  danger: '#ef4444',
  success: '#22c55e',
  warning: '#f59e0b',
  info: '#0ea5e9',
}

export const CHART_TIME_RANGES = [
  { label: '1H', value: 3600 },
  { label: '1D', value: 86400 },
  { label: '7D', value: 604800 },
  { label: '30D', value: 2592000 },
]

// Error Messages
export const ERROR_MESSAGES = {
  WALLET_NOT_CONNECTED: 'Please connect your wallet',
  INSUFFICIENT_BALANCE: 'Insufficient balance',
  INVALID_AMOUNT: 'Invalid amount',
  SLIPPAGE_TOO_HIGH: 'Slippage tolerance too high',
  DEADLINE_EXPIRED: 'Transaction deadline expired',
  NETWORK_ERROR: 'Network error occurred',
  CONTRACT_ERROR: 'Contract interaction failed',
  USER_REJECTED: 'Transaction rejected by user',
  POOL_NOT_FOUND: 'Pool not found',
  INTENT_EXPIRED: 'Intent has expired',
  SOLVER_NOT_FOUND: 'No solver available',
}

// Success Messages
export const SUCCESS_MESSAGES = {
  SWAP_SUCCESSFUL: 'Swap completed successfully',
  INTENT_CREATED: 'Intent created successfully',
  INTENT_EXECUTED: 'Intent executed successfully',
  LIQUIDITY_ADDED: 'Liquidity added successfully',
  TRANSACTION_CONFIRMED: 'Transaction confirmed',
  WALLET_CONNECTED: 'Wallet connected successfully',
}

// Local Storage Keys
export const STORAGE_KEYS = {
  THEME: 'orbital-amm-theme',
  SLIPPAGE: 'orbital-amm-slippage',
  DEADLINE: 'orbital-amm-deadline',
  RECENT_TOKENS: 'orbital-amm-recent-tokens',
  TRANSACTION_HISTORY: 'orbital-amm-transactions',
  USER_PREFERENCES: 'orbital-amm-preferences',
}

// API Endpoints (if backend is implemented)
export const API_ENDPOINTS = {
  POOLS: '/api/pools',
  TOKENS: '/api/tokens',
  QUOTES: '/api/quotes',
  TRANSACTIONS: '/api/transactions',
  ANALYTICS: '/api/analytics',
  PRICES: '/api/prices',
}

// Contract Events
export const CONTRACT_EVENTS = {
  ORBITAL_AMM: {
    SWAP: 'Swap',
    POOL_CREATED: 'PoolCreated',
    LIQUIDITY_ADDED: 'LiquidityAdded',
    ORACLE_UPDATE: 'OracleUpdate',
    DYNAMIC_FEE_UPDATED: 'DynamicFeeUpdated',
    ARBITRAGE_DETECTED: 'ArbitrageDetected',
  },
  INTENTS_ENGINE: {
    INTENT_CREATED: 'IntentCreated',
    INTENT_MATCHED: 'IntentMatched',
    INTENT_EXECUTED: 'IntentExecuted',
    INTENT_CANCELLED: 'IntentCancelled',
    SOLVER_REGISTERED: 'SolverRegistered',
  },
}

// Social Links
export const SOCIAL_LINKS = {
  TWITTER: 'https://twitter.com/rust-intents',
  DISCORD: 'https://discord.gg/rust-intents',
  GITHUB: 'https://github.com/rust-intents/rust-intents',
  DOCS: 'https://docs.rust-intents.com',
}

// Feature Flags
export const FEATURES = {
  DARK_MODE: true,
  ANALYTICS: true,
  CHARTS: true,
  NOTIFICATIONS: true,
  TRANSACTION_HISTORY: true,
  INTENT_CREATION: true,
  LIQUIDITY_PROVISION: false, // Coming soon
  GOVERNANCE: false, // Coming soon
  MOBILE_APP: false, // Coming soon
}

// Environment Configuration
export const IS_PRODUCTION = process.env.NODE_ENV === 'production'
export const IS_DEVELOPMENT = process.env.NODE_ENV === 'development'

// App Metadata
export const APP_CONFIG = {
  name: 'Orbital AMM',
  description: 'Cross-Chain Intent Execution with Virtual Liquidity',
  version: '1.0.0',
  author: 'Rust Intents Team',
  keywords: ['defi', 'amm', 'cross-chain', 'intents', 'web3'],
}

export default {
  HOLESKY_CONFIG,
  SUPPORTED_TOKENS,
  DEFAULT_SLIPPAGE,
  CHART_COLORS,
  ERROR_MESSAGES,
  SUCCESS_MESSAGES,
  APP_CONFIG,
}