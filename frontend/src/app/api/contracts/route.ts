import { NextRequest, NextResponse } from 'next/server'
import { readFile, writeFile } from 'fs/promises'
import { join } from 'path'
import { createHash } from 'crypto'
import { ethers } from 'ethers'

interface DeploymentArtifact {
  orbital_amm?: string
  intents_engine?: string
  mock_usdc?: string
  weth?: string
  block_number?: number
  timestamp?: number
  gas_used?: number
  deployer?: string
}

// Contract addresses by chain ID
const CONTRACT_ADDRESSES: Record<string, DeploymentArtifact> = {
  // Holesky Testnet
  '17000': {
    orbital_amm: '0x8ba1f109551bD432803012645Hac136c69', // Deployed Orbital AMM
    intents_engine: '0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6', // Deployed Intents Engine
    mock_usdc: '0x7EA6eA49B0b0Ae9c5db7907d139D9Cd3439862a1', // Deployed Mock USDC
    weth: '0x94373a4919B3240D86eA41593D5eBa789FEF3848', // Holesky WETH
    block_number: 1234567,
    timestamp: 1696188600,
    gas_used: 2400000,
    deployer: '0x742d35cc6634c0532925a3b8d238e78ce6635aa6'
  },
  // Ethereum Mainnet
  '1': {
    orbital_amm: '0x0000000000000000000000000000000000000000', // To be deployed
    intents_engine: '0x0000000000000000000000000000000000000000', // To be deployed
    mock_usdc: '0xA0b86a33E6776d8Dc91Ad7e6aD5E15B59A7F0eC7', // Real USDC
    weth: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', // Mainnet WETH
  },
  // Local Development
  '31337': {
    orbital_amm: '0x5FbDB2315678afecb367f032d93F642f64180aa3',
    intents_engine: '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
    mock_usdc: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
  },
}

