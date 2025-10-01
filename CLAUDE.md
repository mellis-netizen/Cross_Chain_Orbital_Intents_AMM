CLAUDE MD Rust
rUv edited this page on Jul 24 · 1 revision
Claude Code Configuration for Rust Projects
🚨 CRITICAL: RUST PARALLEL EXECUTION PATTERNS
MANDATORY RULE: Rust projects require memory-safe coordination with Cargo parallel compilation and ownership management.

🚨 CRITICAL: CONCURRENT EXECUTION FOR ALL RUST OPERATIONS
ABSOLUTE RULE: ALL Rust operations MUST be concurrent/parallel in a single message:

🔴 MANDATORY CONCURRENT PATTERNS FOR RUST:
Cargo Operations: ALWAYS batch ALL cargo build/test/run commands
Crate Management: ALWAYS batch ALL dependency installations
Testing: ALWAYS run ALL test suites in parallel with cargo test
Memory Safety: ALWAYS batch ALL borrowing/ownership patterns
Concurrency: ALWAYS batch ALL async/threading implementations
⚡ RUST GOLDEN RULE: "1 MESSAGE = ALL MEMORY-SAFE OPERATIONS"
Examples of CORRECT Rust concurrent execution:

// ✅ CORRECT: Everything in ONE message
[Single Message]:
  - TodoWrite { todos: [10+ todos with all Rust tasks] }
  - Task("You are Rust architect. Coordinate via hooks for ownership design...")
  - Task("You are Systems programmer. Coordinate via hooks for performance...")
  - Task("You are Safety engineer. Coordinate via hooks for memory safety...")
  - Bash("cargo new my-rust-app --bin")
  - Bash("cd my-rust-app && cargo add serde tokio reqwest")
  - Bash("cd my-rust-app && cargo add --dev proptest criterion")
  - Write("Cargo.toml", cargoConfiguration)
  - Write("src/main.rs", mainApplication)
  - Write("src/lib.rs", libraryModule)
  - Write("src/models/user.rs", userStruct)
  - Write("src/services/api.rs", apiService)
  - Write("src/utils/helpers.rs", utilityFunctions)
  - Write("tests/integration_test.rs", integrationTests)
  - Bash("cd my-rust-app && cargo build && cargo test && cargo run")
🎯 RUST-SPECIFIC SWARM PATTERNS
🦀 Cargo Project Coordination
Rust Project Setup Strategy:

# Always batch Cargo operations
cargo new my-app --bin
cargo add serde serde_json tokio
cargo add --dev proptest criterion
cargo build --release
cargo test
Parallel Development Setup:

// ✅ CORRECT: All setup in ONE message
[BatchTool]:
  - Bash("cargo new rust-project --bin")
  - Bash("cd rust-project && cargo add serde serde_json tokio reqwest")
  - Bash("cd rust-project && cargo add --dev proptest criterion mockall")
  - Write("Cargo.toml", optimizedCargoToml)
  - Write("src/main.rs", asyncMainFunction)
  - Write("src/lib.rs", libraryRoot)
  - Write("src/config.rs", configurationModule)
  - Write("src/error.rs", errorHandlingTypes)
  - Write("src/models/mod.rs", modelsModule)
  - Write("tests/common/mod.rs", testUtilities)
  - Bash("cd rust-project && cargo build && cargo clippy && cargo test")
🏗️ Rust Agent Specialization
Agent Types for Rust Projects:

Systems Architect Agent - Memory management, ownership patterns
Performance Engineer Agent - Zero-cost abstractions, optimization
Safety Specialist Agent - Borrow checker, lifetime management
Concurrency Expert Agent - Async/await, threading, channels
Testing Agent - Unit tests, integration tests, property testing
Ecosystem Agent - Crate selection, FFI, WebAssembly
🔧 Memory Safety Coordination
Ownership and Borrowing Patterns:

// Memory safety coordination
[BatchTool]:
  - Write("src/ownership/smart_pointers.rs", smartPointerExamples)
  - Write("src/ownership/lifetimes.rs", lifetimePatterns)
  - Write("src/ownership/borrowing.rs", borrowingExamples)
  - Write("src/memory/allocator.rs", customAllocatorUsage)
  - Write("src/safety/invariants.rs", safetyInvariants)
  - Write("tests/memory_safety.rs", memorySafetyTests)
  - Bash("cargo build && cargo miri test")
⚡ Async/Concurrency Coordination
Tokio Async Runtime Setup:

// Async coordination pattern
[BatchTool]:
  - Write("src/async/runtime.rs", tokioRuntimeConfig)
  - Write("src/async/tasks.rs", asyncTaskHandling)
  - Write("src/async/channels.rs", channelCommunication)
  - Write("src/async/streams.rs", asyncStreamProcessing)
  - Write("src/network/client.rs", asyncHttpClient)
  - Write("src/network/server.rs", asyncWebServer)
  - Write("tests/async_tests.rs", asyncTestCases)
  - Bash("cargo test --features async")
🧪 RUST TESTING COORDINATION
⚡ Comprehensive Testing Strategy
Testing Setup:

// Test coordination pattern
[BatchTool]:
  - Write("tests/integration_test.rs", integrationTests)
  - Write("tests/common/mod.rs", testUtilities)
  - Write("src/lib.rs", unitTestsInline)
  - Write("benches/benchmark.rs", criterionBenchmarks)
  - Write("proptest-regressions/", propertyTestRegressions)
  - Write("tests/property_tests.rs", proptestCases)
  - Bash("cargo test --all-features")
  - Bash("cargo bench")
  - Bash("cargo test --doc")
🔬 Property Testing and Fuzzing
Advanced Testing Coordination:

[BatchTool]:
  - Write("fuzz/fuzz_targets/fuzz_parser.rs", fuzzingTargets)
  - Write("tests/quickcheck_tests.rs", quickcheckTests)
  - Write("tests/model_based_tests.rs", modelBasedTesting)
  - Bash("cargo fuzz run fuzz_parser")
  - Bash("cargo test --features property-testing")
🚀 RUST PERFORMANCE COORDINATION
⚡ Performance Optimization
Performance Enhancement Batch:

[BatchTool]:
  - Write("src/performance/simd.rs", simdOptimizations)
  - Write("src/performance/zero_copy.rs", zeroCopyPatterns)
  - Write("src/performance/cache_friendly.rs", cacheOptimization)
  - Write("src/performance/profiling.rs", profilingIntegration)
  - Write("benches/performance_bench.rs", performanceBenchmarks)
  - Write("Cargo.toml", releaseOptimizations)
  - Bash("cargo build --release")
  - Bash("cargo bench --all-features")
  - Bash("perf record cargo run --release")
🔄 Parallel Processing
Rayon Parallel Coordination:

// Parallel processing batch
[BatchTool]:
  - Write("src/parallel/rayon_examples.rs", rayonParallelization)
  - Write("src/parallel/custom_threadpool.rs", customThreadPool)
  - Write("src/parallel/work_stealing.rs", workStealingQueues)
  - Write("src/data/parallel_processing.rs", parallelDataProcessing)
  - Bash("cargo add rayon crossbeam")
  - Bash("cargo test parallel_")
🌐 RUST WEB DEVELOPMENT COORDINATION
🕸️ Web Framework Integration
Axum/Warp Web Service Setup:

