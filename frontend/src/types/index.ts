import { Address } from 'viem'

// Core blockchain types
export type ChainId = number
export type TokenAddress = Address
export type BigNumberish = string | number | bigint

// Intent types
export enum IntentStatus {
  CREATED = 'Created',
  MATCHED = 'Matched',
  EXECUTED = 'Executed',
  CANCELLED = 'Cancelled',
  FAILED = 'Failed',
}

export interface Intent {
  id: string
  user: Address
  sourceChainId: ChainId
  destChainId: ChainId
  sourceToken: TokenAddress
  destToken: TokenAddress
  sourceAmount: string
  minDestAmount: string
  deadline: number
  nonce: number
  dataHash: string
  status: IntentStatus
}

export interface IntentExecution {
  solver: Address
  matchedAt: number
  executedAt: number
  destAmount: string
  proofHash: string
  verified: boolean
}

// Solver types
export interface Solver {
  address: Address
  stake: string
  reputationScore: number
  successfulIntents: number
  failedIntents: number
  lastActive: number
  isRegistered: boolean
}

// Pool types
export interface Pool {
  id: string
  token0: TokenAddress
  token1: TokenAddress
  reserve0: string
  reserve1: string
  virtualReserve0: string
  virtualReserve1: string
  kLast: string
  cumulativeVolume: string
  active: boolean
  totalLiquidityShares: string
  rebalanceThreshold: string
}

export interface PoolState {
  reserve0: string
  reserve1: string
  virtual0: string
  virtual1: string
  k: string
  volume: string
}

export interface DynamicFeeState {
  currentFee: string
  baseFee: string
  volatilityFactor: string
  volume24h: string
}

export interface RebalanceState {
  lastRebalance: number
  rebalanceCount: number
  targetRatio: string
  autoRebalanceEnabled: boolean
}

export interface ArbitrageGuardState {
  lastPrice: string
  priceDeviationThreshold: string
  locked: boolean
}

// Token types
export interface Token {
  address: TokenAddress
  symbol: string
  name: string
  decimals: number
  logoURI?: string
  chainId: ChainId
}

export interface TokenBalance {
  token: Token
  balance: string
  formattedBalance: string
}

// Swap types
export interface SwapQuote {
  amountIn: string
  amountOut: string
  amountOutMin: string
  fee: string
  priceImpact: string
  route: Token[]
  gasEstimate: string
}

export interface SwapParams {
  poolId: string
  zeroForOne: boolean
  amountIn: string
  minAmountOut: string
}

// Transaction types
export enum TransactionStatus {
  PENDING = 'pending',
  CONFIRMED = 'confirmed',
  FAILED = 'failed',
}

export interface Transaction {
  hash: string
  type: 'swap' | 'intent' | 'liquidity'
  status: TransactionStatus
  timestamp: number
  from: Address
  to?: Address
  value?: string
  gasUsed?: string
  gasPrice?: string
  blockNumber?: number
}

// Price and chart types
export interface PriceData {
  timestamp: number
  price: number
  volume: number
}

export interface PoolAnalytics {
  tvl: string
  volume24h: string
  volume7d: string
  fees24h: string
  apy: string
  priceChange24h: string
}

// UI types
export interface NotificationData {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  message: string
  timestamp: number
}

export interface LoadingState {
  isLoading: boolean
  message?: string
}

export interface ErrorState {
  hasError: boolean
  message?: string
  code?: string
}

// Form types
export interface SwapFormData {
  fromToken: Token | null
  toToken: Token | null
  fromAmount: string
  slippage: number
}

export interface IntentFormData {
  sourceChain: ChainId
  destChain: ChainId
  sourceToken: Token | null
  destToken: Token | null
  amount: string
  slippage: number
  deadline: number
}

export interface LiquidityFormData {
  token0: Token | null
  token1: Token | null
  amount0: string
  amount1: string
}

// Contract interaction types
export interface ContractAddresses {
  intentsEngine: Address
  orbitalAMM: Address
  mockUSDC: Address
}

export interface NetworkConfig {
  chainId: ChainId
  name: string
  rpcUrl: string
  blockExplorer: string
  nativeCurrency: {
    name: string
    symbol: string
    decimals: number
  }
  contracts: ContractAddresses
}

// API types
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  timestamp: number
}

export interface PoolMetrics {
  totalValueLocked: string
  volume24h: string
  feesGenerated: string
  numberOfSwaps: number
  uniqueUsers: number
}

export interface SystemMetrics {
  totalIntents: number
  successfulIntents: number
  totalSolvers: number
  averageExecutionTime: number
  totalVolumeUSD: string
}

// Theme types
export type Theme = 'light' | 'dark'

export interface ThemeConfig {
  theme: Theme
  primaryColor: string
  accentColor: string
}

// Utility types
export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>
export type Nullable<T> = T | null
export type ArrayElement<T> = T extends (infer U)[] ? U : never