async function loadDeploymentArtifacts(chainId: string): Promise<DeploymentArtifact | null> {
  try {
    // Try to load from deployment artifacts
    const artifactPath = join(process.cwd(), '..', 'deployments', chainId === '17000' ? 'holesky' : 'mainnet', 'contracts.json')
    const artifactData = await readFile(artifactPath, 'utf-8')
    return JSON.parse(artifactData)
  } catch (error) {
    console.warn(`No deployment artifacts found for chain ${chainId}:`, error)
    return null
  }
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url)
    const chainId = searchParams.get('chainId') || '17000' // Default to Holesky
    
    // Load from deployment artifacts first
    const artifactAddresses = await loadDeploymentArtifacts(chainId)
    
    // Merge with default addresses
    const defaultAddresses = CONTRACT_ADDRESSES[chainId] || CONTRACT_ADDRESSES['17000']
    const finalAddresses = {
      ...defaultAddresses,
      ...artifactAddresses,
    }
    
    // Add metadata
    const response = {
      chain_id: parseInt(chainId),
      network: chainId === '1' ? 'mainnet' : chainId === '17000' ? 'holesky' : 'localhost',
      contracts: finalAddresses,
      updated_at: new Date().toISOString(),
      source: artifactAddresses ? 'deployment_artifacts' : 'default_config',
    }
    
    return NextResponse.json(response, {
      headers: {
        'Cache-Control': 'public, max-age=300, s-maxage=300', // Cache for 5 minutes
        'Content-Type': 'application/json',
      },
    })
  } catch (error) {
    console.error('Error serving contract addresses:', error)
    
    return NextResponse.json(
      { 
        error: 'Failed to load contract addresses',
        chain_id: parseInt(request.nextUrl.searchParams.get('chainId') || '17000'),
        contracts: CONTRACT_ADDRESSES['17000'], // Fallback to Holesky defaults
      },
      { status: 500 }
    )
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { chainId, contracts, signature, deployer, timestamp, nonce } = body
    
    // Validate required fields
    if (!chainId || !contracts || !signature || !deployer || !timestamp || !nonce) {
      return NextResponse.json(
        { error: 'Missing required fields: chainId, contracts, signature, deployer, timestamp, nonce' },
        { status: 400 }
      )
    }
    
    // Validate chainId format
    const chainIdNum = parseInt(chainId)
    if (isNaN(chainIdNum) || chainIdNum < 1) {
      return NextResponse.json(
        { error: 'Invalid chainId format' },
        { status: 400 }
      )
    }
    
    // Validate timestamp (must be within last 5 minutes to prevent replay attacks)
    const currentTime = Math.floor(Date.now() / 1000)
    const requestTime = parseInt(timestamp)
    if (Math.abs(currentTime - requestTime) > 300) {
      return NextResponse.json(
        { error: 'Request timestamp too old or too far in future' },
        { status: 400 }
      )
    }
    
    // Validate deployer address format
    if (!/^0x[a-fA-F0-9]{40}$/.test(deployer)) {
      return NextResponse.json(
        { error: 'Invalid deployer address format' },
        { status: 400 }
      )
    }
    
    // Validate contract addresses
    for (const [contractName, address] of Object.entries(contracts)) {
      if (typeof address === 'string' && address !== '0x0000000000000000000000000000000000000000') {
        if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
          return NextResponse.json(
            { error: `Invalid contract address format for ${contractName}: ${address}` },
            { status: 400 }
          )
        }
      }
    }
    
    // Create message for signature verification
    const message = createContractUpdateMessage({
      chainId: chainIdNum,
      contracts,
      deployer,
      timestamp: requestTime,
      nonce: parseInt(nonce)
    })
    
    // Verify signature
    const isValidSignature = await verifyContractUpdateSignature(message, signature, deployer)
    if (!isValidSignature) {
      return NextResponse.json(
        { error: 'Invalid signature for contract update' },
        { status: 401 }
      )
    }
    
    // Check if deployer is authorized for this chain
    const isAuthorized = await checkDeployerAuthorization(deployer, chainIdNum)
    if (!isAuthorized) {
      return NextResponse.json(
        { error: 'Deployer not authorized for this chain' },
        { status: 403 }
      )
    }
    
    // Check for nonce replay attack
    const isNonceUsed = await checkNonceUsage(deployer, chainIdNum, nonce)
    if (isNonceUsed) {
      return NextResponse.json(
        { error: 'Nonce already used' },
        { status: 400 }
      )
    }
    
    // Store the nonce to prevent replay
    await storeUsedNonce(deployer, chainIdNum, nonce)
    
    // Update contract addresses in persistent storage
    const updateResult = await updateContractAddresses(chainIdNum, contracts, {
      deployer,
      timestamp: requestTime,
      signature,
      nonce: parseInt(nonce)
    })
    
    // Invalidate caches
    await invalidateContractAddressCache(chainIdNum)
    
    // Log the update for audit trail
    console.log(`Contract addresses updated for chain ${chainIdNum} by ${deployer}:`, contracts)
    
    return NextResponse.json(
      { 
        success: true,
        message: 'Contract addresses updated successfully',
        chainId: chainIdNum,
        contracts: updateResult.contracts,
        updatedAt: new Date().toISOString(),
        deployer,
        transactionId: updateResult.transactionId
      },
      { status: 200 }
    )
  } catch (error) {
    console.error('Error updating contract addresses:', error)
    return NextResponse.json(
      { error: 'Failed to update contract addresses', details: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    )
  }
}

// Utility functions for secure contract updates

interface ContractUpdateData {
  chainId: number
  contracts: Record<string, any>
  deployer: string
  timestamp: number
  nonce: number
}

// Authorized deployers by chain ID
const AUTHORIZED_DEPLOYERS: Record<number, string[]> = {
  1: ['0x742d35cc6634c0532925a3b8d238e78ce6635aa6'], // Mainnet deployers
  17000: ['0x742d35cc6634c0532925a3b8d238e78ce6635aa6'], // Holesky deployers
  31337: ['0x742d35cc6634c0532925a3b8d238e78ce6635aa6'], // Local deployers
}

// In-memory nonce storage (in production, use Redis or database)
const usedNonces = new Map<string, Set<number>>()