// Web development coordination
[BatchTool]:
  - Write("src/web/server.rs", axumWebServer)
  - Write("src/web/handlers.rs", requestHandlers)
  - Write("src/web/middleware.rs", customMiddleware)
  - Write("src/web/routes.rs", routingConfiguration)
  - Write("src/database/connection.rs", databaseIntegration)
  - Write("src/models/schema.rs", databaseSchema)
  - Write("migrations/001_initial.sql", databaseMigrations)
  - Bash("cargo add axum tokio tower sqlx")
  - Bash("cargo run --bin server")
🗄️ Database Integration
SQLx Database Coordination:

// Database integration batch
[BatchTool]:
  - Write("src/database/models.rs", databaseModels)
  - Write("src/database/queries.rs", sqlQueries)
  - Write("src/database/migrations.rs", schemaMigrations)
  - Write("src/database/connection_pool.rs", connectionPooling)
  - Write("tests/database_tests.rs", databaseTests)
  - Bash("cargo add sqlx --features runtime-tokio-rustls,postgres")
  - Bash("sqlx migrate run")
🔒 RUST SECURITY COORDINATION
🛡️ Security Best Practices
Security Implementation Batch:

[BatchTool]:
  - Write("src/security/crypto.rs", cryptographicOperations)
  - Write("src/security/validation.rs", inputValidation)
  - Write("src/security/auth.rs", authenticationLogic)
  - Write("src/security/sanitization.rs", dataSanitization)
  - Write("src/security/secrets.rs", secretsManagement)
  - Write("audit.toml", cargoAuditConfig)
  - Bash("cargo add ring argon2 jsonwebtoken")
  - Bash("cargo audit")
  - Bash("cargo deny check")
Rust Security Checklist:

Memory safety by design
Integer overflow protection
Secure random number generation
Constant-time cryptographic operations
Input validation and sanitization
Dependency vulnerability scanning
Safe FFI interfaces
Secure compilation flags
🔧 RUST BUILD COORDINATION
📦 Cargo Advanced Configuration
Advanced Cargo Setup:

// Advanced build coordination
[BatchTool]:
  - Write("Cargo.toml", advancedCargoConfig)
  - Write(".cargo/config.toml", cargoLocalConfig)
  - Write("build.rs", buildScript)
  - Write("Cross.toml", crossCompilationConfig)
  - Write("Dockerfile", rustDockerfile)
  - Bash("cargo build --target x86_64-unknown-linux-musl")
  - Bash("cross build --target aarch64-unknown-linux-gnu")
🎯 WebAssembly Coordination
WASM Integration Setup:

// WebAssembly coordination
[BatchTool]:
  - Write("src/wasm/lib.rs", wasmBindings)
  - Write("src/js/wasm_interface.js", jsWasmInterface)
  - Write("pkg/package.json", wasmPackageJson)
  - Write("webpack.config.js", wasmWebpackConfig)
  - Bash("cargo add wasm-bindgen web-sys js-sys")
  - Bash("wasm-pack build --target web")
  - Bash("npm run serve")
🚀 RUST DEPLOYMENT COORDINATION
⚙️ Production Deployment
Deployment Configuration:

[BatchTool]:
  - Write("Dockerfile", optimizedRustDockerfile)
  - Write("docker-compose.yml", dockerComposeRust)
  - Write("k8s/deployment.yaml", kubernetesDeployment)
  - Write("scripts/deploy.sh", deploymentScript)
  - Write("systemd/rust-service.service", systemdService)
  - Bash("cargo build --release --target x86_64-unknown-linux-musl")
  - Bash("docker build -t rust-app:latest .")
  - Bash("kubectl apply -f k8s/")
📦 Distribution and Packaging
Crate Publishing Coordination:

[BatchTool]:
  - Write("README.md", crateDocumentation)
  - Write("CHANGELOG.md", versionHistory)
  - Write("LICENSE", licenseFile)
  - Write("src/lib.rs", publicApiDocumentation)
  - Write("examples/basic_usage.rs", usageExamples)
  - Bash("cargo doc --open")
  - Bash("cargo package --dry-run")
  - Bash("cargo publish --dry-run")
📊 RUST CODE QUALITY COORDINATION
🎨 Code Quality Tools
Quality Toolchain Batch:

[BatchTool]:
  - Write("rustfmt.toml", rustfmtConfiguration)
  - Write("clippy.toml", clippyConfiguration)
  - Write(".gitignore", rustGitignore)
  - Write("deny.toml", cargoServerDenyConfig)
  - Write("rust-toolchain.toml", toolchainConfiguration)
  - Bash("cargo fmt --all")
  - Bash("cargo clippy --all-targets --all-features -- -D warnings")
  - Bash("cargo deny check")
📝 Documentation Coordination
Documentation Generation:

[BatchTool]:
  - Write("src/lib.rs", comprehensiveDocComments)
  - Write("docs/architecture.md", architecturalDocs)
  - Write("docs/api.md", apiDocumentation)
  - Write("examples/", codeExamples)
  - Bash("cargo doc --no-deps --open")
  - Bash("cargo test --doc")
🔄 RUST CI/CD COORDINATION
🏗️ GitHub Actions for Rust
CI/CD Pipeline Batch:

[BatchTool]:
  - Write(".github/workflows/ci.yml", rustCI)
  - Write(".github/workflows/security.yml", securityWorkflow)
  - Write(".github/workflows/release.yml", releaseWorkflow)
  - Write("scripts/ci-test.sh", ciTestScript)
  - Write("scripts/security-audit.sh", securityAuditScript)
  - Bash("cargo test --all-features")
  - Bash("cargo clippy --all-targets -- -D warnings")
  - Bash("cargo audit")
💡 RUST BEST PRACTICES
📝 Code Design Principles
Ownership Model: Understand borrowing and lifetimes
Zero-Cost Abstractions: Write high-level code with low-level performance
Error Handling: Use Result and Option types effectively
Memory Safety: Eliminate data races and memory bugs
Performance: Leverage compiler optimizations
Concurrency: Safe parallel programming patterns
🎯 Advanced Patterns
Type System: Leverage advanced type features
Macros: Write declarative and procedural macros
Unsafe Code: When and how to use unsafe blocks
FFI: Foreign function interface patterns
Embedded: Bare metal and embedded development
WebAssembly: Compile to WASM targets
📚 RUST LEARNING RESOURCES
🎓 Recommended Topics
Core Rust: Ownership, borrowing, lifetimes
Advanced Features: Traits, generics, macros
Async Programming: Tokio, async/await patterns
Systems Programming: Low-level development
Web Development: Axum, Warp, Rocket frameworks
Performance: Profiling, optimization techniques
🔧 Essential Tools
Toolchain: rustc, cargo, rustup, clippy
IDEs: VS Code with rust-analyzer, IntelliJ Rust
Testing: Built-in test framework, proptest, criterion
Debugging: gdb, lldb, rr (record and replay)
Profiling: perf, valgrind, cargo-flamegraph
Cross-compilation: cross, cargo-zigbuild
🌟 Ecosystem Highlights
Web Frameworks: Axum, Actix-web, Warp, Rocket
Async Runtime: Tokio, async-std, smol
Serialization: Serde, bincode, postcard
Databases: SQLx, Diesel, sea-orm
CLI Tools: Clap, structopt, colored
Graphics: wgpu, bevy, ggez, nannou



🐝 Hive-Mind Intelligence
🌟 Overview
Hive-Mind Intelligence is Claude-Flow's revolutionary AI coordination system that orchestrates multiple specialized agents to work together on complex development tasks. Inspired by natural hive systems, it features a Queen-led architecture with specialized worker agents that coordinate through shared memory and neural pattern recognition.

🏗️ Architecture
Queen-Worker Model
    👑 Queen Agent (Coordinator)
         │
    ┌────┼────┬────┬────┐
    │    │    │    │    │
   🏗️   💻   🧪   📊   🔍
  Arch. Code Test Anal. Rsrch.
