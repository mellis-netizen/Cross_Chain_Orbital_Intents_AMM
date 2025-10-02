import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SwapInterface } from '../swap/SwapInterface'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// Mock the Web3 hooks
jest.mock('../../hooks/useWeb3', () => ({
  useWeb3: () => ({
    account: '0x123...',
    isConnected: true,
    chainId: 1,
  }),
}))

jest.mock('../../hooks/useContracts', () => ({
  useContracts: () => ({
    orbitalAMM: {
      getQuote: jest.fn().mockResolvedValue('1000000'),
      swap: jest.fn().mockResolvedValue({ hash: '0xabc...' }),
    },
  }),
}))

// Mock token list
const mockTokens = [
  { address: '0x1', symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  { address: '0x2', symbol: 'USDT', name: 'Tether USD', decimals: 6 },
  { address: '0x3', symbol: 'DAI', name: 'Dai Stablecoin', decimals: 18 },
]

const TestWrapper = ({ children }: { children: React.ReactNode }) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  })
  
  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  )
}

describe('SwapInterface Component', () => {
  beforeEach(() => {
    // Reset all mocks before each test
    jest.clearAllMocks()
  })

  it('renders swap interface with input fields', () => {
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    expect(screen.getByText(/swap/i)).toBeInTheDocument()
    expect(screen.getByPlaceholderText(/amount/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /swap/i })).toBeInTheDocument()
  })

  it('handles token selection', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Open token selector for input token
    const inputTokenButton = screen.getByTestId('input-token-selector')
    await user.click(inputTokenButton)

    // Select USDC
    await user.click(screen.getByText('USDC'))
    
    expect(screen.getByDisplayValue('USDC')).toBeInTheDocument()
  })

  it('calculates quote when amount is entered', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Enter amount
    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '100')

    // Wait for quote calculation
    await waitFor(() => {
      expect(screen.getByText(/estimated output/i)).toBeInTheDocument()
    })
  })

  it('shows price impact warning for large trades', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Enter large amount
    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '1000000')

    await waitFor(() => {
      expect(screen.getByText(/high price impact/i)).toBeInTheDocument()
    })
  })

  it('disables swap button when insufficient balance', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} balance="50" />
      </TestWrapper>
    )

    // Enter amount greater than balance
    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '100')

    const swapButton = screen.getByRole('button', { name: /swap/i })
    expect(swapButton).toBeDisabled()
    expect(screen.getByText(/insufficient balance/i)).toBeInTheDocument()
  })

  it('handles swap execution', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} balance="1000" />
      </TestWrapper>
    )

    // Set up swap
    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '100')

    // Execute swap
    const swapButton = screen.getByRole('button', { name: /swap/i })
    await user.click(swapButton)

    // Confirm transaction
    await user.click(screen.getByRole('button', { name: /confirm/i }))

    await waitFor(() => {
      expect(screen.getByText(/transaction submitted/i)).toBeInTheDocument()
    })
  })

  it('shows slippage settings', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Open settings
    const settingsButton = screen.getByRole('button', { name: /settings/i })
    await user.click(settingsButton)

    expect(screen.getByText(/slippage tolerance/i)).toBeInTheDocument()
    expect(screen.getByDisplayValue('0.5')).toBeInTheDocument() // Default 0.5%
  })

  it('updates slippage tolerance', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Open settings
    await user.click(screen.getByRole('button', { name: /settings/i }))

    // Change slippage
    const slippageInput = screen.getByDisplayValue('0.5')
    await user.clear(slippageInput)
    await user.type(slippageInput, '1.0')

    expect(screen.getByDisplayValue('1.0')).toBeInTheDocument()
  })

  it('handles token swap direction', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    // Set initial tokens
    await user.click(screen.getByTestId('input-token-selector'))
    await user.click(screen.getByText('USDC'))

    await user.click(screen.getByTestId('output-token-selector'))
    await user.click(screen.getByText('DAI'))

    // Flip tokens
    const flipButton = screen.getByRole('button', { name: /flip/i })
    await user.click(flipButton)

    // Verify tokens are flipped
    expect(screen.getByDisplayValue('DAI')).toBeInTheDocument()
    expect(screen.getByDisplayValue('USDC')).toBeInTheDocument()
  })

  it('displays loading state during transaction', async () => {
    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} balance="1000" />
      </TestWrapper>
    )

    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '100')

    const swapButton = screen.getByRole('button', { name: /swap/i })
    await user.click(swapButton)

    // Should show loading state
    expect(screen.getByText(/swapping/i)).toBeInTheDocument()
    expect(swapButton).toBeDisabled()
  })

  it('handles network errors gracefully', async () => {
    // Mock network error
    jest.mocked(require('../../hooks/useContracts').useContracts).mockReturnValue({
      orbitalAMM: {
        getQuote: jest.fn().mockRejectedValue(new Error('Network error')),
        swap: jest.fn(),
      },
    })

    const user = userEvent.setup()
    
    render(
      <TestWrapper>
        <SwapInterface tokens={mockTokens} />
      </TestWrapper>
    )

    const amountInput = screen.getByPlaceholderText(/amount/i)
    await user.type(amountInput, '100')

    await waitFor(() => {
      expect(screen.getByText(/failed to get quote/i)).toBeInTheDocument()
    })
  })
})