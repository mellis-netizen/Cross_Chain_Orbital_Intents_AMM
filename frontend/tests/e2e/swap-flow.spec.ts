import { test, expect } from '@playwright/test'

test.describe('Swap Flow E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the swap page
    await page.goto('/swap')
    
    // Mock MetaMask connection
    await page.addInitScript(() => {
      window.ethereum = {
        isMetaMask: true,
        request: async ({ method }: { method: string }) => {
          if (method === 'eth_requestAccounts') {
            return ['0x742d35Cc6634C0532925a3b8D9f8B0d3516b1111']
          }
          if (method === 'eth_chainId') {
            return '0x1' // Ethereum mainnet
          }
          if (method === 'eth_accounts') {
            return ['0x742d35Cc6634C0532925a3b8D9f8B0d3516b1111']
          }
          return null
        },
        on: () => {},
        removeListener: () => {},
      }
    })
  })

  test('should display swap interface correctly', async ({ page }) => {
    // Check that main swap components are visible
    await expect(page.locator('h1')).toContainText('Swap')
    await expect(page.locator('[data-testid="input-amount"]')).toBeVisible()
    await expect(page.locator('[data-testid="output-amount"]')).toBeVisible()
    await expect(page.locator('[data-testid="swap-button"]')).toBeVisible()
  })

  test('should connect wallet successfully', async ({ page }) => {
    // Click connect wallet button
    await page.click('[data-testid="connect-wallet"]')
    
    // Should show connected state
    await expect(page.locator('[data-testid="wallet-address"]')).toContainText('0x742d35...1111')
    await expect(page.locator('[data-testid="network-status"]')).toContainText('Ethereum')
  })

  test('should select tokens and calculate quotes', async ({ page }) => {
    // Connect wallet first
    await page.click('[data-testid="connect-wallet"]')
    
    // Select input token
    await page.click('[data-testid="input-token-selector"]')
    await page.click('[data-testid="token-USDC"]')
    
    // Select output token
    await page.click('[data-testid="output-token-selector"]')
    await page.click('[data-testid="token-DAI"]')
    
    // Enter amount
    await page.fill('[data-testid="input-amount"]', '100')
    
    // Wait for quote calculation
    await expect(page.locator('[data-testid="output-amount"]')).not.toBeEmpty()
    await expect(page.locator('[data-testid="exchange-rate"]')).toBeVisible()
  })

  test('should show price impact warning for large trades', async ({ page }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Set up tokens
    await page.click('[data-testid="input-token-selector"]')
    await page.click('[data-testid="token-USDC"]')
    await page.click('[data-testid="output-token-selector"]')
    await page.click('[data-testid="token-DAI"]')
    
    // Enter large amount
    await page.fill('[data-testid="input-amount"]', '1000000')
    
    // Should show price impact warning
    await expect(page.locator('[data-testid="price-impact-warning"]')).toBeVisible()
    await expect(page.locator('[data-testid="price-impact-warning"]')).toContainText('High Price Impact')
  })

  test('should handle insufficient balance', async ({ page }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Mock insufficient balance
    await page.addInitScript(() => {
      window.mockBalance = '50' // Less than what we'll try to swap
    })
    
    await page.click('[data-testid="input-token-selector"]')
    await page.click('[data-testid="token-USDC"]')
    
    // Enter amount greater than balance
    await page.fill('[data-testid="input-amount"]', '100')
    
    // Swap button should be disabled
    await expect(page.locator('[data-testid="swap-button"]')).toBeDisabled()
    await expect(page.locator('text=Insufficient Balance')).toBeVisible()
  })

  test('should open and configure slippage settings', async ({ page }) => {
    // Open settings modal
    await page.click('[data-testid="swap-settings"]')
    
    // Should show slippage settings
    await expect(page.locator('[data-testid="slippage-setting"]')).toBeVisible()
    await expect(page.locator('[data-testid="slippage-input"]')).toHaveValue('0.5')
    
    // Change slippage
    await page.fill('[data-testid="slippage-input"]', '1.0')
    await page.click('[data-testid="save-settings"]')
    
    // Settings should be saved
    await page.click('[data-testid="swap-settings"]')
    await expect(page.locator('[data-testid="slippage-input"]')).toHaveValue('1.0')
  })

  test('should flip token positions', async ({ page }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Select tokens
    await page.click('[data-testid="input-token-selector"]')
    await page.click('[data-testid="token-USDC"]')
    await page.click('[data-testid="output-token-selector"]')
    await page.click('[data-testid="token-DAI"]')
    
    // Enter amount
    await page.fill('[data-testid="input-amount"]', '100')
    
    // Wait for initial quote
    await page.waitForFunction(() => {
      const output = document.querySelector('[data-testid="output-amount"]') as HTMLInputElement
      return output && output.value && output.value !== '0'
    })
    
    const initialOutput = await page.inputValue('[data-testid="output-amount"]')
    
    // Flip tokens
    await page.click('[data-testid="flip-tokens"]')
    
    // Tokens should be flipped
    await expect(page.locator('[data-testid="input-token-symbol"]')).toContainText('DAI')
    await expect(page.locator('[data-testid="output-token-symbol"]')).toContainText('USDC')
    
    // Amount should move to output field
    await expect(page.locator('[data-testid="input-amount"]')).toHaveValue(initialOutput)
  })

  test('should execute swap transaction', async ({ page }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Set up swap
    await page.click('[data-testid="input-token-selector"]')
    await page.click('[data-testid="token-USDC"]')
    await page.click('[data-testid="output-token-selector"]')
    await page.click('[data-testid="token-DAI"]')
    await page.fill('[data-testid="input-amount"]', '100')
    
    // Wait for quote
    await page.waitForFunction(() => {
      const output = document.querySelector('[data-testid="output-amount"]') as HTMLInputElement
      return output && output.value && output.value !== '0'
    })
    
    // Mock successful transaction
    await page.addInitScript(() => {
      window.ethereum.request = async ({ method }: { method: string }) => {
        if (method === 'eth_sendTransaction') {
          return '0x123456789abcdef'
        }
        return null
      }
    })
    
    // Execute swap
    await page.click('[data-testid="swap-button"]')
    
    // Confirm transaction
    await page.click('[data-testid="confirm-swap"]')
    
    // Should show transaction submitted
    await expect(page.locator('text=Transaction Submitted')).toBeVisible()
    await expect(page.locator('[data-testid="transaction-hash"]')).toContainText('0x123456789abcdef')
  })

  test('should handle network switching', async ({ page }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Mock network switch
    await page.addInitScript(() => {
      window.ethereum.request = async ({ method }: { method: string }) => {
        if (method === 'wallet_switchEthereumChain') {
          return null
        }
        if (method === 'eth_chainId') {
          return '0x89' // Polygon
        }
        return null
      }
    })
    
    // Switch network
    await page.click('[data-testid="network-selector"]')
    await page.click('[data-testid="network-polygon"]')
    
    // Should show Polygon network
    await expect(page.locator('[data-testid="network-status"]')).toContainText('Polygon')
  })

  test('should display transaction history', async ({ page }) => {
    await page.goto('/swap?tab=history')
    await page.click('[data-testid="connect-wallet"]')
    
    // Should show transaction history tab
    await expect(page.locator('[data-testid="transaction-history"]')).toBeVisible()
    
    // Should show recent transactions (if any)
    const hasTransactions = await page.locator('[data-testid="transaction-item"]').count()
    if (hasTransactions > 0) {
      await expect(page.locator('[data-testid="transaction-item"]').first()).toBeVisible()
    } else {
      await expect(page.locator('text=No transactions found')).toBeVisible()
    }
  })

  test('should be responsive on mobile devices', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 })
    
    await page.goto('/swap')
    
    // Should still be functional on mobile
    await expect(page.locator('[data-testid="input-amount"]')).toBeVisible()
    await expect(page.locator('[data-testid="swap-button"]')).toBeVisible()
    
    // Mobile-specific elements should be visible
    await expect(page.locator('[data-testid="mobile-menu"]')).toBeVisible()
  })

  test('should handle offline mode gracefully', async ({ page, context }) => {
    await page.click('[data-testid="connect-wallet"]')
    
    // Go offline
    await context.setOffline(true)
    
    // Try to perform actions
    await page.click('[data-testid="input-token-selector"]')
    
    // Should show offline indicator
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible()
    
    // Go back online
    await context.setOffline(false)
    await expect(page.locator('[data-testid="offline-indicator"]')).not.toBeVisible()
  })
})