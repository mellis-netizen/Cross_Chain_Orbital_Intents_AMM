'use client'

import { useState } from 'react'
import { ChevronDown, Search } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Modal } from '@/components/ui/Modal'
import { Token } from '@/types'
import { SUPPORTED_TOKENS } from '@/constants'
import { isValidAddress } from '@/utils'

interface TokenSelectorProps {
  selectedToken?: Token | null
  onTokenSelect: (token: Token) => void
  disabled?: boolean
}

export function TokenSelector({ selectedToken, onTokenSelect, disabled }: TokenSelectorProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  const filteredTokens = SUPPORTED_TOKENS.filter(token => {
    const query = searchQuery.toLowerCase()
    return (
      token.symbol.toLowerCase().includes(query) ||
      token.name.toLowerCase().includes(query) ||
      token.address.toLowerCase().includes(query)
    )
  })

  const handleTokenSelect = (token: Token) => {
    onTokenSelect(token)
    setIsOpen(false)
    setSearchQuery('')
  }

  return (
    <>
      <Button
        variant="outline"
        onClick={() => setIsOpen(true)}
        disabled={disabled}
        className="h-12 px-3 min-w-[120px] flex items-center space-x-2"
      >
        {selectedToken ? (
          <>
            {selectedToken.logoURI && (
              <img
                src={selectedToken.logoURI}
                alt={selectedToken.symbol}
                className="w-6 h-6 rounded-full"
                onError={(e) => {
                  const target = e.target as HTMLImageElement
                  target.style.display = 'none'
                }}
              />
            )}
            <span className="font-semibold">{selectedToken.symbol}</span>
          </>
        ) : (
          <span>Select Token</span>
        )}
        <ChevronDown className="h-4 w-4 opacity-50" />
      </Button>

      <Modal
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        title="Select Token"
        size="sm"
      >
        <div className="space-y-4">
          {/* Search */}
          <Input
            placeholder="Search by name, symbol, or address"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            leftIcon={<Search className="h-4 w-4" />}
          />

          {/* Token List */}
          <div className="max-h-80 overflow-y-auto space-y-1">
            {filteredTokens.length > 0 ? (
              filteredTokens.map((token) => (
                <TokenListItem
                  key={token.address}
                  token={token}
                  isSelected={selectedToken?.address === token.address}
                  onSelect={() => handleTokenSelect(token)}
                />
              ))
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                {searchQuery ? (
                  <>
                    <p>No tokens found</p>
                    {isValidAddress(searchQuery) && (
                      <p className="text-xs mt-1">
                        Custom tokens are not supported yet
                      </p>
                    )}
                  </>
                ) : (
                  <p>No tokens available</p>
                )}
              </div>
            )}
          </div>

          {/* Popular Tokens */}
          {!searchQuery && (
            <div className="border-t pt-4">
              <h4 className="text-sm font-medium mb-2 text-muted-foreground">Popular Tokens</h4>
              <div className="flex flex-wrap gap-2">
                {SUPPORTED_TOKENS.slice(0, 4).map((token) => (
                  <Button
                    key={token.address}
                    variant="ghost"
                    size="sm"
                    onClick={() => handleTokenSelect(token)}
                    className="flex items-center space-x-2"
                  >
                    {token.logoURI && (
                      <img
                        src={token.logoURI}
                        alt={token.symbol}
                        className="w-4 h-4 rounded-full"
                      />
                    )}
                    <span>{token.symbol}</span>
                  </Button>
                ))}
              </div>
            </div>
          )}
        </div>
      </Modal>
    </>
  )
}

interface TokenListItemProps {
  token: Token
  isSelected: boolean
  onSelect: () => void
}

function TokenListItem({ token, isSelected, onSelect }: TokenListItemProps) {
  return (
    <button
      onClick={onSelect}
      disabled={isSelected}
      className={`w-full flex items-center space-x-3 p-3 rounded-lg hover:bg-muted transition-colors text-left ${
        isSelected ? 'bg-muted cursor-not-allowed opacity-50' : ''
      }`}
    >
      <div className="relative">
        {token.logoURI ? (
          <img
            src={token.logoURI}
            alt={token.symbol}
            className="w-8 h-8 rounded-full"
            onError={(e) => {
              const target = e.target as HTMLImageElement
              target.style.display = 'none'
              target.nextElementSibling?.classList.remove('hidden')
            }}
          />
        ) : null}
        {/* Fallback */}
        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-orbital-400 to-orbital-600 flex items-center justify-center text-white text-sm font-semibold">
          {token.symbol.charAt(0)}
        </div>
      </div>
      
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between">
          <div className="font-semibold">{token.symbol}</div>
          <div className="text-sm text-muted-foreground">
            Balance: 0.00
          </div>
        </div>
        <div className="text-sm text-muted-foreground truncate">
          {token.name}
        </div>
      </div>
    </button>
  )
}

export default TokenSelector