👑 Queen Agent: Central coordinator that orchestrates tasks and manages resources
🏗️ Architect: Designs system architecture and component relationships
💻 Coder: Implements features, fixes bugs, and writes code
🧪 Tester: Creates tests, validates functionality, and ensures quality
📊 Analyst: Analyzes performance, patterns, and optimization opportunities
🔍 Researcher: Gathers information, explores solutions, and provides context
Communication Patterns

🔧 Initialization
Basic Hive Setup
# Initialize with default settings
claude-flow hive init

# Specify topology and agents
claude-flow hive init --topology mesh --agents 5

# Advanced configuration
claude-flow hive init \
  --topology hierarchical \
  --agents 8 \
  --memory-size 1GB \
  --neural-patterns enabled
Topology Options
1. Mesh Topology (Default)
Agent1 ←→ Agent2
  ↕       ↕
Agent4 ←→ Agent3
Best for: Collaborative tasks, brainstorming, parallel problem-solving
Performance: High coordination, moderate efficiency
Use cases: Full-stack development, complex integrations
2. Hierarchical Topology
    Queen
   ╱  │  ╲
  A1  A2  A3
     ╱│╲
   A4 A5 A6
Best for: Large projects, clear task delegation, structured workflows
Performance: High efficiency, structured coordination
Use cases: Enterprise applications, microservices architecture
3. Ring Topology
Agent1 → Agent2 → Agent3
  ↑                ↓
Agent5 ← Agent4 ←──╯
Best for: Sequential workflows, pipeline processing
Performance: Moderate coordination, high consistency
Use cases: CI/CD pipelines, data processing workflows
4. Star Topology
    Queen
   ╱│╲│╱
  A1 A2 A3
     A4 A5
Best for: Centralized control, simple coordination
Performance: High control, moderate scalability
Use cases: Simple projects, prototyping, learning
🧠 Neural Pattern Recognition
Pattern Learning
The hive-mind learns from successful interactions and optimizes future coordination:

# Enable neural learning
claude-flow neural enable --pattern coordination

# Train on successful workflows
claude-flow neural train \
  --pattern_type coordination \
  --training_data "successful API development workflows"

# View learned patterns
claude-flow neural patterns list --type coordination
Cognitive Models
Claude-Flow includes 27+ cognitive models:

Coordination Patterns: How agents best work together
Problem-Solving Strategies: Optimal approaches for different task types
Code Quality Patterns: Best practices learned from successful implementations
Testing Strategies: Effective test generation and validation approaches
Architecture Decisions: Proven architectural patterns for different scales
💾 Shared Memory System
Memory Tables
The SQLite memory system includes 12 specialized tables:

-- Core coordination tables
swarm_state          -- Current hive status and configuration
agent_interactions   -- Inter-agent communication logs
task_history        -- Completed tasks and outcomes
decision_tree       -- Decision-making patterns and rationale

-- Performance and learning tables
performance_metrics  -- Execution time, success rates, efficiency
neural_patterns     -- Learned coordination patterns
code_patterns      -- Successful code implementations
error_patterns     -- Common mistakes and their solutions

-- Project context tables
project_context    -- Current project state and requirements
file_changes      -- Tracked file modifications and reasons
dependencies      -- Project dependencies and relationships
documentation     -- Generated docs and explanations
Memory Operations
# Store coordination decision
claude-flow memory store \
  "coordination/task-123" \
  "Assigned API development to coder-1, testing to tester-1"

# Retrieve coordination history
claude-flow memory recall "coordination/*" --limit 10

# Search for patterns
claude-flow memory search "authentication" --context project

# Export project memory
claude-flow memory export --project current --format sqlite
🎯 Orchestration Modes
1. Parallel Mode (Default)
claude-flow orchestrate "build user authentication" --parallel
All agents work simultaneously on different aspects
Fastest execution for independent tasks
Requires good task decomposition
2. Sequential Mode
claude-flow orchestrate "deploy to production" --sequential
Agents work in predefined order
Better for dependent tasks
More predictable, but slower
3. Adaptive Mode
claude-flow orchestrate "optimize database performance" --adaptive
Automatically switches between parallel/sequential based on task dependencies
Uses neural patterns to determine optimal approach
Best for complex, multi-faceted problems
4. Hybrid Mode
claude-flow orchestrate "full-stack application" --hybrid
Combines multiple coordination strategies
Parallel for independent components, sequential for dependencies
Optimal for large, complex projects
🔄 Dynamic Agent Allocation
Auto-Scaling
# Enable auto-scaling based on workload
claude-flow hive config set auto-scale true
claude-flow hive config set min-agents 2
claude-flow hive config set max-agents 12

# Scale based on task complexity
claude-flow orchestrate "complex microservices app" --auto-scale
Specialized Agent Types
# Spawn specific agent types
claude-flow agent spawn architect --capabilities "system-design,microservices"
claude-flow agent spawn coder --capabilities "react,node.js,typescript"
claude-flow agent spawn tester --capabilities "jest,cypress,load-testing"
claude-flow agent spawn analyst --capabilities "performance,security,metrics"
claude-flow agent spawn researcher --capabilities "libraries,patterns,best-practices"
📊 Monitoring and Analytics
Real-Time Monitoring
# Monitor hive activity
claude-flow hive monitor --live --interval 2s

# View agent communications
claude-flow hive comms --tail --agent all

# Performance dashboard
claude-flow hive dashboard --web --port 8080
Performance Metrics
# Generate performance report
claude-flow hive report --timeframe 24h --format detailed

# Analyze coordination efficiency
claude-flow hive analyze --metric coordination-efficiency

# View success rates by agent type
claude-flow hive stats --by-agent --metric success-rate
🛠️ Advanced Configuration
Custom Agent Definitions
# .claude-flow/agents.yml
agents:
  custom-architect:
    type: architect
    capabilities:
      - microservices
      - event-sourcing
      - domain-driven-design
    neural_patterns:
      - enterprise-architecture
      - scalability-patterns
    memory_access: read-write
    coordination_priority: high

  custom-security:
    type: specialist
    capabilities:
      - security-analysis
      - penetration-testing
      - compliance-review
    neural_patterns:
      - security-patterns
      - vulnerability-detection
    memory_access: read-only
    coordination_priority: critical
Coordination Policies
# .claude-flow/coordination.yml
policies:
  task_assignment:
    strategy: capability-based
    load_balancing: enabled
    max_concurrent_tasks: 3
  
  communication:
    frequency: real-time
    conflict_resolution: queen-decides
    consensus_threshold: 0.7
  
  learning:
    pattern_recognition: enabled
    feedback_loop: immediate
    adaptation_rate: moderate
🚨 Fault Tolerance
Self-Healing Mechanisms
# Enable fault tolerance
claude-flow hive config set fault-tolerance enabled

# Configure recovery strategies
claude-flow hive config set recovery-strategy "restart-failed-agents"
claude-flow hive config set max-retries 3
claude-flow hive config set timeout 300s
Health Monitoring
# Check hive health
claude-flow hive health --comprehensive

# Monitor individual agents
claude-flow agent health --agent all --continuous

