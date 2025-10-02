// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract IntentsEngine is Ownable, ReentrancyGuard {
    enum IntentStatus { Created, Matched, Executed, Cancelled, Failed }

    struct Intent {
        address user;
        uint256 sourceChainId;
        uint256 destChainId;
        address sourceToken;
        address destToken;
        uint256 sourceAmount;
        uint256 minDestAmount;
        uint256 deadline;
        uint256 nonce;
        bytes32 dataHash;
        IntentStatus status;
    }

    struct IntentExecution {
        address solver;
        uint256 matchedAt;
        uint256 executedAt;
        uint256 destAmount;
        bytes32 proofHash;
        bool verified;
    }

    struct Solver {
        uint256 stake;
        uint256 reputationScore;
        uint256 successfulIntents;
        uint256 failedIntents;
        uint256 lastActive;
        bool isRegistered;
    }

    mapping(bytes32 => Intent) public intents;
    mapping(address => Solver) public solvers;
    mapping(address => uint256) public userNonces;
    mapping(bytes32 => IntentExecution) public executions;

    uint256 public minSolverStake;
    uint256 public intentFee;
    uint256 public slashPercentage;
    address public feeRecipient;

    event IntentCreated(bytes32 indexed intentId, address indexed user, uint256 timestamp);
    event IntentMatched(bytes32 indexed intentId, address indexed solver, uint256 timestamp);
    event IntentExecuted(bytes32 indexed intentId, address indexed solver, bool success);
    event IntentCancelled(bytes32 indexed intentId, address indexed user);
    event SolverRegistered(address indexed solver, uint256 stake);
    event SolverSlashed(address indexed solver, uint256 amount, bytes32 intentId);

    error IntentNotFound();
    error IntentExpired();
    error UnauthorizedSolver();
    error InsufficientStake();
    error IntentAlreadyMatched();
    error IntentNotMatched();
    error ExecutionFailed();
    error InvalidIntent();

    constructor() Ownable(msg.sender) {}

    function initialize(
        address _owner,
        address _feeRecipient,
        uint256 _minStake,
        uint256 _intentFee,
        uint256 _slashPercentage
    ) external {
        require(_owner != address(0), "Invalid owner");
        require(_feeRecipient != address(0), "Invalid fee recipient");
        
        _transferOwnership(_owner);
        feeRecipient = _feeRecipient;
        minSolverStake = _minStake;
        intentFee = _intentFee;
        slashPercentage = _slashPercentage;
    }

    function createIntent(
        uint256 sourceChainId,
        uint256 destChainId,
        address sourceToken,
        address destToken,
        uint256 sourceAmount,
        uint256 minDestAmount,
        uint256 deadline,
        bytes calldata data
    ) external payable returns (bytes32) {
        require(deadline > block.timestamp, "Intent expired");
        require(sourceAmount > 0 && minDestAmount > 0, "Invalid amounts");
        require(msg.value >= intentFee, "Insufficient fee");

        uint256 nonce = userNonces[msg.sender]++;
        
        bytes32 intentId = keccak256(abi.encodePacked(
            msg.sender,
            sourceChainId,
            destChainId,
            sourceToken,
            destToken,
            sourceAmount,
            minDestAmount,
            deadline,
            nonce
        ));

        intents[intentId] = Intent({
            user: msg.sender,
            sourceChainId: sourceChainId,
            destChainId: destChainId,
            sourceToken: sourceToken,
            destToken: destToken,
            sourceAmount: sourceAmount,
            minDestAmount: minDestAmount,
            deadline: deadline,
            nonce: nonce,
            dataHash: keccak256(data),
            status: IntentStatus.Created
        });

        // Transfer source tokens to escrow
        if (sourceToken != address(0)) {
            IERC20(sourceToken).transferFrom(msg.sender, address(this), sourceAmount);
        }

        // Transfer intent fee
        if (msg.value > intentFee) {
            payable(msg.sender).transfer(msg.value - intentFee);
        }
        payable(feeRecipient).transfer(intentFee);

        emit IntentCreated(intentId, msg.sender, block.timestamp);
        return intentId;
    }

    function matchIntent(bytes32 intentId) external {
        Solver storage solver = solvers[msg.sender];
        require(solver.isRegistered && solver.stake >= minSolverStake, "Unauthorized solver");

        Intent storage intent = intents[intentId];
        require(intent.status == IntentStatus.Created, "Intent not available");
        require(intent.deadline > block.timestamp, "Intent expired");

        intent.status = IntentStatus.Matched;
        
        executions[intentId] = IntentExecution({
            solver: msg.sender,
            matchedAt: block.timestamp,
            executedAt: 0,
            destAmount: 0,
            proofHash: bytes32(0),
            verified: false
        });

        solver.lastActive = block.timestamp;

        emit IntentMatched(intentId, msg.sender, block.timestamp);
    }

    function executeIntent(
        bytes32 intentId,
        uint256 destAmount,
        bytes calldata proof
    ) external nonReentrant {
        IntentExecution storage execution = executions[intentId];
        require(execution.solver == msg.sender, "Unauthorized solver");

        Intent storage intent = intents[intentId];
        require(intent.status == IntentStatus.Matched, "Intent not matched");
        require(destAmount >= intent.minDestAmount, "Insufficient output");

        intent.status = IntentStatus.Executed;
        execution.executedAt = block.timestamp;
        execution.destAmount = destAmount;
        execution.proofHash = keccak256(proof);
        execution.verified = true;

        // Update solver reputation
        Solver storage solver = solvers[msg.sender];
        solver.successfulIntents++;
        solver.reputationScore += 10;
        solver.lastActive = block.timestamp;

        // Release escrowed tokens to solver
        if (intent.sourceToken != address(0)) {
            IERC20(intent.sourceToken).transfer(msg.sender, intent.sourceAmount);
        } else {
            payable(msg.sender).transfer(intent.sourceAmount);
        }

        emit IntentExecuted(intentId, msg.sender, true);
    }

    function cancelIntent(bytes32 intentId) external {
        Intent storage intent = intents[intentId];
        require(intent.user == msg.sender, "Not intent owner");
        require(intent.status == IntentStatus.Created, "Cannot cancel");

        intent.status = IntentStatus.Cancelled;

        // Return escrowed tokens
        if (intent.sourceToken != address(0)) {
            IERC20(intent.sourceToken).transfer(msg.sender, intent.sourceAmount);
        } else {
            payable(msg.sender).transfer(intent.sourceAmount);
        }

        emit IntentCancelled(intentId, msg.sender);
    }

    function registerSolver(uint256 stakeAmount) external payable {
        require(msg.value >= minSolverStake || stakeAmount >= minSolverStake, "Insufficient stake");
        
        uint256 totalStake = msg.value + stakeAmount;
        require(totalStake >= minSolverStake, "Insufficient total stake");

        Solver storage solver = solvers[msg.sender];
        solver.stake += totalStake;
        solver.isRegistered = true;
        solver.lastActive = block.timestamp;

        // Initialize reputation if new solver
        if (solver.reputationScore == 0) {
            solver.reputationScore = 5000; // Start with 50% reputation
        }

        emit SolverRegistered(msg.sender, solver.stake);
    }

    function slashSolver(address solverAddress, bytes32 intentId) external onlyOwner {
        Solver storage solver = solvers[solverAddress];
        uint256 slashAmount = (solver.stake * slashPercentage) / 100;
        
        solver.stake -= slashAmount;
        solver.failedIntents++;
        
        if (solver.reputationScore >= 20) {
            solver.reputationScore -= 20;
        } else {
            solver.reputationScore = 0;
        }

        if (solver.stake < minSolverStake) {
            solver.isRegistered = false;
        }

        // Transfer slashed amount to fee recipient
        payable(feeRecipient).transfer(slashAmount);

        emit SolverSlashed(solverAddress, slashAmount, intentId);
    }

    function getIntent(bytes32 intentId) external view returns (Intent memory) {
        return intents[intentId];
    }

    function getExecution(bytes32 intentId) external view returns (IntentExecution memory) {
        return executions[intentId];
    }

    function getSolver(address solverAddress) external view returns (Solver memory) {
        return solvers[solverAddress];
    }

    // Emergency functions
    function emergencyWithdraw(address token, uint256 amount) external onlyOwner {
        if (token == address(0)) {
            payable(owner()).transfer(amount);
        } else {
            IERC20(token).transfer(owner(), amount);
        }
    }
}