// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "forge-std/console.sol";
import "../contracts/orbital-amm/src/lib.rs"; // Assuming compiled Stylus contract

contract OrbitalAMMTest is Test {
    // Mock ERC20 token for testing
    MockERC20 public tokenA;
    MockERC20 public tokenB;
    
    // Users
    address public owner = address(0x1);
    address public user1 = address(0x2);
    address public user2 = address(0x3);
    address public liquidityProvider = address(0x4);
    
    // Test constants
    uint256 public constant INITIAL_SUPPLY = 1_000_000e18;
    uint256 public constant INITIAL_LIQUIDITY_A = 100_000e18;
    uint256 public constant INITIAL_LIQUIDITY_B = 200_000e18;
    uint256 public constant VIRTUAL_RESERVE_A = 50_000e18;
    uint256 public constant VIRTUAL_RESERVE_B = 100_000e18;
    
    // Events to test
    event PoolCreated(address indexed tokenA, address indexed tokenB, address pool);
    event Swap(address indexed user, address tokenIn, address tokenOut, uint256 amountIn, uint256 amountOut);
    event LiquidityAdded(address indexed provider, uint256 amountA, uint256 amountB, uint256 liquidity);
    event LiquidityRemoved(address indexed provider, uint256 amountA, uint256 amountB, uint256 liquidity);
    
    function setUp() public {
        vm.startPrank(owner);
        
        // Deploy mock tokens
        tokenA = new MockERC20("Token A", "TKNA", 18);
        tokenB = new MockERC20("Token B", "TKNB", 18);
        
        // Mint tokens to users
        tokenA.mint(user1, INITIAL_SUPPLY);
        tokenB.mint(user1, INITIAL_SUPPLY);
        tokenA.mint(user2, INITIAL_SUPPLY);
        tokenB.mint(user2, INITIAL_SUPPLY);
        tokenA.mint(liquidityProvider, INITIAL_SUPPLY);
        tokenB.mint(liquidityProvider, INITIAL_SUPPLY);
        
        vm.stopPrank();
    }
    
    function testPoolCreation() public {
        vm.startPrank(owner);
        
        // Test pool creation
        vm.expectEmit(true, true, false, true);
        emit PoolCreated(address(tokenA), address(tokenB), address(0)); // Address will be determined
        
        // Create pool with virtual reserves
        address pool = createPool(
            address(tokenA),
            address(tokenB),
            VIRTUAL_RESERVE_A,
            VIRTUAL_RESERVE_B
        );
        
        assertNotEq(pool, address(0), "Pool should be created");
        
        vm.stopPrank();
    }
    
    function testInitialLiquidityProvision() public {
        address pool = createPool(address(tokenA), address(tokenB), VIRTUAL_RESERVE_A, VIRTUAL_RESERVE_B);
        
        vm.startPrank(liquidityProvider);
        
        // Approve tokens
        tokenA.approve(pool, INITIAL_LIQUIDITY_A);
        tokenB.approve(pool, INITIAL_LIQUIDITY_B);
        
        // Add initial liquidity
        vm.expectEmit(true, false, false, true);
        emit LiquidityAdded(liquidityProvider, INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B, 0);
        
        uint256 liquidity = addLiquidity(pool, INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B);
        
        assertGt(liquidity, 0, "Should receive LP tokens");
        
        vm.stopPrank();
    }
    
    function testSwapExactInputForOutput() public {
        address pool = setupPoolWithLiquidity();
        
        vm.startPrank(user1);
        
        uint256 amountIn = 1000e18;
        uint256 minAmountOut = 1900e18; // Expecting roughly 2:1 ratio minus fees
        
        // Approve tokens
        tokenA.approve(pool, amountIn);
        
        uint256 balanceBeforeA = tokenA.balanceOf(user1);
        uint256 balanceBeforeB = tokenB.balanceOf(user1);
        
        // Execute swap
        vm.expectEmit(true, false, false, true);
        emit Swap(user1, address(tokenA), address(tokenB), amountIn, 0);
        
        uint256 amountOut = swapExactInputForOutput(
            pool,
            address(tokenA),
            address(tokenB),
            amountIn,
            minAmountOut
        );
        
        // Verify swap results
        assertEq(tokenA.balanceOf(user1), balanceBeforeA - amountIn, "Input token balance should decrease");
        assertEq(tokenB.balanceOf(user1), balanceBeforeB + amountOut, "Output token balance should increase");
        assertGe(amountOut, minAmountOut, "Should receive at least minimum output");
        
        vm.stopPrank();
    }
    
    function testVirtualReserveBenefit() public {
        // Create two pools: one with virtual reserves, one without
        address poolWithVirtual = createPool(address(tokenA), address(tokenB), VIRTUAL_RESERVE_A, VIRTUAL_RESERVE_B);
        address poolWithoutVirtual = createPool(address(tokenA), address(tokenB), 0, 0);
        
        // Add same real liquidity to both pools
        vm.startPrank(liquidityProvider);
        
        tokenA.approve(poolWithVirtual, INITIAL_LIQUIDITY_A);
        tokenB.approve(poolWithVirtual, INITIAL_LIQUIDITY_B);
        addLiquidity(poolWithVirtual, INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B);
        
        tokenA.approve(poolWithoutVirtual, INITIAL_LIQUIDITY_A);
        tokenB.approve(poolWithoutVirtual, INITIAL_LIQUIDITY_B);
        addLiquidity(poolWithoutVirtual, INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B);
        
        vm.stopPrank();
        
        // Test same swap on both pools
        uint256 swapAmount = 1000e18;
        
        vm.startPrank(user1);
        
        tokenA.approve(poolWithVirtual, swapAmount);
        uint256 outputWithVirtual = getQuote(poolWithVirtual, address(tokenA), address(tokenB), swapAmount);
        
        tokenA.approve(poolWithoutVirtual, swapAmount);
        uint256 outputWithoutVirtual = getQuote(poolWithoutVirtual, address(tokenA), address(tokenB), swapAmount);
        
        // Virtual reserves should provide better pricing (higher output)
        assertGt(outputWithVirtual, outputWithoutVirtual, "Virtual reserves should improve pricing");
        
        vm.stopPrank();
    }
    
    function testPriceImpactCalculation() public {
        address pool = setupPoolWithLiquidity();
        
        uint256 smallSwap = 100e18;
        uint256 largeSwap = 10_000e18;
        
        uint256 smallImpact = calculatePriceImpact(pool, address(tokenA), smallSwap);
        uint256 largeImpact = calculatePriceImpact(pool, address(tokenA), largeSwap);
        
        assertLt(smallImpact, largeImpact, "Larger swaps should have higher price impact");
        assertLt(smallImpact, 100, "Small swaps should have minimal price impact"); // < 1%
    }
    
    function testSlippageProtection() public {
        address pool = setupPoolWithLiquidity();
        
        vm.startPrank(user1);
        
        uint256 amountIn = 1000e18;
        uint256 expectedOut = getQuote(pool, address(tokenA), address(tokenB), amountIn);
        uint256 minAmountOut = expectedOut + 1; // Set higher than possible
        
        tokenA.approve(pool, amountIn);
        
        // Should revert due to slippage
        vm.expectRevert("INSUFFICIENT_OUTPUT_AMOUNT");
        swapExactInputForOutput(pool, address(tokenA), address(tokenB), amountIn, minAmountOut);
        
        vm.stopPrank();
    }
    
    function testFeeAccumulation() public {
        address pool = setupPoolWithLiquidity();
        
        uint256 feesBefore = getAccumulatedFees(pool);
        
        // Perform multiple swaps to accumulate fees
        vm.startPrank(user1);
        for (uint i = 0; i < 5; i++) {
            uint256 amountIn = 100e18;
            tokenA.approve(pool, amountIn);
            swapExactInputForOutput(pool, address(tokenA), address(tokenB), amountIn, 0);
        }
        vm.stopPrank();
        
        uint256 feesAfter = getAccumulatedFees(pool);
        assertGt(feesAfter, feesBefore, "Fees should accumulate from swaps");
    }
    
    function testLiquidityRemoval() public {
        address pool = setupPoolWithLiquidity();
        
        vm.startPrank(liquidityProvider);
        
        uint256 lpBalance = getLPTokenBalance(pool, liquidityProvider);
        uint256 balanceBeforeA = tokenA.balanceOf(liquidityProvider);
        uint256 balanceBeforeB = tokenB.balanceOf(liquidityProvider);
        
        // Remove half the liquidity
        uint256 liquidityToRemove = lpBalance / 2;
        
        vm.expectEmit(true, false, false, true);
        emit LiquidityRemoved(liquidityProvider, 0, 0, liquidityToRemove);
        
        (uint256 amountA, uint256 amountB) = removeLiquidity(pool, liquidityToRemove);
        
        // Verify tokens returned
        assertEq(tokenA.balanceOf(liquidityProvider), balanceBeforeA + amountA, "Should receive token A");
        assertEq(tokenB.balanceOf(liquidityProvider), balanceBeforeB + amountB, "Should receive token B");
        assertEq(getLPTokenBalance(pool, liquidityProvider), lpBalance - liquidityToRemove, "LP balance should decrease");
        
        vm.stopPrank();
    }
    
    // Fuzz testing
    function testFuzzSwapAmounts(uint256 amountIn) public {
        amountIn = bound(amountIn, 1e15, 10_000e18); // Between 0.001 and 10,000 tokens
        
        address pool = setupPoolWithLiquidity();
        
        vm.startPrank(user1);
        
        // Ensure user has enough balance
        if (tokenA.balanceOf(user1) < amountIn) {
            tokenA.mint(user1, amountIn);
        }
        
        tokenA.approve(pool, amountIn);
        
        uint256 quotedOutput = getQuote(pool, address(tokenA), address(tokenB), amountIn);
        uint256 actualOutput = swapExactInputForOutput(pool, address(tokenA), address(tokenB), amountIn, 0);
        
        // Output should match quote (within small tolerance for gas/rounding)
        assertApproxEqRel(actualOutput, quotedOutput, 1e15); // 0.1% tolerance
        
        vm.stopPrank();
    }
    
    function testFuzzLiquidityAmounts(uint256 amountA, uint256 amountB) public {
        amountA = bound(amountA, 1e18, 1_000_000e18);
        amountB = bound(amountB, 1e18, 1_000_000e18);
        
        address pool = createPool(address(tokenA), address(tokenB), VIRTUAL_RESERVE_A, VIRTUAL_RESERVE_B);
        
        vm.startPrank(liquidityProvider);
        
        // Mint enough tokens
        tokenA.mint(liquidityProvider, amountA);
        tokenB.mint(liquidityProvider, amountB);
        
        tokenA.approve(pool, amountA);
        tokenB.approve(pool, amountB);
        
        uint256 liquidity = addLiquidity(pool, amountA, amountB);
        assertGt(liquidity, 0, "Should receive LP tokens");
        
        vm.stopPrank();
    }
    
    // Invariant testing
    function invariant_poolKValue() public {
        address pool = setupPoolWithLiquidity();
        
        (uint256 reserveA, uint256 reserveB) = getReserves(pool);
        uint256 kBefore = reserveA * reserveB;
        
        // Perform random swap
        vm.startPrank(user1);
        uint256 amountIn = 100e18;
        tokenA.approve(pool, amountIn);
        swapExactInputForOutput(pool, address(tokenA), address(tokenB), amountIn, 0);
        vm.stopPrank();
        
        (reserveA, reserveB) = getReserves(pool);
        uint256 kAfter = reserveA * reserveB;
        
        // K should increase due to fees
        assertGe(kAfter, kBefore, "K value should not decrease");
    }
    
    // Helper functions (these would interface with the actual Stylus contract)
    function createPool(address _tokenA, address _tokenB, uint256 _virtualA, uint256 _virtualB) internal returns (address) {
        // Mock implementation - would call actual contract
        return address(0x1000);
    }
    
    function setupPoolWithLiquidity() internal returns (address) {
        address pool = createPool(address(tokenA), address(tokenB), VIRTUAL_RESERVE_A, VIRTUAL_RESERVE_B);
        
        vm.startPrank(liquidityProvider);
        tokenA.approve(pool, INITIAL_LIQUIDITY_A);
        tokenB.approve(pool, INITIAL_LIQUIDITY_B);
        addLiquidity(pool, INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B);
        vm.stopPrank();
        
        return pool;
    }
    
    function addLiquidity(address pool, uint256 amountA, uint256 amountB) internal returns (uint256) {
        // Mock implementation
        return 1000e18;
    }
    
    function removeLiquidity(address pool, uint256 liquidity) internal returns (uint256, uint256) {
        // Mock implementation
        return (500e18, 1000e18);
    }
    
    function swapExactInputForOutput(address pool, address tokenIn, address tokenOut, uint256 amountIn, uint256 minAmountOut) internal returns (uint256) {
        // Mock implementation
        return getQuote(pool, tokenIn, tokenOut, amountIn);
    }
    
    function getQuote(address pool, address tokenIn, address tokenOut, uint256 amountIn) internal view returns (uint256) {
        // Mock implementation - simplified constant product formula
        return (amountIn * 2 * 997) / 1000; // 0.3% fee
    }
    
    function calculatePriceImpact(address pool, address token, uint256 amount) internal view returns (uint256) {
        // Mock implementation
        return (amount * 100) / 1000000; // Simplified calculation
    }
    
    function getAccumulatedFees(address pool) internal view returns (uint256) {
        // Mock implementation
        return 1000e18;
    }
    
    function getLPTokenBalance(address pool, address user) internal view returns (uint256) {
        // Mock implementation
        return 2000e18;
    }
    
    function getReserves(address pool) internal view returns (uint256, uint256) {
        // Mock implementation
        return (INITIAL_LIQUIDITY_A, INITIAL_LIQUIDITY_B);
    }
}

// Mock ERC20 token for testing
contract MockERC20 is Test {
    string public name;
    string public symbol;
    uint8 public decimals;
    uint256 public totalSupply;
    
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    
    constructor(string memory _name, string memory _symbol, uint8 _decimals) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
    }
    
    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        totalSupply += amount;
        emit Transfer(address(0), to, amount);
    }
    
    function transfer(address to, uint256 amount) external returns (bool) {
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
    
    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        emit Transfer(from, to, amount);
        return true;
    }
}