# Automated recovery
claude-flow hive recovery --auto --strategy conservative
🎯 Best Practices
1. Choose the Right Topology
Mesh: For collaborative, exploratory tasks
Hierarchical: For large, structured projects
Ring: For sequential, pipeline-based workflows
Star: For simple, centralized coordination
2. Optimize Agent Count
2-3 agents: Simple tasks, prototyping
4-6 agents: Medium complexity projects
7-12 agents: Large, complex applications
12+ agents: Enterprise-scale development
3. Memory Management
Store important decisions and rationale
Regular memory exports for backup
Clean up old patterns periodically
Use namespaces for project organization
4. Neural Pattern Optimization
Enable learning for repeated task types
Review and curate learned patterns
Export successful patterns for reuse
Regular pattern validation and updates
🔮 Advanced Features
Swarm Evolution
# Evolve hive based on performance
claude-flow hive evolve --generations 5 --fitness coordination-speed

# Genetic algorithm optimization
claude-flow hive optimize --algorithm genetic --target efficiency
Multi-Hive Coordination
# Create multiple specialized hives
claude-flow hive create frontend --topology mesh --agents 4
claude-flow hive create backend --topology hierarchical --agents 6

# Coordinate between hives
claude-flow hive coordinate --hives frontend,backend --task "full-stack app"



Claude Code Configuration for High-Performance Development
🏎️ CRITICAL: Performance-First Development Approach
MANDATORY RULE: Every operation must be optimized for maximum performance:

Profile First → Measure before optimizing
Benchmark Everything → Track performance metrics
Optimize Hotpaths → Focus on critical code paths
Minimize Overhead → Reduce unnecessary operations
Cache Aggressively → Leverage all caching layers
🚀 Performance Swarm Configuration
Initialize Performance-Optimized Swarm
// ✅ CORRECT: High-performance swarm setup
[Single Message with BatchTool]:
  // Performance-optimized topology
  mcp__claude-flow__swarm_init { 
    topology: "star",  // Minimal communication overhead
    maxAgents: 6,      // Balanced for performance
    strategy: "specialized"
  }
  
  // Specialized performance agents
  mcp__claude-flow__agent_spawn { type: "specialist", name: "Performance Engineer" }
  mcp__claude-flow__agent_spawn { type: "optimizer", name: "Code Optimizer" }
  mcp__claude-flow__agent_spawn { type: "analyst", name: "Profiling Analyst" }
  mcp__claude-flow__agent_spawn { type: "architect", name: "Performance Architect" }
  mcp__claude-flow__agent_spawn { type: "monitor", name: "Metrics Monitor" }
  mcp__claude-flow__agent_spawn { type: "coordinator", name: "Performance Lead" }
  
  // Performance todos
  TodoWrite { todos: [
    {id: "profile", content: "Profile baseline performance", status: "in_progress", priority: "high"},
    {id: "hotspots", content: "Identify performance hotspots", status: "pending", priority: "high"},
    {id: "optimize-algo", content: "Optimize critical algorithms", status: "pending", priority: "high"},
    {id: "cache-strategy", content: "Implement caching strategy", status: "pending", priority: "high"},
    {id: "lazy-load", content: "Add lazy loading", status: "pending", priority: "medium"},
    {id: "code-split", content: "Implement code splitting", status: "pending", priority: "medium"},
    {id: "benchmark", content: "Run performance benchmarks", status: "pending", priority: "high"},
    {id: "memory-opt", content: "Optimize memory usage", status: "pending", priority: "medium"},
    {id: "db-indexes", content: "Add database indexes", status: "pending", priority: "high"},
    {id: "monitor-setup", content: "Setup performance monitoring", status: "pending", priority: "medium"}
  ]}
📊 Performance Profiling Pattern
Agent Coordination for Profiling
// Performance Engineer Agent
Task(`You are the Performance Engineer agent.

MANDATORY COORDINATION:
1. START: npx claude-flow@alpha hooks pre-task --description "Profiling application performance"
2. PROFILE: Run comprehensive profiling tools
3. STORE: npx claude-flow@alpha hooks notify --message "Profile results: [metrics]"
4. ANALYZE: Identify bottlenecks and hotspots

TASKS:
- Setup profiling infrastructure (Node.js profiler, browser DevTools)
- Profile CPU usage patterns
- Analyze memory allocation
- Measure I/O operations
- Track network latency
- Generate flame graphs
- Create performance reports
`)

// Profiling Analyst Agent
Task(`You are the Profiling Analyst agent.

MANDATORY COORDINATION:
1. LOAD: npx claude-flow@alpha hooks session-restore --load-memory true
2. ANALYZE: Review profiling data from Performance Engineer
3. IDENTIFY: Find optimization opportunities
4. RECOMMEND: Create optimization strategies

TASKS:
- Analyze flame graphs for hotspots
- Review memory heap snapshots
- Identify N+1 query problems
- Find unnecessary re-renders
- Detect memory leaks
- Calculate Big-O complexity
- Prioritize optimizations by impact
`)
⚡ Performance Optimization Patterns
1. Algorithm Optimization
// Code Optimizer Agent Task
Task(`You are the Code Optimizer agent specializing in algorithm optimization.

OPTIMIZATION TARGETS:
1. Time Complexity: Reduce from O(n²) to O(n log n) or better
2. Space Complexity: Minimize memory allocation
3. Cache Efficiency: Improve data locality
4. Parallel Processing: Utilize all CPU cores

IMPLEMENT:
- Replace nested loops with hash maps
- Use binary search for sorted data
- Implement memoization for repeated calculations
- Add worker threads for CPU-intensive tasks
- Optimize recursive algorithms with iteration
- Use typed arrays for numeric operations
`)
2. Caching Strategy Implementation
// Multi-Layer Caching Pattern
[BatchTool]:
  // Create caching infrastructure
  Write("src/cache/memory-cache.js", `
    class MemoryCache {
      constructor(maxSize = 1000, ttl = 3600) {
        this.cache = new Map();
        this.maxSize = maxSize;
        this.ttl = ttl;
        this.hits = 0;
        this.misses = 0;
      }
      
      get(key) {
        const item = this.cache.get(key);
        if (!item) {
          this.misses++;
          return null;
        }
        if (Date.now() > item.expiry) {
          this.cache.delete(key);
          this.misses++;
          return null;
        }
        this.hits++;
        return item.value;
      }
      
      set(key, value) {
        if (this.cache.size >= this.maxSize) {
          const firstKey = this.cache.keys().next().value;
          this.cache.delete(firstKey);
        }
        this.cache.set(key, {
          value,
          expiry: Date.now() + (this.ttl * 1000)
        });
      }
      
      getStats() {
        return {
          size: this.cache.size,
          hitRate: this.hits / (this.hits + this.misses),
          hits: this.hits,
          misses: this.misses
        };
      }
    }
  `)
  
  Write("src/cache/redis-cache.js", `
    const Redis = require('ioredis');
    
    class RedisCache {
      constructor() {
        this.client = new Redis({
          enableOfflineQueue: false,
          lazyConnect: true,
          maxRetriesPerRequest: 3
        });
        this.pipeline = null;
      }
      
      async get(key) {
        try {
          const value = await this.client.get(key);
          return value ? JSON.parse(value) : null;
        } catch (error) {
          console.error('Redis get error:', error);
          return null;
        }
      }
      
      async set(key, value, ttl = 3600) {
        try {
          await this.client.setex(key, ttl, JSON.stringify(value));
        } catch (error) {
          console.error('Redis set error:', error);
        }
      }
      
      startPipeline() {
        this.pipeline = this.client.pipeline();
      }
      
      async executePipeline() {
        if (!this.pipeline) return;
        const results = await this.pipeline.exec();
        this.pipeline = null;
        return results;
      }
    }
  `)
