'use client'

import { AlertTriangle } from 'lucide-react'
import { formatPercentage } from '@/utils'

interface PriceImpactWarningProps {
  priceImpact: number
}

export function PriceImpactWarning({ priceImpact }: PriceImpactWarningProps) {
  const getSeverity = (impact: number) => {
    if (impact > 0.15) return 'severe' // >15%
    if (impact > 0.05) return 'high'   // >5%
    if (impact > 0.03) return 'medium' // >3%
    return 'low'
  }

  const severity = getSeverity(priceImpact)

  const severityConfig = {
    low: {
      bgColor: 'bg-yellow-50 dark:bg-yellow-900/20',
      borderColor: 'border-yellow-200 dark:border-yellow-800',
      textColor: 'text-yellow-800 dark:text-yellow-200',
      iconColor: 'text-yellow-600 dark:text-yellow-400',
    },
    medium: {
      bgColor: 'bg-orange-50 dark:bg-orange-900/20',
      borderColor: 'border-orange-200 dark:border-orange-800',
      textColor: 'text-orange-800 dark:text-orange-200',
      iconColor: 'text-orange-600 dark:text-orange-400',
    },
    high: {
      bgColor: 'bg-red-50 dark:bg-red-900/20',
      borderColor: 'border-red-200 dark:border-red-800',
      textColor: 'text-red-800 dark:text-red-200',
      iconColor: 'text-red-600 dark:text-red-400',
    },
    severe: {
      bgColor: 'bg-red-100 dark:bg-red-900/30',
      borderColor: 'border-red-300 dark:border-red-700',
      textColor: 'text-red-900 dark:text-red-100',
      iconColor: 'text-red-700 dark:text-red-300',
    },
  }

  const config = severityConfig[severity]

  const getMessage = (impact: number, severity: string) => {
    if (severity === 'severe') {
      return {
        title: 'Severe Price Impact',
        description: `This swap will significantly impact the pool price by ${formatPercentage(impact)}. Consider reducing your trade size.`,
      }
    }
    if (severity === 'high') {
      return {
        title: 'High Price Impact',
        description: `This swap will have a high price impact of ${formatPercentage(impact)}. You may receive fewer tokens than expected.`,
      }
    }
    if (severity === 'medium') {
      return {
        title: 'Moderate Price Impact',
        description: `This swap will impact the pool price by ${formatPercentage(impact)}.`,
      }
    }
    return {
      title: 'Low Price Impact',
      description: `This swap will have a minimal price impact of ${formatPercentage(impact)}.`,
    }
  }

  const { title, description } = getMessage(priceImpact, severity)

  if (priceImpact < 0.01) return null // Don't show for very low impact

  return (
    <div className={`p-3 rounded-lg border ${config.bgColor} ${config.borderColor}`}>
      <div className="flex items-start space-x-3">
        <AlertTriangle className={`h-5 w-5 mt-0.5 ${config.iconColor}`} />
        <div className="flex-1 min-w-0">
          <h4 className={`font-medium text-sm ${config.textColor}`}>
            {title}
          </h4>
          <p className={`text-sm mt-1 ${config.textColor} opacity-90`}>
            {description}
          </p>
          
          {severity === 'severe' && (
            <div className={`mt-2 p-2 rounded ${config.bgColor} ${config.borderColor} border`}>
              <p className={`text-xs ${config.textColor} font-medium`}>
                ⚠️ Consider these alternatives:
              </p>
              <ul className={`text-xs ${config.textColor} mt-1 space-y-1 opacity-90`}>
                <li>• Reduce your trade size</li>
                <li>• Split into multiple smaller trades</li>
                <li>• Try again when liquidity is higher</li>
              </ul>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default PriceImpactWarning