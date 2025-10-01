import { configureChains, createConfig } from 'wagmi'
import { jsonRpcProvider } from 'wagmi/providers/jsonRpc'
import { MetaMaskConnector } from 'wagmi/connectors/metaMask'
import { WalletConnectConnector } from 'wagmi/connectors/walletConnect'
import { InjectedConnector } from 'wagmi/connectors/injected'
import { HOLESKY_CONFIG } from '@/constants'

// Define Holesky chain
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

// Configure chains and providers
const { chains, publicClient, webSocketPublicClient } = configureChains(
  [holeskyChain],
  [
    jsonRpcProvider({
      rpc: (chain) => ({
        http: chain.rpcUrls.default.http[0],
      }),
    }),
  ]
)

// Configure connectors
const connectors = [
  new MetaMaskConnector({
    chains,
    options: {
      shimDisconnect: true,
    },
  }),
  new WalletConnectConnector({
    chains,
    options: {
      projectId: process.env.NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID || 'orbital-amm',
      metadata: {
        name: 'Orbital AMM',
        description: 'Cross-Chain Intent Execution with Virtual Liquidity',
        url: typeof window !== 'undefined' ? window.location.origin : '',
        icons: ['/icons/logo.svg'],
      },
    },
  }),
  new InjectedConnector({
    chains,
    options: {
      name: 'Injected',
      shimDisconnect: true,
    },
  }),
]

// Create wagmi config
export const wagmiConfig = createConfig({
  autoConnect: true,
  connectors,
  publicClient,
  webSocketPublicClient,
})

export { chains }
export default wagmiConfig