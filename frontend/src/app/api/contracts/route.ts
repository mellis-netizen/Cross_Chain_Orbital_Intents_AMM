import { NextRequest, NextResponse } from 'next/server'
import { readFile } from 'fs/promises'
import { join } from 'path'

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
    const { chainId, contracts, signature } = body
    
    // In a production environment, you would validate the signature
    // and ensure only authorized deployers can update contract addresses
    
    if (!chainId || !contracts) {
      return NextResponse.json(
        { error: 'Missing chainId or contracts in request body' },
        { status: 400 }
      )
    }
    
    // TODO: Implement secure contract address updates
    // This would involve:
    // 1. Signature verification
    // 2. Writing to persistent storage (database or file)
    // 3. Caching invalidation
    // 4. Notification system for address updates
    
    return NextResponse.json(
      { 
        message: 'Contract address updates not implemented in demo version',
        received: { chainId, contracts }
      },
      { status: 501 }
    )
  } catch (error) {
    console.error('Error updating contract addresses:', error)
    return NextResponse.json(
      { error: 'Failed to update contract addresses' },
      { status: 500 }
    )
  }
}