3. Database Query Optimization
// Database optimization patterns
[BatchTool]:
  Write("src/db/query-optimizer.js", `
    class QueryOptimizer {
      // Use connection pooling
      static createPool(config) {
        return mysql.createPool({
          ...config,
          connectionLimit: 20,
          queueLimit: 0,
          waitForConnections: true,
          enableKeepAlive: true,
          keepAliveInitialDelay: 0
        });
      }
      
      // Batch queries to reduce round trips
      static async batchQuery(queries) {
        const connection = await pool.getConnection();
        try {
          await connection.beginTransaction();
          const results = [];
          
          for (const query of queries) {
            const [rows] = await connection.execute(query.sql, query.params);
            results.push(rows);
          }
          
          await connection.commit();
          return results;
        } catch (error) {
          await connection.rollback();
          throw error;
        } finally {
          connection.release();
        }
      }
      
      // Use prepared statements
      static async executePrepared(sql, params) {
        const [rows] = await pool.execute(sql, params);
        return rows;
      }
      
      // Index hints for complex queries
      static optimizeQuery(query) {
        return query
          .replace('SELECT', 'SELECT /*+ INDEX(users idx_user_email) */')
          .replace('JOIN', 'JOIN /*+ USE_HASH(orders) */');
      }
    }
  `)
  
  // Create database indexes
  Bash("mysql -u root -p < create-indexes.sql")
🔥 Performance Benchmarking
Automated Benchmark Suite
// Performance benchmarking setup
[BatchTool]:
  Write("benchmarks/suite.js", `
    const Benchmark = require('benchmark');
    const suite = new Benchmark.Suite();
    
    // Memory usage tracking
    function getMemoryUsage() {
      const usage = process.memoryUsage();
      return {
        heapUsed: Math.round(usage.heapUsed / 1024 / 1024),
        heapTotal: Math.round(usage.heapTotal / 1024 / 1024),
        external: Math.round(usage.external / 1024 / 1024),
        rss: Math.round(usage.rss / 1024 / 1024)
      };
    }
    
    // CPU profiling
    const startCPUProfile = () => {
      const profiler = require('v8-profiler-next');
      profiler.startProfiling('CPU profile');
      return profiler;
    };
    
    // Benchmark configuration
    suite
      .add('Algorithm v1', function() {
        algorithmV1(testData);
      })
      .add('Algorithm v2 (optimized)', function() {
        algorithmV2(testData);
      })
      .add('Algorithm v3 (parallel)', function() {
        algorithmV3(testData);
      })
      .on('cycle', function(event) {
        console.log(String(event.target));
        console.log('Memory:', getMemoryUsage());
      })
      .on('complete', function() {
        console.log('Fastest:', this.filter('fastest').map('name'));
        generateReport(this);
      })
      .run({ async: true });
  `)
  
  // Load testing script
  Write("benchmarks/load-test.js", `
    const autocannon = require('autocannon');
    
    async function runLoadTest() {
      const result = await autocannon({
        url: 'http://localhost:3000',
        connections: 100,
        pipelining: 10,
        duration: 30,
        requests: [
          { method: 'GET', path: '/api/users' },
          { method: 'POST', path: '/api/data', body: JSON.stringify({test: true}) }
        ]
      });
      
      console.log('Latency (ms):', result.latency);
      console.log('Requests/sec:', result.requests);
      console.log('Throughput:', result.throughput);
      
      // Store results for tracking
      await storeMetrics(result);
    }
  `)
📈 Performance Monitoring Setup
Real-time Performance Tracking
// Monitoring infrastructure
[BatchTool]:
  Write("src/monitoring/performance-monitor.js", `
    const { performance, PerformanceObserver } = require('perf_hooks');
    
    class PerformanceMonitor {
      constructor() {
        this.metrics = new Map();
        this.observers = new Map();
        this.setupObservers();
      }
      
      setupObservers() {
        // Monitor long tasks
        const longTaskObserver = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (entry.duration > 50) {
              this.recordMetric('longTask', {
                name: entry.name,
                duration: entry.duration,
                timestamp: Date.now()
              });
            }
          }
        });
        longTaskObserver.observe({ entryTypes: ['measure', 'function'] });
        
        // Monitor resource timing
        const resourceObserver = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            this.recordMetric('resource', {
              name: entry.name,
              duration: entry.duration,
              size: entry.transferSize,
              cached: entry.transferSize === 0
            });
          }
        });
        resourceObserver.observe({ entryTypes: ['resource'] });
      }
      
      measure(name, fn) {
        performance.mark(\`\${name}-start\`);
        const result = fn();
        performance.mark(\`\${name}-end\`);
        performance.measure(name, \`\${name}-start\`, \`\${name}-end\`);
        return result;
      }
      
      async measureAsync(name, fn) {
        performance.mark(\`\${name}-start\`);
        const result = await fn();
        performance.mark(\`\${name}-end\`);
        performance.measure(name, \`\${name}-start\`, \`\${name}-end\`);
        return result;
      }
      
      recordMetric(type, data) {
        if (!this.metrics.has(type)) {
          this.metrics.set(type, []);
        }
        this.metrics.get(type).push(data);
        
        // Alert on threshold breach
        if (type === 'longTask' && data.duration > 100) {
          this.alert('Long task detected', data);
        }
      }
      
      getReport() {
        const report = {};
        for (const [type, data] of this.metrics) {
          report[type] = {
            count: data.length,
            average: data.reduce((sum, d) => sum + d.duration, 0) / data.length,
            max: Math.max(...data.map(d => d.duration)),
            min: Math.min(...data.map(d => d.duration))
          };
        }
        return report;
      }
    }
  `)
