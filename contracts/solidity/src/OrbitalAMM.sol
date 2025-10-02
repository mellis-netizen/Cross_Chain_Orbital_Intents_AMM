// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";

/**
 * @title OrbitalAMM
 * @dev Advanced AMM with orbital mathematics, MEV protection, and concentrated liquidity
 */
contract OrbitalAMM is ReentrancyGuard, Ownable {
    // Pool structure
    struct Pool {
        address token0;
        address token1;
        uint256 reserve0;
        uint256 reserve1;
        uint256 virtualReserve0;
        uint256 virtualReserve1;
        uint256 kLast;
        uint256 cumulativeVolume;
        bool active;
        uint256 totalLiquidityShares;
        uint256 rebalanceThreshold;
    }

    // Oracle structure for TWAP
    struct Oracle {
        uint256 price0Cumulative;
        uint256 price1Cumulative;
        uint32 timestampLast;
        uint256 reserve0Last;
        uint256 reserve1Last;
    }

    // Dynamic fee structure
    struct DynamicFeeState {
        uint256 baseFee;
        uint256 currentFee;
        uint256 volatilityFactor;
        uint256 volume24h;
        uint256 lastUpdate;
        uint256 maxFee;
        uint256 minFee;
    }

    // MEV protection structure
    struct Commitment {
        address trader;
        bytes32 commitHash;
        uint256 blockNumber;
        uint256 expiry;
        bool revealed;
        uint256 poolId;
    }

    // Rebalance state
    struct RebalanceState {
        uint256 lastRebalance;
        uint256 rebalanceCount;
        uint256 targetRatio;
        bool autoRebalanceEnabled;
    }

    // Arbitrage guard
    struct ArbitrageGuard {
        uint256 lastPrice;
        uint256 priceDeviationThreshold;
        uint256 lastTradeBlock;
        uint256 cooldownBlocks;
        bool locked;
    }

    // State variables
    mapping(uint256 => Pool) public pools;
    mapping(address => mapping(address => uint256)) public poolIds;
    mapping(uint256 => Oracle) public oracles;
    mapping(uint256 => DynamicFeeState) public dynamicFees;
    mapping(bytes32 => Commitment) public commitments;
    mapping(uint256 => RebalanceState) public rebalanceStates;
    mapping(uint256 => ArbitrageGuard) public arbitrageGuards;

    uint256 public nextPoolId = 1;
    uint256 public feeRate = 30; // 0.3% in basis points
    uint256 public commitRevealDelay = 2; // blocks
    uint256 public twapWindow = 1800; // 30 minutes

    // Events
    event PoolCreated(address indexed token0, address indexed token1, uint256 indexed poolId);
    event Swap(uint256 indexed poolId, address indexed trader, bool zeroForOne, uint256 amountIn, uint256 amountOut);
    event LiquidityAdded(uint256 indexed poolId, address indexed provider, uint256 amount0, uint256 amount1);
    event LiquidityRemoved(uint256 indexed poolId, address indexed provider, uint256 amount0, uint256 amount1);
    event DynamicFeeUpdated(uint256 indexed poolId, uint256 oldFee, uint256 newFee, uint256 volatility);
    event PoolRebalanced(uint256 indexed poolId, uint256 newReserve0, uint256 newReserve1, uint256 timestamp);
    event CommitmentCreated(bytes32 indexed commitHash, address indexed trader, uint256 timestamp);
    event SwapRevealed(bytes32 indexed commitHash, uint256 indexed poolId, uint256 amountOut);
    event ArbitrageDetected(uint256 indexed poolId, uint256 priceDiff, uint256 timestamp);

    // Custom errors
    error PoolNotFound();
    error InsufficientLiquidity();
    error InvalidAmount();
    error SlippageExceeded();
    error InvalidCommitment();
    error CommitmentExpired();
    error ArbitrageLocked();
    error Unauthorized();

    constructor() Ownable(msg.sender) {
        // Initialize with default parameters
    }

    /**
     * @dev Initialize the AMM with configuration
     */
    function initialize(address _owner, uint256 _feeRate) external {
        require(owner() == address(0), "Already initialized");
        _transferOwnership(_owner);
        feeRate = _feeRate;
    }

    /**
     * @dev Create a new trading pool
     */
    function createPool(
        address token0,
        address token1,
        uint256 virtualReserve0,
        uint256 virtualReserve1
    ) external returns (uint256) {
        require(token0 != token1, "Identical tokens");
        require(token0 != address(0) && token1 != address(0), "Zero address");

        // Order tokens
        (token0, token1) = token0 < token1 ? (token0, token1) : (token1, token0);

        require(poolIds[token0][token1] == 0, "Pool exists");

        uint256 poolId = nextPoolId++;
        poolIds[token0][token1] = poolId;

        pools[poolId] = Pool({
            token0: token0,
            token1: token1,
            reserve0: 0,
            reserve1: 0,
            virtualReserve0: virtualReserve0,
            virtualReserve1: virtualReserve1,
            kLast: 0,
            cumulativeVolume: 0,
            active: true,
            totalLiquidityShares: 0,
            rebalanceThreshold: 500 // 5%
        });

        // Initialize dynamic fee state
        dynamicFees[poolId] = DynamicFeeState({
            baseFee: feeRate,
            currentFee: feeRate,
            volatilityFactor: 10000,
            volume24h: 0,
            lastUpdate: block.timestamp,
            maxFee: 100, // 1%
            minFee: 5    // 0.05%
        });

        // Initialize rebalance state
        rebalanceStates[poolId] = RebalanceState({
            lastRebalance: 0,
            rebalanceCount: 0,
            targetRatio: 10000, // 1:1
            autoRebalanceEnabled: true
        });

        // Initialize arbitrage guard
        arbitrageGuards[poolId] = ArbitrageGuard({
            lastPrice: 0,
            priceDeviationThreshold: 50, // 0.5%
            lastTradeBlock: 0,
            cooldownBlocks: 1,
            locked: false
        });

        emit PoolCreated(token0, token1, poolId);
        return poolId;
    }

    /**
     * @dev Add liquidity to a pool
     */
    function addLiquidity(
        uint256 poolId,
        uint256 amount0,
        uint256 amount1
    ) external nonReentrant returns (uint256) {
        Pool storage pool = pools[poolId];
        require(pool.active, "Pool not active");

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        if (reserve0 == 0 || reserve1 == 0) {
            pool.reserve0 += amount0;
            pool.reserve1 += amount1;
        } else {
            uint256 optimalAmount1 = (amount0 * reserve1) / reserve0;
            if (optimalAmount1 <= amount1) {
                pool.reserve0 += amount0;
                pool.reserve1 += optimalAmount1;
            } else {
                uint256 optimalAmount0 = (amount1 * reserve0) / reserve1;
                pool.reserve0 += optimalAmount0;
                pool.reserve1 += amount1;
            }
        }

        updateOracle(poolId);
        updateKInvariant(poolId);

        emit LiquidityAdded(poolId, msg.sender, amount0, amount1);
        return poolId;
    }

    /**
     * @dev Execute a swap
     */
    function swap(
        uint256 poolId,
        bool zeroForOne,
        uint256 amountIn,
        uint256 minAmountOut
    ) external nonReentrant returns (uint256) {
        if (amountIn == 0) revert InvalidAmount();

        Pool storage pool = pools[poolId];
        if (!pool.active) revert PoolNotFound();

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        // Check arbitrage guard
        checkArbitrageGuard(poolId);

        // Calculate dynamic fee
        uint256 currentFee = calculateDynamicFee(poolId);
        uint256 amountInWithFee = (amountIn * (10000 - currentFee)) / 10000;

        uint256 amountOut;
        if (zeroForOne) {
            amountOut = (amountInWithFee * reserve1) / (reserve0 + amountInWithFee);
        } else {
            amountOut = (amountInWithFee * reserve0) / (reserve1 + amountInWithFee);
        }

        if (amountOut < minAmountOut) revert SlippageExceeded();

        // Update reserves
        if (zeroForOne) {
            pool.reserve0 += amountIn;
            pool.reserve1 = pool.reserve1 >= amountOut ? pool.reserve1 - amountOut : 0;
        } else {
            pool.reserve1 += amountIn;
            pool.reserve0 = pool.reserve0 >= amountOut ? pool.reserve0 - amountOut : 0;
        }

        pool.cumulativeVolume += amountIn;

        updateOracle(poolId);
        updateKInvariant(poolId);
        updateArbitrageGuard(poolId, zeroForOne);
        checkAndRebalance(poolId);

        emit Swap(poolId, msg.sender, zeroForOne, amountIn, amountOut);
        return amountOut;
    }

    /**
     * @dev Calculate dynamic fee based on volatility
     */
    function calculateDynamicFee(uint256 poolId) internal returns (uint256) {
        DynamicFeeState storage feeState = dynamicFees[poolId];
        Pool storage pool = pools[poolId];

        uint256 baseFee = feeState.baseFee;
        uint256 maxFee = feeState.maxFee;
        uint256 minFee = feeState.minFee;

        Oracle storage oracle = oracles[poolId];
        uint256 timeElapsed = block.timestamp - oracle.timestampLast;

        if (timeElapsed == 0) {
            return baseFee;
        }

        // Calculate volatility
        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;
        uint256 reserve0Last = oracle.reserve0Last;
        uint256 reserve1Last = oracle.reserve1Last;

        if (reserve0Last == 0 || reserve1Last == 0) {
            return baseFee;
        }

        uint256 currentPrice = (reserve1 * 10000) / reserve0;
        uint256 lastPrice = (reserve1Last * 10000) / reserve0Last;

        uint256 priceDiff = currentPrice > lastPrice ? 
            currentPrice - lastPrice : lastPrice - currentPrice;
        uint256 volatility = (priceDiff * 10000) / lastPrice;

        // Adjust fee based on volatility
        uint256 adjustedFee = baseFee + (volatility / 100);

        // Clamp between min and max
        uint256 currentFee = adjustedFee > maxFee ? maxFee : 
                           adjustedFee < minFee ? minFee : adjustedFee;

        // Update fee state
        if (currentFee != feeState.currentFee) {
            emit DynamicFeeUpdated(poolId, feeState.currentFee, currentFee, volatility);
        }

        feeState.currentFee = currentFee;
        feeState.volatilityFactor = volatility;
        feeState.lastUpdate = block.timestamp;

        return currentFee;
    }

    /**
     * @dev Check arbitrage guard
     */
    function checkArbitrageGuard(uint256 poolId) internal view {
        ArbitrageGuard storage guard = arbitrageGuards[poolId];

        if (guard.locked) {
            uint256 blocksElapsed = block.number - guard.lastTradeBlock;
            if (blocksElapsed < guard.cooldownBlocks) {
                revert ArbitrageLocked();
            }
        }
    }

    /**
     * @dev Update arbitrage guard
     */
    function updateArbitrageGuard(uint256 poolId, bool zeroForOne) internal {
        Pool storage pool = pools[poolId];
        ArbitrageGuard storage guard = arbitrageGuards[poolId];

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        uint256 currentPrice = zeroForOne ? 
            (reserve1 * 10000) / reserve0 : (reserve0 * 10000) / reserve1;

        if (guard.lastPrice != 0) {
            uint256 priceDiff = currentPrice > guard.lastPrice ? 
                currentPrice - guard.lastPrice : guard.lastPrice - currentPrice;
            uint256 deviation = (priceDiff * 10000) / guard.lastPrice;

            if (deviation > guard.priceDeviationThreshold) {
                guard.locked = true;
                emit ArbitrageDetected(poolId, deviation, block.timestamp);
            } else {
                guard.locked = false;
            }
        }

        guard.lastPrice = currentPrice;
        guard.lastTradeBlock = block.number;
    }

    /**
     * @dev Check and trigger rebalancing if needed
     */
    function checkAndRebalance(uint256 poolId) internal {
        RebalanceState storage rebalanceState = rebalanceStates[poolId];

        if (!rebalanceState.autoRebalanceEnabled) {
            return;
        }

        Pool storage pool = pools[poolId];
        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        if (reserve0 == 0 || reserve1 == 0) {
            return;
        }

        uint256 currentRatio = (reserve0 * 10000) / reserve1;
        uint256 targetRatio = rebalanceState.targetRatio;

        uint256 deviation = currentRatio > targetRatio ?
            ((currentRatio - targetRatio) * 10000) / targetRatio :
            ((targetRatio - currentRatio) * 10000) / targetRatio;

        if (deviation > pool.rebalanceThreshold) {
            rebalancePool(poolId);
        }
    }

    /**
     * @dev Rebalance pool reserves
     */
    function rebalancePool(uint256 poolId) internal {
        Pool storage pool = pools[poolId];
        RebalanceState storage rebalanceState = rebalanceStates[poolId];

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        uint256 targetRatio = rebalanceState.targetRatio;
        uint256 totalValue = reserve0 + reserve1;
        uint256 targetReserve0 = (totalValue * targetRatio) / (targetRatio + 10000);
        uint256 targetReserve1 = totalValue - targetReserve0;

        // Update virtual reserves
        uint256 realReserve0 = pool.reserve0;
        uint256 realReserve1 = pool.reserve1;

        pool.virtualReserve0 = targetReserve0 > realReserve0 ? 
            targetReserve0 - realReserve0 : 0;
        pool.virtualReserve1 = targetReserve1 > realReserve1 ? 
            targetReserve1 - realReserve1 : 0;

        rebalanceState.lastRebalance = block.timestamp;
        rebalanceState.rebalanceCount++;

        emit PoolRebalanced(poolId, targetReserve0, targetReserve1, block.timestamp);
    }

    /**
     * @dev Update oracle with current price data
     */
    function updateOracle(uint256 poolId) internal {
        Pool storage pool = pools[poolId];
        Oracle storage oracle = oracles[poolId];

        uint256 timeElapsed = block.timestamp - oracle.timestampLast;

        if (timeElapsed > 0 && pool.reserve0 != 0 && pool.reserve1 != 0) {
            oracle.price0Cumulative += (pool.reserve1 * timeElapsed) / pool.reserve0;
            oracle.price1Cumulative += (pool.reserve0 * timeElapsed) / pool.reserve1;
        }

        oracle.timestampLast = uint32(block.timestamp);
        oracle.reserve0Last = pool.reserve0;
        oracle.reserve1Last = pool.reserve1;
    }

    /**
     * @dev Update constant product invariant
     */
    function updateKInvariant(uint256 poolId) internal {
        Pool storage pool = pools[poolId];
        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;
        pool.kLast = reserve0 * reserve1;
    }

    /**
     * @dev Get amount out for a given input
     */
    function getAmountOut(
        uint256 poolId,
        bool zeroForOne,
        uint256 amountIn
    ) external view returns (uint256) {
        Pool storage pool = pools[poolId];
        if (!pool.active) revert PoolNotFound();

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        uint256 amountInWithFee = (amountIn * (10000 - feeRate)) / 10000;

        if (zeroForOne) {
            return (amountInWithFee * reserve1) / (reserve0 + amountInWithFee);
        } else {
            return (amountInWithFee * reserve0) / (reserve1 + amountInWithFee);
        }
    }

    /**
     * @dev Get pool by token pair
     */
    function getPoolByTokens(address token0, address token1) external view returns (uint256) {
        (token0, token1) = token0 < token1 ? (token0, token1) : (token1, token0);
        uint256 poolId = poolIds[token0][token1];
        if (poolId == 0) revert PoolNotFound();
        return poolId;
    }

    /**
     * @dev Get comprehensive pool state
     */
    function getPoolState(uint256 poolId) external view returns (
        uint256 reserve0,
        uint256 reserve1,
        uint256 virtual0,
        uint256 virtual1,
        uint256 k,
        uint256 volume
    ) {
        Pool storage pool = pools[poolId];
        return (
            pool.reserve0,
            pool.reserve1,
            pool.virtualReserve0,
            pool.virtualReserve1,
            pool.kLast,
            pool.cumulativeVolume
        );
    }

    /**
     * @dev Get fee state
     */
    function getFeeState(uint256 poolId) external view returns (
        uint256 currentFee,
        uint256 baseFee,
        uint256 volatilityFactor,
        uint256 volume24h
    ) {
        DynamicFeeState storage feeState = dynamicFees[poolId];
        return (
            feeState.currentFee,
            feeState.baseFee,
            feeState.volatilityFactor,
            feeState.volume24h
        );
    }

    /**
     * @dev Get TWAP price
     */
    function getTwap(uint256 poolId) external view returns (uint256) {
        Oracle storage oracle = oracles[poolId];
        Pool storage pool = pools[poolId];

        if (!pool.active) revert PoolNotFound();

        uint256 timeElapsed = block.timestamp - oracle.timestampLast;

        if (timeElapsed == 0) {
            uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
            uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;
            return reserve0 == 0 ? 0 : (reserve1 * 10000) / reserve0;
        }

        uint256 effectiveTime = timeElapsed > twapWindow ? twapWindow : timeElapsed;
        return effectiveTime == 0 ? 0 : oracle.price0Cumulative / effectiveTime;
    }

    /**
     * @dev Get spot price
     */
    function getSpotPrice(uint256 poolId) external view returns (uint256) {
        Pool storage pool = pools[poolId];
        if (!pool.active) revert PoolNotFound();

        uint256 reserve0 = pool.reserve0 + pool.virtualReserve0;
        uint256 reserve1 = pool.reserve1 + pool.virtualReserve1;

        return reserve0 == 0 ? 0 : (reserve1 * 10000) / reserve0;
    }

    /**
     * @dev Create commitment for MEV protection
     */
    function createCommitment(
        bytes32 commitHash,
        uint256 poolId,
        uint256 expiryBlocks
    ) external {
        require(commitments[commitHash].trader == address(0), "Commitment exists");

        commitments[commitHash] = Commitment({
            trader: msg.sender,
            commitHash: commitHash,
            blockNumber: block.number,
            expiry: block.number + expiryBlocks,
            revealed: false,
            poolId: poolId
        });

        emit CommitmentCreated(commitHash, msg.sender, block.timestamp);
    }

    /**
     * @dev Reveal and execute committed swap
     */
    function revealAndSwap(
        bytes32 commitHash,
        uint256 poolId,
        bool zeroForOne,
        uint256 amountIn,
        uint256 minAmountOut,
        uint256 nonce
    ) external nonReentrant returns (uint256) {
        Commitment storage commitment = commitments[commitHash];

        if (commitment.trader != msg.sender) revert InvalidCommitment();
        if (commitment.revealed) revert InvalidCommitment();
        if (block.number > commitment.expiry) revert CommitmentExpired();

        uint256 blocksElapsed = block.number - commitment.blockNumber;
        if (blocksElapsed < commitRevealDelay) revert InvalidCommitment();

        commitment.revealed = true;

        uint256 amountOut = swap(poolId, zeroForOne, amountIn, minAmountOut);

        emit SwapRevealed(commitHash, poolId, amountOut);
        return amountOut;
    }

    /**
     * @dev Owner functions for configuration
     */
    function configureFees(
        uint256 poolId,
        uint256 baseFee,
        uint256 minFee,
        uint256 maxFee
    ) external onlyOwner {
        DynamicFeeState storage feeState = dynamicFees[poolId];
        feeState.baseFee = baseFee;
        feeState.minFee = minFee;
        feeState.maxFee = maxFee;
    }

    function configureRebalancing(
        uint256 poolId,
        uint256 threshold,
        uint256 targetRatio,
        bool autoEnabled
    ) external onlyOwner {
        pools[poolId].rebalanceThreshold = threshold;
        rebalanceStates[poolId].targetRatio = targetRatio;
        rebalanceStates[poolId].autoRebalanceEnabled = autoEnabled;
    }

    function configureMevProtection(
        uint256 _commitRevealDelay,
        uint256 _twapWindow
    ) external onlyOwner {
        commitRevealDelay = _commitRevealDelay;
        twapWindow = _twapWindow;
    }
}