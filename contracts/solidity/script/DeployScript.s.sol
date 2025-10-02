// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../MockUSDC.sol";
import "../OrbitalAMM.sol";
import "../IntentsEngine.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);

        // Deploy Mock USDC
        MockUSDC usdc = new MockUSDC();
        console.log("Mock USDC deployed at:", address(usdc));

        // Deploy Orbital AMM
        OrbitalAMM orbitalAMM = new OrbitalAMM();
        orbitalAMM.initialize(deployer, 30); // 0.3% fee
        console.log("Orbital AMM deployed at:", address(orbitalAMM));

        // Deploy Intents Engine
        IntentsEngine intentsEngine = new IntentsEngine();
        intentsEngine.initialize(
            deployer,                    // owner
            deployer,                    // fee recipient
            1 ether,                     // min solver stake: 1 ETH
            0.001 ether,                 // intent fee: 0.001 ETH
            10                           // slash percentage: 10%
        );
        console.log("Intents Engine deployed at:", address(intentsEngine));

        // Create initial pool: ETH/USDC
        uint256 poolId = orbitalAMM.createPool(
            address(0),           // ETH (represented as zero address)
            address(usdc),        // USDC
            10 ether,            // virtual ETH reserve
            20000 * 10**6        // virtual USDC reserve (20,000 USDC)
        );
        console.log("Created ETH/USDC pool with ID:", poolId);

        // Mint some USDC to deployer for testing
        usdc.mint(deployer, 100000 * 10**6); // 100k USDC
        console.log("Minted 100,000 USDC to deployer");

        // Add initial liquidity
        usdc.approve(address(orbitalAMM), 50000 * 10**6);
        orbitalAMM.addLiquidity{value: 20 ether}(poolId, 20 ether, 50000 * 10**6);
        console.log("Added initial liquidity: 20 ETH + 50,000 USDC");

        vm.stopBroadcast();

        // Write deployment info to file
        string memory deploymentInfo = string(abi.encodePacked(
            '{\n',
            '  "MockUSDC": "', vm.toString(address(usdc)), '",\n',
            '  "OrbitalAMM": "', vm.toString(address(orbitalAMM)), '",\n',
            '  "IntentsEngine": "', vm.toString(address(intentsEngine)), '",\n',
            '  "ETH_USDC_Pool": ', vm.toString(poolId), ',\n',
            '  "deployer": "', vm.toString(deployer), '",\n',
            '  "network": "holesky",\n',
            '  "chainId": 17000\n',
            '}'
        ));

        vm.writeFile("deployment-addresses.json", deploymentInfo);
        console.log("Deployment info written to deployment-addresses.json");
    }
}