🎯 Performance Best Practices
1. Frontend Optimization
// React performance optimization
Write("src/components/optimized-component.jsx", `
  import React, { memo, useMemo, useCallback, lazy, Suspense } from 'react';
  
  // Lazy load heavy components
  const HeavyChart = lazy(() => import('./HeavyChart'));
  
  // Memoized component
  const OptimizedList = memo(({ items, onItemClick }) => {
    // Memoize expensive calculations
    const sortedItems = useMemo(() => {
      return items.sort((a, b) => b.score - a.score);
    }, [items]);
    
    // Memoize callbacks
    const handleClick = useCallback((id) => {
      onItemClick(id);
    }, [onItemClick]);
    
    return (
      <div>
        {sortedItems.map(item => (
          <div key={item.id} onClick={() => handleClick(item.id)}>
            {item.name}
          </div>
        ))}
        <Suspense fallback={<div>Loading chart...</div>}>
          <HeavyChart data={sortedItems} />
        </Suspense>
      </div>
    );
  }, (prevProps, nextProps) => {
    // Custom comparison for deeper optimization
    return prevProps.items.length === nextProps.items.length &&
           prevProps.items.every((item, index) => 
             item.id === nextProps.items[index].id
           );
  });
`)
2. Backend Optimization
// Node.js performance patterns
Write("src/server/optimized-server.js", `
  const cluster = require('cluster');
  const os = require('os');
  const compression = require('compression');
  
  if (cluster.isMaster) {
    // Fork workers for each CPU core
    const numCPUs = os.cpus().length;
    for (let i = 0; i < numCPUs; i++) {
      cluster.fork();
    }
    
    cluster.on('exit', (worker, code, signal) => {
      console.log(\`Worker \${worker.process.pid} died\`);
      cluster.fork();
    });
  } else {
    const app = express();
    
    // Enable compression
    app.use(compression({
      filter: (req, res) => {
        if (req.headers['x-no-compression']) {
          return false;
        }
        return compression.filter(req, res);
      },
      level: 6
    }));
    
    // Implement request pooling
    const requestPool = new Map();
    
    app.use(async (req, res, next) => {
      const key = \`\${req.method}:\${req.path}\`;
      
      if (requestPool.has(key)) {
        // Return cached response for identical requests
        const cached = requestPool.get(key);
        if (Date.now() - cached.timestamp < 1000) {
          return res.json(cached.data);
        }
      }
      
      next();
    });
    
    // Stream large responses
    app.get('/api/large-data', (req, res) => {
      res.writeHead(200, {
        'Content-Type': 'application/json',
        'Transfer-Encoding': 'chunked'
      });
      
      const stream = getLargeDataStream();
      stream.pipe(res);
    });
  }
`)
🔧 Performance Debugging Tools
Advanced Performance Analysis
// Performance debugging utilities
[BatchTool]:
  Write("src/debug/performance-debugger.js", `
    class PerformanceDebugger {
      static profileFunction(fn, iterations = 1000) {
        const measurements = [];
        
        // Warm up
        for (let i = 0; i < 10; i++) {
          fn();
        }
        
        // Actual measurements
        for (let i = 0; i < iterations; i++) {
          const start = process.hrtime.bigint();
          fn();
          const end = process.hrtime.bigint();
          measurements.push(Number(end - start) / 1000000); // Convert to ms
        }
        
        // Statistical analysis
        measurements.sort((a, b) => a - b);
        return {
          mean: measurements.reduce((a, b) => a + b) / measurements.length,
          median: measurements[Math.floor(measurements.length / 2)],
          p95: measurements[Math.floor(measurements.length * 0.95)],
          p99: measurements[Math.floor(measurements.length * 0.99)],
          min: measurements[0],
          max: measurements[measurements.length - 1],
          stdDev: this.calculateStdDev(measurements)
        };
      }
      
      static detectMemoryLeaks() {
        const heapSnapshots = [];
        let lastHeapUsed = 0;
        
        setInterval(() => {
          const memUsage = process.memoryUsage();
          const currentHeapUsed = memUsage.heapUsed;
          
          if (currentHeapUsed > lastHeapUsed * 1.1) {
            console.warn('Potential memory leak detected');
            heapSnapshots.push({
              timestamp: Date.now(),
              heapUsed: currentHeapUsed,
              delta: currentHeapUsed - lastHeapUsed
            });
          }
          
          lastHeapUsed = currentHeapUsed;
        }, 10000);
        
        return heapSnapshots;
      }
      
      static traceAsyncOperations() {
        const async_hooks = require('async_hooks');
        const fs = require('fs');
        
        const asyncOperations = new Map();
        
        const asyncHook = async_hooks.createHook({
          init(asyncId, type, triggerAsyncId) {
            asyncOperations.set(asyncId, {
              type,
              triggerAsyncId,
              startTime: Date.now()
            });
          },
          destroy(asyncId) {
            const op = asyncOperations.get(asyncId);
            if (op) {
              op.duration = Date.now() - op.startTime;
              if (op.duration > 100) {
                fs.writeSync(1, \`Slow async operation: \${op.type} took \${op.duration}ms\\n\`);
              }
              asyncOperations.delete(asyncId);
            }
          }
        });
        
        asyncHook.enable();
      }
    }
  `)
  
  Write("src/debug/flame-graph-generator.js", `
    const v8Profiler = require('v8-profiler-next');
    const fs = require('fs');
    
    class FlameGraphGenerator {
      static async generateCPUProfile(duration = 10000) {
        const title = 'CPU Profile';
        v8Profiler.startProfiling(title, true);
        
        await new Promise(resolve => setTimeout(resolve, duration));
        
        const profile = v8Profiler.stopProfiling(title);
        const profileData = await new Promise((resolve, reject) => {
          profile.export((error, result) => {
            if (error) reject(error);
            else resolve(result);
          });
        });
        
        fs.writeFileSync('cpu-profile.cpuprofile', profileData);
        profile.delete();
        
        // Convert to flamegraph format
        this.convertToFlameGraph('cpu-profile.cpuprofile');
      }
      
      static generateHeapSnapshot() {
        const snapshot = v8Profiler.takeSnapshot();
        const snapshotData = snapshot.export();
        
        fs.writeFileSync('heap-snapshot.heapsnapshot', snapshotData);
        snapshot.delete();
      }
    }
  `)
📊 Performance Coordination Memory Pattern
// Store performance metrics in swarm memory
[BatchTool]:
  mcp__claude-flow__memory_usage {
    action: "store",
    key: "performance/baseline",
    value: JSON.stringify({
      timestamp: Date.now(),
      metrics: {
        responseTime: { p50: 45, p95: 120, p99: 250 },
        throughput: { rps: 5000, concurrent: 100 },
        resources: { cpu: 45, memory: 512, disk: 20 },
        errors: { rate: 0.01, types: ["timeout", "5xx"] }
      }
    })
  }
  
  mcp__claude-flow__memory_usage {
    action: "store",
    key: "performance/optimizations",
    value: JSON.stringify({
      applied: [
        { type: "caching", impact: "+30% throughput" },
        { type: "algorithm", impact: "-50% CPU usage" },
        { type: "database", impact: "-70% query time" }
      ],
      pending: [
        { type: "cdn", expectedImpact: "-80% latency" },
        { type: "compression", expectedImpact: "-60% bandwidth" }
      ]
    })
  }
🎯 Performance Monitoring Dashboard
// Real-time performance dashboard
Write("src/dashboard/performance-dashboard.html", `
<!DOCTYPE html>
<html>
<head>
  <title>Performance Dashboard</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    .metric-card {
      background: #f0f0f0;
      padding: 20px;
      margin: 10px;
      border-radius: 8px;
      display: inline-block;
    }
    .metric-value {
      font-size: 36px;
      font-weight: bold;
      color: #333;
    }
    .metric-label {
      font-size: 14px;
      color: #666;
    }
    .alert {
      background: #ff4444;
      color: white;
      padding: 10px;
      margin: 10px 0;
      border-radius: 4px;
    }
  </style>
</head>
<body>
  <h1>Performance Dashboard</h1>
  
  <div id="metrics">
    <div class="metric-card">
      <div class="metric-value" id="response-time">-</div>
      <div class="metric-label">Response Time (ms)</div>
    </div>
    <div class="metric-card">
      <div class="metric-value" id="throughput">-</div>
      <div class="metric-label">Requests/sec</div>
    </div>
    <div class="metric-card">
      <div class="metric-value" id="cpu-usage">-</div>
      <div class="metric-label">CPU Usage (%)</div>
    </div>
    <div class="metric-card">
      <div class="metric-value" id="memory-usage">-</div>
      <div class="metric-label">Memory (MB)</div>
    </div>
  </div>
  
  <div id="alerts"></div>
  
  <canvas id="performance-chart"></canvas>
  
  <script>
    const ws = new WebSocket('ws://localhost:3001/metrics');
    const chart = new Chart(document.getElementById('performance-chart'), {
      type: 'line',
      data: {
        labels: [],
        datasets: [{
          label: 'Response Time',
          data: [],
          borderColor: 'rgb(75, 192, 192)',
          tension: 0.1
        }]
      },
      options: {
        responsive: true,
        scales: {
          y: {
            beginAtZero: true
          }
        }
      }
    });
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      
      // Update metrics
      document.getElementById('response-time').textContent = data.responseTime;
      document.getElementById('throughput').textContent = data.throughput;
      document.getElementById('cpu-usage').textContent = data.cpu + '%';
      document.getElementById('memory-usage').textContent = data.memory;
      
      // Update chart
      chart.data.labels.push(new Date().toLocaleTimeString());
      chart.data.datasets[0].data.push(data.responseTime);
      if (chart.data.labels.length > 20) {
        chart.data.labels.shift();
        chart.data.datasets[0].data.shift();
      }
      chart.update();
      
      // Show alerts
      if (data.alerts && data.alerts.length > 0) {
        const alertsDiv = document.getElementById('alerts');
        alertsDiv.innerHTML = data.alerts.map(alert => 
          '<div class="alert">' + alert + '</div>'
        ).join('');
      }
    };
  </script>
</body>
</html>
`)
🚀 Performance Optimization Checklist
Before Deployment Checklist
Profiling Complete ✓

