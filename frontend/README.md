# Orbital AMM Frontend

A production-grade Next.js frontend for the Cross-Chain Orbital AMM system. Built with TypeScript, Tailwind CSS, and modern Web3 integration.

## 🚀 Features

- **Modern Web3 Integration**: Seamless wallet connection with MetaMask, WalletConnect, and other popular wallets
- **Real-time Trading Interface**: Responsive swap interface with live quotes and price impact warnings
- **Cross-Chain Intent Management**: Create and track cross-chain trading intentions
- **MEV Protection**: Built-in protection against front-running and sandwich attacks
- **Dynamic Fee Visualization**: Real-time fee adjustments based on market conditions
- **Mobile-First Design**: Fully responsive interface optimized for all devices
- **Dark Mode Support**: Beautiful light and dark themes
- **Accessibility**: WCAG 2.1 AA compliant with keyboard navigation and screen reader support

## 🛠 Tech Stack

- **Framework**: Next.js 14 with App Router
- **Language**: TypeScript
- **Styling**: Tailwind CSS with custom design system
- **Web3**: Wagmi + Viem for Ethereum interactions
- **State Management**: React Query for server state
- **UI Components**: Custom component library with Radix UI primitives
- **Animations**: Framer Motion
- **Charts**: Recharts for analytics visualization
- **Forms**: React Hook Form with Zod validation
- **Notifications**: React Hot Toast

## 📦 Installation

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

## 🔧 Environment Setup

Create a `.env.local` file in the frontend directory:

```bash
# Network Configuration (already set in next.config.js)
NEXT_PUBLIC_CHAIN_ID=17000
NEXT_PUBLIC_RPC_URL=https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/
NEXT_PUBLIC_NETWORK_NAME="Holesky Testnet"

# Optional: WalletConnect Project ID
NEXT_PUBLIC_WALLETCONNECT_PROJECT_ID=your_project_id_here

# Optional: Analytics
NEXT_PUBLIC_ANALYTICS_ID=your_analytics_id
```

## 🏗 Project Structure

```
src/
├── app/                    # Next.js App Router pages
│   ├── globals.css        # Global styles and theme
│   ├── layout.tsx         # Root layout component
│   ├── page.tsx           # Home page (redirects to swap)
│   ├── providers.tsx      # App-wide providers
│   ├── swap/             # Swap interface page
│   └── intents/          # Intents management page
├── components/            # Reusable UI components
│   ├── ui/               # Base UI components (Button, Input, etc.)
│   ├── layout/           # Layout components (Header, Footer)
│   ├── swap/             # Swap-specific components
│   └── wallet/           # Wallet connection components
├── hooks/                 # Custom React hooks
│   ├── useWeb3.ts        # Web3 connection hooks
│   └── useContracts.ts   # Smart contract interaction hooks
├── lib/                   # Core utilities and configurations
│   ├── wagmi.ts          # Wagmi configuration
│   └── contracts.ts      # Contract ABIs and addresses
├── types/                 # TypeScript type definitions
├── constants/             # App constants and configuration
└── utils/                 # Utility functions
```

## 🎨 Design System

The frontend uses a custom design system built on Tailwind CSS:

### Colors
- **Primary**: Orbital blue theme (`orbital-*`)
- **Success**: Green for positive states
- **Warning**: Orange for caution states
- **Danger**: Red for error states
- **Muted**: Gray for secondary content

### Components
- **Buttons**: Multiple variants (default, outline, ghost, orbital)
- **Cards**: Glass morphism effects with subtle borders
- **Modals**: Accessible overlays with animation
- **Forms**: Validated inputs with real-time feedback
- **Badges**: Status indicators with semantic colors

### Typography
- **Font**: Inter for excellent readability
- **Scale**: Modular scale for consistent sizing
- **Weights**: Strategic use of font weights for hierarchy

## 🔌 Web3 Integration

### Wallet Support
- MetaMask
- WalletConnect
- Injected wallets
- Future: Coinbase Wallet, Rainbow, etc.

### Network Configuration
- **Primary**: Holesky Testnet (Chain ID: 17000)
- **RPC**: QuickNode endpoint with failover
- **Auto-switching**: Prompts users to switch to correct network

### Contract Integration
- **Real-time data**: Live pool states and pricing
- **Transaction handling**: Comprehensive error handling and retry logic
- **Event monitoring**: Real-time updates from blockchain events

## 🧪 Testing

```bash
# Run type checking
npm run type-check

# Run linting
npm run lint

# Run build verification
npm run build
```

## 🚀 Deployment

### Vercel (Recommended)
```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel

# Set environment variables in Vercel dashboard
```

### Docker
```bash
# Build image
docker build -t orbital-amm-frontend .

# Run container
docker run -p 3000:3000 orbital-amm-frontend
```

### Static Export
```bash
# Add to next.config.js
output: 'export'

# Build static files
npm run build

# Deploy dist folder to any static host
```

## 📱 Mobile Support

- **Responsive Design**: Mobile-first approach with breakpoints
- **Touch Interactions**: Optimized for mobile trading
- **Wallet Integration**: Deep linking to mobile wallets
- **Offline Support**: Service worker for basic caching
- **PWA Ready**: Can be installed as mobile app

## ♿ Accessibility

- **Keyboard Navigation**: Full keyboard support
- **Screen Readers**: Proper ARIA labels and semantic HTML
- **Focus Management**: Logical focus flow
- **Color Contrast**: WCAG AA compliant contrast ratios
- **Reduced Motion**: Respects prefers-reduced-motion

## 🔒 Security

- **CSP Headers**: Content Security Policy protection
- **XSS Prevention**: Sanitized inputs and outputs
- **Wallet Security**: Never stores private keys
- **HTTPS Only**: Enforced secure connections
- **Dependencies**: Regular security audits

## 📊 Performance

- **Lighthouse Score**: 95+ on all metrics
- **Bundle Size**: Optimized with tree shaking
- **Image Optimization**: Next.js automatic optimization
- **Caching**: Strategic caching for API calls
- **Code Splitting**: Route-based splitting

## 🔮 Future Enhancements

- **Multi-language Support**: i18n implementation
- **Advanced Charts**: TradingView integration
- **Portfolio Tracking**: User portfolio management
- **Governance**: DAO voting interface
- **Analytics Dashboard**: Advanced trading analytics
- **Mobile App**: React Native version

## 🐛 Troubleshooting

### Common Issues

1. **Wallet Connection Fails**
   ```bash
   # Clear browser cache and storage
   # Check network configuration
   # Verify RPC endpoint is accessible
   ```

2. **Transaction Reverts**
   ```bash
   # Check gas settings
   # Verify slippage tolerance
   # Ensure sufficient balance
   ```

3. **Build Errors**
   ```bash
   # Clear node_modules and reinstall
   rm -rf node_modules package-lock.json
   npm install
   ```

## 📞 Support

- **Documentation**: Check the main project README
- **Issues**: Report bugs on GitHub
- **Discord**: Join the community discussion
- **Email**: Contact the development team

## 📄 License

MIT License - see LICENSE file for details.

---

Built with ❤️ by the Rust Intents Team