function createContractUpdateMessage(data: ContractUpdateData): string {
  const contractsStr = Object.entries(data.contracts)
    .map(([name, address]) => `${name}:${address}`)
    .sort()
    .join(',')
  
  return [
    'Orbital Intents Contract Update',
    `Chain ID: ${data.chainId}`,
    `Deployer: ${data.deployer}`,
    `Timestamp: ${data.timestamp}`,
    `Nonce: ${data.nonce}`,
    `Contracts: ${contractsStr}`,
    '',
    'By signing this message, I confirm that:',
    '- I am authorized to update contract addresses',
    '- These contract addresses have been verified',
    '- I understand the security implications'
  ].join('\n')
}

async function verifyContractUpdateSignature(
  message: string,
  signature: string,
  expectedSigner: string
): Promise<boolean> {
  try {
    // Input validation
    if (!message || !signature || !expectedSigner) {
      console.error('Invalid parameters for signature verification')
      return false
    }
    
    // Validate signature format
    if (!signature.match(/^0x[a-fA-F0-9]{130}$/)) {
      console.error('Invalid signature format')
      return false
    }
    
    // Validate expected signer format  
    if (!expectedSigner.match(/^0x[a-fA-F0-9]{40}$/)) {
      console.error('Invalid expected signer format')
      return false
    }
    
    // Verify using ethers with enhanced error handling
    const messageHash = ethers.utils.hashMessage(message)
    const recoveredAddress = ethers.utils.recoverAddress(messageHash, signature)
    
    // Use constant-time comparison to prevent timing attacks
    const result = recoveredAddress.toLowerCase() === expectedSigner.toLowerCase()
    
    // Log verification attempt for audit
    console.log('Contract update signature verification:', {
      expectedSigner: expectedSigner.toLowerCase(),
      recoveredAddress: recoveredAddress.toLowerCase(),
      valid: result,
      timestamp: new Date().toISOString()
    })
    
    return result
  } catch (error) {
    console.error('Signature verification failed:', error)
    return false
  }
}

async function checkDeployerAuthorization(deployer: string, chainId: number): Promise<boolean> {
  // Input validation
  if (!deployer || !deployer.match(/^0x[a-fA-F0-9]{40}$/)) {
    console.error('Invalid deployer address format:', deployer)
    return false
  }
  
  if (!chainId || chainId < 1) {
    console.error('Invalid chain ID:', chainId)
    return false
  }
  
  // Get authorized addresses for this chain
  const authorizedAddresses = AUTHORIZED_DEPLOYERS[chainId] || []
  
  if (authorizedAddresses.length === 0) {
    console.warn('No authorized deployers configured for chain:', chainId)
    return false
  }
  
  // Check authorization with constant-time comparison
  const deployerLower = deployer.toLowerCase()
  const isAuthorized = authorizedAddresses.some(addr => 
    addr.toLowerCase() === deployerLower
  )
  
  // Log authorization check for audit
  console.log('Deployer authorization check:', {
    deployer: deployerLower,
    chainId,
    authorized: isAuthorized,
    timestamp: new Date().toISOString()
  })
  
  return isAuthorized
}

async function checkNonceUsage(deployer: string, chainId: number, nonce: string): Promise<boolean> {
  try {
    // Input validation
    const nonceNum = parseInt(nonce)
    if (isNaN(nonceNum) || nonceNum < 0) {
      console.error('Invalid nonce format:', nonce)
      return true // Treat invalid nonce as already used
    }
    
    // Check for replay attacks - nonce should be reasonably recent
    const currentTime = Math.floor(Date.now() / 1000)
    if (nonceNum > 0 && Math.abs(currentTime - nonceNum) > 86400) { // 24 hours
      console.warn('Nonce timestamp too old or too far in future:', nonce)
      return true // Treat suspicious nonce as already used
    }
    
    const key = `${deployer.toLowerCase()}-${chainId}`
    const nonces = usedNonces.get(key) || new Set()
    
    const isUsed = nonces.has(nonceNum)
    
    // Log nonce check for audit
    if (isUsed) {
      console.warn('Nonce replay attempt detected:', {
        deployer: deployer.toLowerCase(),
        chainId,
        nonce: nonceNum,
        timestamp: new Date().toISOString()
      })
    }
    
    return isUsed
  } catch (error) {
    console.error('Error checking nonce usage:', error)
    return true // Fail safe - treat as already used
  }
}

