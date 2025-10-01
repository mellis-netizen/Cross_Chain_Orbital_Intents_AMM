import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { formatUnits, parseUnits } from 'viem'

/**
 * Utility function to merge Tailwind CSS classes
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * Format a number with appropriate decimal places
 */
export function formatNumber(
  value: number | string,
  options?: {
    decimals?: number
    notation?: 'standard' | 'compact'
    currency?: string
  }
): string {
  const num = typeof value === 'string' ? parseFloat(value) : value
  
  if (isNaN(num)) return '0'
  
  const { decimals = 2, notation = 'standard', currency } = options || {}
  
  const formatter = new Intl.NumberFormat('en-US', {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
    notation,
    style: currency ? 'currency' : 'decimal',
    currency: currency || 'USD',
  })
  
  return formatter.format(num)
}

/**
 * Format token amounts with appropriate decimal places
 */
export function formatTokenAmount(
  amount: string | bigint,
  decimals: number = 18,
  displayDecimals: number = 4
): string {
  try {
    const formatted = formatUnits(BigInt(amount), decimals)
    const num = parseFloat(formatted)
    
    if (num === 0) return '0'
    if (num < 0.0001) return '< 0.0001'
    
    return formatNumber(num, { decimals: displayDecimals })
  } catch {
    return '0'
  }
}

/**
 * Parse token amount to wei/smallest unit
 */
export function parseTokenAmount(
  amount: string,
  decimals: number = 18
): bigint {
  try {
    if (!amount || amount === '') return BigInt(0)
    return parseUnits(amount, decimals)
  } catch {
    return BigInt(0)
  }
}

/**
 * Format percentage with appropriate decimal places
 */
export function formatPercentage(
  value: number | string,
  decimals: number = 2
): string {
  const num = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(num)) return '0%'
  return `${formatNumber(num * 100, { decimals })}%`
}

/**
 * Format USD amount
 */
export function formatUSD(
  value: number | string,
  notation?: 'standard' | 'compact'
): string {
  const num = typeof value === 'string' ? parseFloat(value) : value
  if (isNaN(num)) return '$0'
  
  return formatNumber(num, { 
    decimals: num < 1 ? 4 : 2, 
    notation,
    currency: 'USD'
  })
}

/**
 * Truncate address for display
 */
export function truncateAddress(
  address: string,
  startLength: number = 6,
  endLength: number = 4
): string {
  if (!address) return ''
  if (address.length <= startLength + endLength) return address
  
  return `${address.slice(0, startLength)}...${address.slice(-endLength)}`
}

/**
 * Format time ago
 */
export function formatTimeAgo(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `${days}d ago`
  if (hours > 0) return `${hours}h ago`
  if (minutes > 0) return `${minutes}m ago`
  return `${seconds}s ago`
}

/**
 * Format duration
 */
export function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  
  if (minutes > 0) {
    return `${minutes}m ${remainingSeconds}s`
  }
  return `${remainingSeconds}s`
}

/**
 * Calculate price impact
 */
export function calculatePriceImpact(
  inputAmount: string,
  outputAmount: string,
  inputReserve: string,
  outputReserve: string
): number {
  try {
    const inputAmountBN = BigInt(inputAmount)
    const outputAmountBN = BigInt(outputAmount)
    const inputReserveBN = BigInt(inputReserve)
    const outputReserveBN = BigInt(outputReserve)
    
    // Current price: outputReserve / inputReserve
    const currentPrice = Number(outputReserveBN) / Number(inputReserveBN)
    
    // Execution price: outputAmount / inputAmount
    const executionPrice = Number(outputAmountBN) / Number(inputAmountBN)
    
    // Price impact = (currentPrice - executionPrice) / currentPrice
    const priceImpact = (currentPrice - executionPrice) / currentPrice
    
    return Math.abs(priceImpact)
  } catch {
    return 0
  }
}

/**
 * Calculate slippage amount
 */
export function calculateSlippageAmount(
  amount: string,
  slippageBps: number
): string {
  try {
    const amountBN = BigInt(amount)
    const slippageAmount = (amountBN * BigInt(slippageBps)) / BigInt(10000)
    return slippageAmount.toString()
  } catch {
    return '0'
  }
}

/**
 * Validate Ethereum address
 */
export function isValidAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address)
}

/**
 * Validate amount input
 */
export function isValidAmount(amount: string): boolean {
  if (!amount || amount === '') return false
  
  const num = parseFloat(amount)
  return !isNaN(num) && num > 0 && isFinite(num)
}

/**
 * Convert basis points to percentage
 */
export function bpsToPercentage(bps: number): number {
  return bps / 100
}

/**
 * Convert percentage to basis points
 */
export function percentageToBps(percentage: number): number {
  return percentage * 100
}

/**
 * Generate random hex string
 */
export function generateRandomHex(length: number = 32): string {
  const chars = '0123456789abcdef'
  let result = '0x'
  
  for (let i = 0; i < length; i++) {
    result += chars[Math.floor(Math.random() * chars.length)]
  }
  
  return result
}

/**
 * Debounce function
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout
  
  return (...args: Parameters<T>) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

/**
 * Throttle function
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean
  
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

/**
 * Copy text to clipboard
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()
    
    try {
      document.execCommand('copy')
      document.body.removeChild(textArea)
      return true
    } catch {
      document.body.removeChild(textArea)
      return false
    }
  }
}

/**
 * Get explorer URL for transaction
 */
export function getExplorerUrl(
  hash: string,
  type: 'tx' | 'address' | 'block' = 'tx',
  baseUrl: string = 'https://holesky.etherscan.io'
): string {
  const paths = {
    tx: 'tx',
    address: 'address',
    block: 'block',
  }
  
  return `${baseUrl}/${paths[type]}/${hash}`
}

/**
 * Sleep/delay function
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

/**
 * Retry function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  retries: number = 3,
  delay: number = 1000
): Promise<T> {
  try {
    return await fn()
  } catch (error) {
    if (retries <= 0) throw error
    
    await sleep(delay)
    return retry(fn, retries - 1, delay * 2)
  }
}

/**
 * Safe JSON parse
 */
export function safeJsonParse<T>(
  json: string,
  fallback: T
): T {
  try {
    return JSON.parse(json)
  } catch {
    return fallback
  }
}

/**
 * Local storage utilities
 */
export const storage = {
  get: <T>(key: string, fallback: T): T => {
    if (typeof window === 'undefined') return fallback
    
    try {
      const item = localStorage.getItem(key)
      return item ? JSON.parse(item) : fallback
    } catch {
      return fallback
    }
  },
  
  set: <T>(key: string, value: T): void => {
    if (typeof window === 'undefined') return
    
    try {
      localStorage.setItem(key, JSON.stringify(value))
    } catch {
      console.warn(`Failed to save to localStorage: ${key}`)
    }
  },
  
  remove: (key: string): void => {
    if (typeof window === 'undefined') return
    localStorage.removeItem(key)
  },
  
  clear: (): void => {
    if (typeof window === 'undefined') return
    localStorage.clear()
  },
}

export default {
  cn,
  formatNumber,
  formatTokenAmount,
  parseTokenAmount,
  formatPercentage,
  formatUSD,
  truncateAddress,
  formatTimeAgo,
  formatDuration,
  calculatePriceImpact,
  calculateSlippageAmount,
  isValidAddress,
  isValidAmount,
  bpsToPercentage,
  percentageToBps,
  generateRandomHex,
  debounce,
  throttle,
  copyToClipboard,
  getExplorerUrl,
  sleep,
  retry,
  safeJsonParse,
  storage,
}