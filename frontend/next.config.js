/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  images: {
    domains: ['assets.coingecko.com'],
  },
  typescript: {
    ignoreBuildErrors: false,
  },
  eslint: {
    ignoreDuringBuilds: false,
  },
  env: {
    NEXT_PUBLIC_CHAIN_ID: '17000', // Holesky
    NEXT_PUBLIC_RPC_URL: 'https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/',
    NEXT_PUBLIC_NETWORK_NAME: 'Holesky Testnet',
  },
}

module.exports = nextConfig