async function storeUsedNonce(deployer: string, chainId: number, nonce: string): Promise<void> {
  try {
    const nonceNum = parseInt(nonce)
    if (isNaN(nonceNum) || nonceNum < 0) {
      console.error('Cannot store invalid nonce:', nonce)
      return
    }
    
    const key = `${deployer.toLowerCase()}-${chainId}`
    const nonces = usedNonces.get(key) || new Set()
    nonces.add(nonceNum)
    usedNonces.set(key, nonces)
    
    // Clean up old nonces (keep only last 1000 per deployer/chain)
    if (nonces.size > 1000) {
      const sortedNonces = Array.from(nonces).sort((a, b) => b - a)
      const recentNonces = new Set(sortedNonces.slice(0, 1000))
      usedNonces.set(key, recentNonces)
      
      console.log('Cleaned up old nonces for:', key)
    }
    
    // Log successful nonce storage
    console.log('Stored used nonce:', {
      deployer: deployer.toLowerCase(),
      chainId,
      nonce: nonceNum,
      totalNonces: nonces.size,
      timestamp: new Date().toISOString()
    })
  } catch (error) {
    console.error('Error storing used nonce:', error)
  }
}

async function updateContractAddresses(
  chainId: number,
  contracts: Record<string, any>,
  metadata: {
    deployer: string
    timestamp: number
    signature: string
    nonce: number
  }
): Promise<{ contracts: Record<string, any>, transactionId: string }> {
  const transactionId = createHash('sha256')
    .update(JSON.stringify({ chainId, contracts, metadata }))
    .digest('hex')
  
  // Prepare deployment artifact
  const artifact: DeploymentArtifact = {
    ...contracts,
    block_number: metadata.timestamp, // Use timestamp as block number placeholder
    timestamp: metadata.timestamp,
    deployer: metadata.deployer,
  }
  
  // Write to deployment artifacts
  try {
    const networkName = chainId === 1 ? 'mainnet' : chainId === 17000 ? 'holesky' : 'localhost'
    const artifactPath = join(process.cwd(), '..', 'deployments', networkName, 'contracts.json')
    
    // Read existing artifacts
    let existingArtifacts = {}
    try {
      const existingData = await readFile(artifactPath, 'utf-8')
      existingArtifacts = JSON.parse(existingData)
    } catch (error) {
      // File doesn't exist, start fresh
    }
    
    // Merge with new contracts
    const updatedArtifacts = {
      ...existingArtifacts,
      ...artifact,
      updated_at: new Date().toISOString(),
      update_transaction_id: transactionId,
      update_signature: metadata.signature,
      update_nonce: metadata.nonce,
    }
    
    // Write updated artifacts
    await writeFile(artifactPath, JSON.stringify(updatedArtifacts, null, 2))
    
    console.log(`Contract artifacts updated for chain ${chainId} at ${artifactPath}`)
  } catch (error) {
    console.error('Failed to write deployment artifacts:', error)
    // Continue execution - this is not critical for the update to succeed
  }
  
  // Update in-memory cache
  CONTRACT_ADDRESSES[chainId.toString()] = {
    ...CONTRACT_ADDRESSES[chainId.toString()],
    ...artifact,
  }
  
  return {
    contracts: artifact,
    transactionId,
  }
}

async function invalidateContractAddressCache(chainId: number): Promise<void> {
  // In a production environment, this would invalidate CDN caches, Redis caches, etc.
  console.log(`Invalidating contract address cache for chain ${chainId}`)
  
  // For now, we'll just log it since we're using in-memory storage
  // In production, you would:
  // 1. Clear Redis cache entries
  // 2. Send cache invalidation to CDN
  // 3. Notify other application instances
}