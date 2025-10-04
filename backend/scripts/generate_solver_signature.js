#!/usr/bin/env node

const { ethers } = require('ethers');
const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

function question(query) {
    return new Promise(resolve => rl.question(query, resolve));
}

async function main() {
    console.log("=== Orbital Intents Solver Registration Signature Generator ===\n");

    try {
        // Get private key
        const privateKey = await question("Enter your solver private key (will not be displayed): ");
        rl.output.write('\n');
        
        // Create wallet
        const wallet = new ethers.Wallet(privateKey);
        console.log(`Solver Address: ${wallet.address}`);
        
        // Get registration parameters
        const bondAmountEth = await question("Enter bond amount in ETH: ");
        const bondAmount = ethers.parseEther(bondAmountEth);
        
        const chainsInput = await question("Enter supported chain IDs (comma-separated, e.g., 1,137,42161): ");
        const supportedChains = chainsInput.split(',').map(c => parseInt(c.trim()));
        
        const feeRateInput = await question("Enter fee rate in basis points (e.g., 30 for 0.3%): ");
        const feeRate = parseFloat(feeRateInput);
        
        // Create the message
        const chainsStr = supportedChains.join(",");
        const message = `Orbital Intents Solver Registration
Solver Address: ${wallet.address}
Bond Amount: ${bondAmount.toString()} wei
Supported Chains: [${chainsStr}]
Fee Rate: ${feeRate} bps

By signing this message, I confirm that:
- I am the owner of the solver address
- I agree to the solver terms and conditions
- I understand that my bond may be slashed for misbehavior`;
        
        console.log("\n--- Message to Sign ---");
        console.log(message);
        console.log("--- End Message ---\n");
        
        // Sign the message
        const signature = await wallet.signMessage(message);
        
        // Create the registration request
        const registrationRequest = {
            solver_address: wallet.address,
            bond_amount: bondAmount.toString(),
            supported_chains: supportedChains,
            fee_rate: feeRate,
            signature: signature
        };
        
        console.log("\n=== Registration Request JSON ===");
        console.log(JSON.stringify(registrationRequest, null, 2));
        
        console.log("\n=== Instructions ===");
        console.log("1. Copy the above JSON");
        console.log("2. Send a POST request to: /api/v1/solver/register");
        console.log("3. Include 'Content-Type: application/json' header");
        console.log("\nExample curl command:");
        console.log(`curl -X POST https://api.orbital-intents.io/api/v1/solver/register \\
  -H "Content-Type: application/json" \\
  -d '${JSON.stringify(registrationRequest)}'`);
        
    } catch (error) {
        console.error("\nError:", error.message);
    } finally {
        rl.close();
    }
}

// Run the script
main().catch(console.error);