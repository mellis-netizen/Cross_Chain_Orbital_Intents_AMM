'use client'

import { useState, useEffect } from 'react'
import { Button } from '@/components/ui/Button'
import { Input } from '@/components/ui/Input'
import { Modal } from '@/components/ui/Modal'
import { SLIPPAGE_OPTIONS, MIN_SLIPPAGE, MAX_SLIPPAGE } from '@/constants'

interface SwapSettingsProps {
  isOpen: boolean
  onClose: () => void
  currentSlippage: number
  onUpdate: (settings: { slippage: number }) => void
}

export function SwapSettings({
  isOpen,
  onClose,
  currentSlippage,
  onUpdate,
}: SwapSettingsProps) {
  const [slippage, setSlippage] = useState(currentSlippage)
  const [customSlippage, setCustomSlippage] = useState('')
  const [isCustom, setIsCustom] = useState(false)

  useEffect(() => {
    setSlippage(currentSlippage)
    const isCustomValue = !SLIPPAGE_OPTIONS.includes(currentSlippage)
    setIsCustom(isCustomValue)
    if (isCustomValue) {
      setCustomSlippage(currentSlippage.toString())
    }
  }, [currentSlippage])

  const handleSlippageSelect = (value: number) => {
    setSlippage(value)
    setIsCustom(false)
    setCustomSlippage('')
  }

  const handleCustomSlippageChange = (value: string) => {
    setCustomSlippage(value)
    const numValue = parseFloat(value)
    if (!isNaN(numValue) && numValue >= MIN_SLIPPAGE && numValue <= MAX_SLIPPAGE) {
      setSlippage(numValue)
      setIsCustom(true)
    }
  }

  const handleSave = () => {
    if (slippage >= MIN_SLIPPAGE && slippage <= MAX_SLIPPAGE) {
      onUpdate({ slippage })
    }
  }

  const isValidSlippage = slippage >= MIN_SLIPPAGE && slippage <= MAX_SLIPPAGE

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Transaction Settings"
      description="Adjust your trading preferences"
      size="sm"
    >
      <div className="space-y-6">
        {/* Slippage Tolerance */}
        <div className="space-y-3">
          <div>
            <h4 className="font-medium mb-1">Slippage Tolerance</h4>
            <p className="text-sm text-muted-foreground">
              Your transaction will revert if the price changes unfavorably by more than this percentage.
            </p>
          </div>

          {/* Preset Options */}
          <div className="flex space-x-2">
            {SLIPPAGE_OPTIONS.map((option) => (
              <Button
                key={option}
                variant={slippage === option && !isCustom ? 'default' : 'outline'}
                size="sm"
                onClick={() => handleSlippageSelect(option)}
                className="flex-1"
              >
                {option}%
              </Button>
            ))}
          </div>

          {/* Custom Input */}
          <div className="flex space-x-2">
            <div className="flex-1">
              <Input
                type="number"
                placeholder="Custom"
                value={customSlippage}
                onChange={(e) => handleCustomSlippageChange(e.target.value)}
                className={`text-center ${
                  isCustom && isValidSlippage ? 'border-primary' : ''
                }`}
                min={MIN_SLIPPAGE}
                max={MAX_SLIPPAGE}
                step="0.1"
              />
            </div>
            <div className="flex items-center px-2 text-sm text-muted-foreground">
              %
            </div>
          </div>

          {/* Validation Messages */}
          {isCustom && customSlippage && !isValidSlippage && (
            <div className="text-sm text-destructive">
              {slippage < MIN_SLIPPAGE && `Slippage must be at least ${MIN_SLIPPAGE}%`}
              {slippage > MAX_SLIPPAGE && `Slippage cannot exceed ${MAX_SLIPPAGE}%`}
            </div>
          )}

          {/* High Slippage Warning */}
          {slippage > 3 && (
            <div className="p-3 bg-warning-50 border border-warning-200 rounded-lg">
              <div className="text-sm text-warning-800">
                <strong>High slippage warning:</strong> You may receive significantly fewer tokens than expected.
              </div>
            </div>
          )}
        </div>

        {/* MEV Protection Info */}
        <div className="p-3 bg-muted/50 rounded-lg">
          <h5 className="font-medium text-sm mb-1">MEV Protection</h5>
          <p className="text-xs text-muted-foreground">
            Orbital AMM includes built-in MEV protection with commit-reveal schemes and dynamic fees.
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex space-x-3">
          <Button
            variant="outline"
            onClick={onClose}
            className="flex-1"
          >
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={!isValidSlippage}
            className="flex-1"
          >
            Save
          </Button>
        </div>
      </div>
    </Modal>
  )
}

export default SwapSettings