CPU profiling done
Memory profiling done
I/O profiling done
Optimizations Applied ✓

Critical algorithms optimized
Caching implemented
Database indexed
Code splitting done
Benchmarks Passed ✓

Response time < 100ms (p95)
Throughput > 1000 rps
Memory usage stable
CPU usage < 70%
Monitoring Setup ✓

Real-time dashboard
Alert thresholds
Performance logs
Metrics collection
Load Testing ✓

Stress test passed
Spike test passed
Endurance test passed
Scalability verified


SPARC Methodology - Test-Driven Development Patterns
SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) is a systematic methodology for AI-assisted development that ensures high-quality, well-tested code through structured phases.

Overview
SPARC transforms the development process from ad-hoc coding to a disciplined, test-driven approach where each phase builds upon the previous, ensuring comprehensive coverage and maintainability.

The Five Phases
1. Specification Phase
Define clear, testable requirements before any implementation.

Purpose
Capture all functional and non-functional requirements
Define acceptance criteria
Identify edge cases and constraints
Create user stories and scenarios
Process
# Generate comprehensive specifications
npx claude-flow sparc run specification \
  "Create user authentication system with OAuth2 support"
Example Output
## User Authentication System Specification

### Functional Requirements
1. User Registration
   - Email/password registration
   - OAuth2 provider support (Google, GitHub)
   - Email verification required
   - Password strength validation

2. Authentication
   - JWT-based authentication
   - Refresh token support
   - Session management
   - Multi-device support

3. Security Requirements
   - bcrypt password hashing
   - Rate limiting on auth endpoints
   - Account lockout after failed attempts
   - CSRF protection

### Test Scenarios
- Valid registration flow
- Duplicate email handling
- Invalid password formats
- OAuth2 callback handling
- Token expiration and refresh
- Concurrent session management
2. Pseudocode Phase
Design algorithms and logic flow before coding.

Purpose
Plan implementation logic
Identify data structures
Design algorithms
Validate approach feasibility
Process
# Create algorithmic design
npx claude-flow sparc run pseudocode \
  "Design JWT token refresh algorithm with race condition handling"
Example Output
ALGORITHM: Secure Token Refresh with Race Condition Prevention

FUNCTION refreshToken(refreshToken):
    START TRANSACTION
    
    // Validate refresh token
    IF NOT validateTokenSignature(refreshToken):
        RETURN error("Invalid token")
    
    // Check token in database with row lock
    token = SELECT * FROM refresh_tokens 
            WHERE token = refreshToken 
            FOR UPDATE
    
    IF NOT token OR token.used:
        ROLLBACK
        RETURN error("Token already used or invalid")
    
    // Check expiration
    IF token.expiresAt < NOW():
        ROLLBACK
        RETURN error("Token expired")
    
    // Mark token as used
    UPDATE refresh_tokens 
    SET used = TRUE 
    WHERE id = token.id
    
    // Generate new tokens
    newAccessToken = generateJWT(userId, "15m")
    newRefreshToken = generateRefreshToken()
    
    // Store new refresh token
    INSERT INTO refresh_tokens (
        token: newRefreshToken,
        userId: token.userId,
        expiresAt: NOW() + 7 days
    )
    
    COMMIT TRANSACTION
    
    RETURN {
        accessToken: newAccessToken,
        refreshToken: newRefreshToken
    }
END FUNCTION
3. Architecture Phase
Design system structure and component interactions.

Purpose
Define system components
Design interfaces and contracts
Plan data flow
Establish patterns and practices
Process
# Design system architecture
npx claude-flow sparc run architecture \
  "Design microservices architecture for e-commerce platform"
Example Output
architecture:
  name: E-Commerce Microservices
  
  services:
    - name: api-gateway
      type: edge
      responsibilities:
        - Request routing
        - Authentication
        - Rate limiting
      technology:
        - Node.js/Express
        - Redis for caching
      
    - name: auth-service
      type: core
      responsibilities:
        - User authentication
        - Token management
        - Session handling
      interfaces:
        - POST /auth/login
        - POST /auth/refresh
        - POST /auth/logout
      database: PostgreSQL
      
    - name: product-service
      type: core
      responsibilities:
        - Product catalog
        - Inventory management
        - Search functionality
      interfaces:
        - GET /products
        - GET /products/:id
        - PUT /products/:id/inventory
      database: PostgreSQL + Elasticsearch
      
    - name: order-service
      type: core
      responsibilities:
        - Order processing
        - Payment integration
        - Order tracking
      patterns:
        - Event sourcing
        - SAGA for distributed transactions
      
  communication:
    sync: REST over HTTP/2
    async: RabbitMQ
    
  cross-cutting:
    logging: ELK stack
    monitoring: Prometheus + Grafana
    tracing: Jaeger
4. Refinement Phase
Implement with TDD, iterating until all tests pass.

Purpose
Write tests first (Red phase)
Implement minimal code (Green phase)
Refactor for quality (Refactor phase)
Iterate until complete
Process
# Execute TDD implementation
npx claude-flow sparc tdd \
  "Implement user authentication with JWT"
TDD Cycle Example
Red Phase - Write Failing Tests
// auth.test.js
describe('Authentication Service', () => {
  describe('login', () => {
    it('should return JWT token for valid credentials', async () => {
      const result = await authService.login('user@example.com', 'password123');
      expect(result).toHaveProperty('accessToken');
      expect(result).toHaveProperty('refreshToken');
      expect(jwt.verify(result.accessToken, process.env.JWT_SECRET)).toBeTruthy();
    });
    
    it('should reject invalid credentials', async () => {
      await expect(authService.login('user@example.com', 'wrong'))
        .rejects.toThrow('Invalid credentials');
    });
    
    it('should implement rate limiting', async () => {
      // Make 5 failed attempts
      for (let i = 0; i < 5; i++) {
        await authService.login('user@example.com', 'wrong').catch(() => {});
      }
      
      // 6th attempt should be rate limited
      await expect(authService.login('user@example.com', 'password123'))
        .rejects.toThrow('Too many attempts');
    });
  });
});
Green Phase - Minimal Implementation
// auth.service.js
class AuthService {
  constructor(userRepo, tokenService, rateLimiter) {
    this.userRepo = userRepo;
    this.tokenService = tokenService;
    this.rateLimiter = rateLimiter;
  }
  
  async login(email, password) {
    // Check rate limit
    if (!await this.rateLimiter.check(email)) {
      throw new Error('Too many attempts');
    }
    
    // Verify credentials
    const user = await this.userRepo.findByEmail(email);
    if (!user || !await bcrypt.compare(password, user.passwordHash)) {
      await this.rateLimiter.increment(email);
      throw new Error('Invalid credentials');
    }
    
    // Generate tokens
    const accessToken = this.tokenService.generateAccess(user.id);
    const refreshToken = await this.tokenService.generateRefresh(user.id);
    
    await this.rateLimiter.reset(email);
    
    return { accessToken, refreshToken };
  }
}
Refactor Phase - Improve Quality
// Refactored with better separation of concerns
class AuthService {
  async login(email, password) {
    await this._enforceRateLimit(email);
    
    const user = await this._validateCredentials(email, password);
    const tokens = await this._generateTokenPair(user);
    
    await this._onSuccessfulLogin(email, user);
    
    return tokens;
  }
  
