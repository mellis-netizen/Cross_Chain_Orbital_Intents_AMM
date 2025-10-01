/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  
  // Enable static export for Netlify
  output: 'export',
  trailingSlash: true,
  
  experimental: {
    optimizeCss: true,
  },
  
  // Disable image optimization for static export
  images: {
    unoptimized: true,
    domains: ['assets.coingecko.com'],
  },
  
  typescript: {
    ignoreBuildErrors: false,
  },
  
  eslint: {
    ignoreDuringBuilds: false,
  },
  
  // Environment variables
  env: {
    NEXT_PUBLIC_APP_NAME: 'Orbital AMM',
    NEXT_PUBLIC_APP_VERSION: '1.0.0',
    NEXT_PUBLIC_NETWORK: 'holesky',
    NEXT_PUBLIC_CHAIN_ID: '17000', // Holesky
    NEXT_PUBLIC_RPC_URL: 'https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/',
    NEXT_PUBLIC_NETWORK_NAME: 'Holesky Testnet',
  },
  
  // Webpack configuration for Web3 compatibility
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
        crypto: false,
        stream: false,
        url: false,
        zlib: false,
        http: false,
        https: false,
        assert: false,
        os: false,
        path: false,
      }
    }
    
    return config
  },
  
  // Compiler options
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
  },
}

module.exports = nextConfig