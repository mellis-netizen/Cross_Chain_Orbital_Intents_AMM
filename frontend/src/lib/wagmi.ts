import { configureChains, createConfig } from 'wagmi'
import { jsonRpcProvider } from 'wagmi/providers/jsonRpc'
import { MetaMaskConnector } from 'wagmi/connectors/metaMask'
import { WalletConnectConnector } from 'wagmi/connectors/walletConnect'
import { InjectedConnector } from 'wagmi/connectors/injected'
import { CoinbaseWalletConnector } from 'wagmi/connectors/coinbaseWallet'
import { HOLESKY_CONFIG } from '@/constants'

// Define supported chains
const holeskyChain = {
  id: HOLESKY_CONFIG.chainId,
  name: HOLESKY_CONFIG.name,
  network: 'holesky',
  nativeCurrency: HOLESKY_CONFIG.nativeCurrency,
  rpcUrls: {
    default: {
      http: [HOLESKY_CONFIG.rpcUrl],
    },
    public: {
      http: [HOLESKY_CONFIG.rpcUrl],
    },
  },
  blockExplorers: {
    default: {
      name: 'Holesky Etherscan',
      url: HOLESKY_CONFIG.blockExplorer,
    },
  },
  testnet: true,
} as const

// Ethereum Mainnet (for cross-chain functionality)
const ethereumChain = {
  id: 1,
  name: 'Ethereum',
  network: 'homestead',
  nativeCurrency: {
    decimals: 18,
    name: 'Ether',
    symbol: 'ETH',
  },
  rpcUrls: {
    default: {
      http: ['https://eth-mainnet.g.alchemy.com/v2/demo'],
    },
    public: {
      http: ['https://eth-mainnet.g.alchemy.com/v2/demo'],
    },
  },
  blockExplorers: {
    default: {
      name: 'Etherscan',
      url: 'https://etherscan.io',
    },
  },
  testnet: false,
} as const

// Polygon (for additional cross-chain support)
const polygonChain = {
  id: 137,
  name: 'Polygon',
  network: 'matic',
  nativeCurrency: {
    decimals: 18,
    name: 'MATIC',
    symbol: 'MATIC',
  },
  rpcUrls: {
    default: {
      http: ['https://polygon-rpc.com'],
    },
    public: {
      http: ['https://polygon-rpc.com'],
    },
  },
  blockExplorers: {
    default: {
      name: 'PolygonScan',
      url: 'https://polygonscan.com',
    },
  },
  testnet: false,
} as const

// Configure chains and providers
const { chains, publicClient, webSocketPublicClient } = configureChains(
  [holeskyChain, ethereumChain, polygonChain],
  [
    jsonRpcProvider({
      rpc: (chain) => ({
        http: chain.rpcUrls.default.http[0],
        webSocket: chain.rpcUrls.default.webSocket?.[0],
      }),
    }),
  ],
  {
    batch: {
      multicall: true,
    },
    pollingInterval: 4_000,
  }
)

// Configure connectors with enhanced wallet support
const connectors = [
  // MetaMask - Most popular wallet
  new MetaMaskConnector({
    chains,
    options: {
      shimDisconnect: true,
      UNSTABLE_shimOnConnectSelectAccount: true,
    },
  }),
  
  // WalletConnect - Mobile and desktop wallets
  new WalletConnectConnector({
    chains,
    options: {
      projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'orbital-amm-demo',
      metadata: {
        name: 'Orbital AMM',
        description: 'Cross-Chain Intent Execution with Virtual Liquidity',
        url: typeof window !== 'undefined' ? window.location.origin : 'https://orbital-amm.netlify.app',
        icons: ['/icons/logo.svg'],
      },
      qrModalOptions: {
        themeMode: 'dark',
        themeVariables: {
          '--wcm-z-index': '1000',
        },
      },
    },
  }),
  
  // Coinbase Wallet
  new CoinbaseWalletConnector({
    chains,
    options: {
      appName: 'Orbital AMM',
      appLogoUrl: '/icons/logo.svg',
      darkMode: true,
      headlessMode: false,
    },
  }),
  
  // Note: Ledger connector removed due to compatibility issues with current wagmi version
  
  // Generic Injected Connector (fallback for other wallets)
  new InjectedConnector({
    chains,
    options: {
      name: 'Other Wallet',
      shimDisconnect: true,
    },
  }),
]

// Create wagmi config with enhanced settings
export const wagmiConfig = createConfig({
  autoConnect: true,
  connectors,
  publicClient,
  webSocketPublicClient,
  persister: null, // Disable persistence for demo
  logger: {
    warn: process.env.NODE_ENV === 'development' ? console.warn : () => {},
  },
})

// Export supported connector types for UI
export const WALLET_CONNECTORS = {
  METAMASK: 'metaMask',
  WALLETCONNECT: 'walletConnect',
  COINBASE: 'coinbaseWallet',
  INJECTED: 'injected',
} as const

// Helper function to get connector display info
export const getConnectorInfo = (connectorId: string) => {
  switch (connectorId) {
    case WALLET_CONNECTORS.METAMASK:
      return {
        name: 'MetaMask',
        icon: '/icons/metamask.svg',
        description: 'Connect using MetaMask',
        downloadUrl: 'https://metamask.io/download/',
      }
    case WALLET_CONNECTORS.WALLETCONNECT:
      return {
        name: 'WalletConnect',
        icon: '/icons/walletconnect.svg',
        description: 'Connect using WalletConnect protocol',
        downloadUrl: 'https://walletconnect.com/',
      }
    case WALLET_CONNECTORS.COINBASE:
      return {
        name: 'Coinbase Wallet',
        icon: '/icons/coinbase.svg',
        description: 'Connect using Coinbase Wallet',
        downloadUrl: 'https://www.coinbase.com/wallet',
      }
    // Ledger case removed - not supported in current wagmi version
    default:
      return {
        name: 'Other Wallet',
        icon: '/icons/wallet.svg',
        description: 'Connect using injected wallet',
        downloadUrl: '',
      }
  }
}

// Network switching helpers
export const switchToHolesky = async () => {
  if (typeof window.ethereum !== 'undefined') {
    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: `0x${HOLESKY_CONFIG.chainId.toString(16)}` }],
      })
    } catch (switchError: any) {
      // This error code indicates that the chain has not been added to MetaMask
      if (switchError.code === 4902) {
        try {
          await window.ethereum.request({
            method: 'wallet_addEthereumChain',
            params: [
              {
                chainId: `0x${HOLESKY_CONFIG.chainId.toString(16)}`,
                chainName: HOLESKY_CONFIG.name,
                nativeCurrency: HOLESKY_CONFIG.nativeCurrency,
                rpcUrls: [HOLESKY_CONFIG.rpcUrl],
                blockExplorerUrls: [HOLESKY_CONFIG.blockExplorer],
              },
            ],
          })
        } catch (addError) {
          console.error('Failed to add Holesky network:', addError)
          throw addError
        }
      } else {
        console.error('Failed to switch to Holesky network:', switchError)
        throw switchError
      }
    }
  }
}

export { chains, holeskyChain, ethereumChain, polygonChain }
export default wagmiConfig