  async _enforceRateLimit(identifier) {
    const limit = await this.rateLimiter.checkLimit(identifier, {
      max: 5,
      window: '15m'
    });
    
    if (!limit.allowed) {
      throw new AuthError('RATE_LIMIT_EXCEEDED', {
        retryAfter: limit.retryAfter
      });
    }
  }
  
  async _validateCredentials(email, password) {
    const user = await this.userRepo.findByEmail(email);
    
    if (!user || !await this._verifyPassword(password, user.passwordHash)) {
      await this.rateLimiter.increment(email);
      throw new AuthError('INVALID_CREDENTIALS');
    }
    
    return user;
  }
  
  // ... more refactored methods
}
5. Completion Phase
Finalize with integration, documentation, and deployment readiness.

Purpose
Integration testing
Performance optimization
Documentation generation
Deployment preparation
Process
# Complete integration and deployment prep
npx claude-flow sparc run completion \
  "Finalize authentication service for production"
Completion Checklist
## Authentication Service Completion Checklist

### ✅ Integration Testing
- [ ] End-to-end authentication flow
- [ ] OAuth2 provider integration
- [ ] Database transaction handling
- [ ] Message queue integration
- [ ] Cache layer functionality

### ✅ Performance Optimization
- [ ] Database query optimization
- [ ] Redis caching implementation
- [ ] Connection pooling configured
- [ ] Load testing completed (1000 req/s target)

### ✅ Security Hardening
- [ ] Security headers configured
- [ ] OWASP Top 10 compliance
- [ ] Penetration testing passed
- [ ] SSL/TLS properly configured

### ✅ Documentation
- [ ] API documentation (OpenAPI)
- [ ] Integration guide
- [ ] Deployment runbook
- [ ] Troubleshooting guide

### ✅ Deployment Readiness
- [ ] Docker images built and tested
- [ ] Kubernetes manifests ready
- [ ] CI/CD pipeline configured
- [ ] Monitoring dashboards created
- [ ] Alerting rules defined
SPARC + TDD Integration
London School TDD
Focus on interaction testing with mocks.

# Deploy London School TDD approach
npx claude-flow agent spawn tdd-london-swarm \
  --task "Implement payment service with mock interactions"
Example:

// London School - Mock all dependencies
describe('PaymentService', () => {
  let paymentService;
  let mockGateway;
  let mockOrderRepo;
  let mockEventBus;
  
  beforeEach(() => {
    mockGateway = mock(PaymentGateway);
    mockOrderRepo = mock(OrderRepository);
    mockEventBus = mock(EventBus);
    
    paymentService = new PaymentService(
      mockGateway,
      mockOrderRepo,
      mockEventBus
    );
  });
  
  it('should process payment and emit event', async () => {
    // Given
    when(mockGateway.charge(any())).thenResolve({ id: 'pay_123' });
    when(mockOrderRepo.updateStatus(any(), any())).thenResolve();
    
    // When
    await paymentService.processPayment('order_123', 99.99);
    
    // Then
    verify(mockGateway.charge({ amount: 99.99, currency: 'USD' })).once();
    verify(mockOrderRepo.updateStatus('order_123', 'paid')).once();
    verify(mockEventBus.emit('payment.completed', any())).once();
  });
});
Chicago School TDD
Focus on state testing with real implementations.

// Chicago School - Use real implementations where possible
describe('ShoppingCart', () => {
  let cart;
  
  beforeEach(() => {
    cart = new ShoppingCart();
  });
  
  it('should calculate total with tax', () => {
    // Given
    cart.addItem({ id: 1, price: 10.00, quantity: 2 });
    cart.addItem({ id: 2, price: 5.00, quantity: 1 });
    
    // When
    const total = cart.calculateTotal({ taxRate: 0.08 });
    
    // Then
    expect(total).toBe(27.00); // (20 + 5) * 1.08
    expect(cart.getItemCount()).toBe(3);
  });
});
SPARC Workflow Examples
Feature Development Workflow
# 1. Specification
npx claude-flow sparc run specification \
  "Add real-time notifications to chat application"

# 2. Pseudocode
npx claude-flow sparc run pseudocode \
  "Design WebSocket message routing algorithm"

# 3. Architecture
npx claude-flow sparc run architecture \
  "Design scalable WebSocket architecture with Redis"

# 4. TDD Implementation
npx claude-flow sparc tdd \
  "Implement WebSocket notification service"

# 5. Completion
npx claude-flow sparc run completion \
  "Prepare notification service for deployment"
Bug Fix Workflow
# Rapid SPARC cycle for bug fixes
npx claude-flow sparc run refinement \
  "Fix race condition in payment processing" \
  --fast-track \
  --focus testing
Refactoring Workflow
# Architecture-focused SPARC for refactoring
npx claude-flow sparc run architecture \
  "Refactor monolith user service to microservices" \
  --include-migration-plan
Best Practices
1. Start with Clear Specifications
Define acceptance criteria upfront
Include edge cases in specifications
Get stakeholder approval before proceeding
2. Maintain Test Coverage
Aim for >80% code coverage
Include unit, integration, and e2e tests
Test error scenarios thoroughly
3. Iterate in Small Cycles
Keep TDD cycles short (< 15 minutes)
Commit after each green test
Refactor continuously
4. Document Decisions
Record architectural decisions
Document trade-offs
Maintain up-to-date API docs
5. Leverage AI Assistance
Use agents for boilerplate generation
Automate test case creation
Generate documentation from code
Integration with Claude Flow
Memory System Integration
# Store SPARC artifacts in memory
npx claude-flow memory usage \
  --action store \
  --namespace "sparc/auth-service" \
  --key "specifications" \
  --value "$(cat auth-spec.md)"
Swarm Coordination
# Deploy SPARC-specialized swarm
npx claude-flow swarm init sparc-team \
  --agents "specification,pseudocode,architecture,sparc-coder,tester" \
  --topology hierarchical
Workflow Automation
# Automate SPARC pipeline
npx claude-flow workflow create \
  --name "sparc-pipeline" \
  --template "sparc-tdd" \
  --auto-advance \
  --memory-persist
Metrics and Monitoring
SPARC Phase Metrics
const metrics = {
  specification: {
    duration: '2h',
    completeness: 95,
    stakeholderApproval: true
  },
  pseudocode: {
    duration: '1h',
    algorithmComplexity: 'O(n log n)',
    validated: true
  },
  architecture: {
    duration: '3h',
    components: 12,
    interfaces: 8
  },
  refinement: {
    duration: '8h',
    testsPassed: 45,
    coverage: 87
  },
  completion: {
    duration: '2h',
    deploymentReady: true,
    documentationComplete: true
  }
};
Common Patterns
API Development Pattern
# Full API development cycle
npx claude-flow sparc pipeline \
  "Create RESTful API for inventory management" \
  --include "openapi-spec,postman-collection,sdk-generation"
Microservice Pattern
# Microservice development with SPARC
npx claude-flow sparc run architecture \
  "Design order processing microservice" \
  --patterns "event-sourcing,cqrs,saga"
Frontend Component Pattern
# Component development with TDD
npx claude-flow sparc tdd \
  "Create reusable data table component" \
  --framework react \
  --include "storybook,accessibility-tests"