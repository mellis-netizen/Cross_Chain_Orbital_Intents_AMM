// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/IntentsEngine.sol";
import "../src/OrbitalAMM.sol";
import "../src/MockUSDC.sol";

/**
 * @title Deploy
 * @dev Deployment script for Holesky testnet
 */
contract Deploy is Script {
    // Configuration
    uint256 constant MIN_SOLVER_STAKE = 1 ether; // 1 ETH minimum stake
    uint256 constant INTENT_FEE = 0.001 ether; // 0.001 ETH per intent
    uint256 constant SLASH_PERCENTAGE = 10; // 10% slash on misbehavior
    uint256 constant AMM_FEE_RATE = 30; // 0.3% swap fee

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        console.log("Deploying contracts with deployer:", deployer);
        console.log("Deployer balance:", deployer.balance);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy MockUSDC first
        console.log("Deploying MockUSDC...");
        MockUSDC mockUSDC = new MockUSDC();
        console.log("MockUSDC deployed at:", address(mockUSDC));
        
        // Deploy IntentsEngine
        console.log("Deploying IntentsEngine...");
        IntentsEngine intentsEngine = new IntentsEngine();
        intentsEngine.initialize(
            deployer, // owner
            deployer, // fee recipient
            MIN_SOLVER_STAKE,
            INTENT_FEE,
            SLASH_PERCENTAGE
        );
        console.log("IntentsEngine deployed at:", address(intentsEngine));
        
        // Deploy OrbitalAMM
        console.log("Deploying OrbitalAMM...");
        OrbitalAMM orbitalAMM = new OrbitalAMM();
        orbitalAMM.initialize(deployer, AMM_FEE_RATE);
        console.log("OrbitalAMM deployed at:", address(orbitalAMM));
        
        // Create ETH/USDC pool with virtual liquidity
        uint256 virtualEthReserve = 100 ether; // Virtual 100 ETH
        uint256 virtualUsdcReserve = 180000 * 10**6; // Virtual 180,000 USDC (assuming 1 ETH = 1800 USDC)
        
        console.log("Creating ETH/USDC pool...");
        uint256 poolId = orbitalAMM.createPool(
            address(0), // ETH (using zero address as placeholder)
            address(mockUSDC),
            virtualEthReserve,
            virtualUsdcReserve
        );
        console.log("ETH/USDC pool created with ID:", poolId);
        
        // Mint some USDC to deployer for testing
        mockUSDC.mint(deployer, 10000 * 10**6); // 10,000 USDC
        console.log("Minted 10,000 USDC to deployer for testing");
        
        vm.stopBroadcast();
        
        // Log deployment summary
        console.log("\n=== DEPLOYMENT COMPLETE ===");
        console.log("Network: Holesky Testnet");
        console.log("Deployer:", deployer);
        console.log("MockUSDC:", address(mockUSDC));
        console.log("IntentsEngine:", address(intentsEngine));
        console.log("OrbitalAMM:", address(orbitalAMM));
        console.log("ETH/USDC Pool ID:", poolId);
        console.log("============================");
        
        // Save deployment addresses to file
        string memory deploymentData = string(abi.encodePacked(
            '{\n',
            '  "network": "holesky",\n',
            '  "chainId": 17000,\n',
            '  "deployer": "', vm.toString(deployer), '",\n',
            '  "contracts": {\n',
            '    "mockUSDC": "', vm.toString(address(mockUSDC)), '",\n',
            '    "intentsEngine": "', vm.toString(address(intentsEngine)), '",\n',
            '    "orbitalAMM": "', vm.toString(address(orbitalAMM)), '"\n',
            '  },\n',
            '  "pools": {\n',
            '    "ethUsdc": ', vm.toString(poolId), '\n',
            '  },\n',
            '  "timestamp": ', vm.toString(block.timestamp), '\n',
            '}\n'
        ));
        
        vm.writeFile("deployments/holesky/contracts.json", deploymentData);
        console.log("Deployment data saved to deployments/holesky/contracts.json");
    }
}