# Cross Chain Orbital Intents AMM - Production Deployment Guide

## 🚀 Quick Start Production Deployment

This guide provides step-by-step instructions to get your Cross Chain Orbital Intents AMM system running in production.

## Phase 1: Core Infrastructure Setup (Day 1-2)

### Step 1: Cloud Infrastructure Provisioning

**Option A: AWS Deployment**
```bash
# 1. Create VPC and networking
aws ec2 create-vpc --cidr-block 10.0.0.0/16
aws ec2 create-subnet --vpc-id vpc-xxx --cidr-block 10.0.1.0/24

# 2. Set up EKS cluster
aws eks create-cluster --name orbital-amm-cluster \
  --version 1.28 \
  --role-arn arn:aws:iam::account:role/EKSServiceRole

# 3. Configure node groups
aws eks create-nodegroup --cluster-name orbital-amm-cluster \
  --nodegroup-name api-nodes \
  --instance-types t3.large \
  --scaling-config minSize=3,maxSize=10,desiredSize=3
```

**Option B: DigitalOcean Deployment (Recommended for MVP)**
```bash
# 1. Create Kubernetes cluster
doctl kubernetes cluster create orbital-amm \
  --region nyc1 \
  --version 1.28.2-do.0 \
  --count 3 \
  --size s-4vcpu-8gb

# 2. Configure kubectl
doctl kubernetes cluster kubeconfig save orbital-amm
```

**Option C: Local Development (Testing)**
```bash
# Use Docker Compose for local testing
docker-compose -f docker-compose.prod.yml up -d
```

### Step 2: Database Setup

**PostgreSQL Setup:**
```bash
# 1. Install PostgreSQL operator
kubectl apply -f https://raw.githubusercontent.com/postgres-operator/postgres-operator/master/install.yaml

# 2. Create database instance
kubectl apply -f - <<EOF
apiVersion: postgresql.cnpg.io/v1
kind: Cluster
metadata:
  name: orbital-amm-db
spec:
  instances: 3
  postgresql:
    parameters:
      max_connections: "500"
      shared_preload_libraries: "pg_stat_statements"
  bootstrap:
    initdb:
      database: orbital_amm
      owner: orbital_user
      secret:
        name: orbital-db-credentials
EOF

# 3. Wait for database to be ready
kubectl wait cluster/orbital-amm-db --for=condition=Ready --timeout=300s
```

**Redis Setup:**
```bash
# 1. Install Redis operator
kubectl apply -f https://raw.githubusercontent.com/spotahome/redis-operator/master/manifests/operator.yaml

# 2. Create Redis cluster
kubectl apply -f - <<EOF
apiVersion: databases.spotahome.com/v1
kind: RedisFailover
metadata:
  name: orbital-redis
spec:
  sentinel:
    replicas: 3
  redis:
    replicas: 3
    resources:
      requests:
        memory: 4Gi
        cpu: 1000m
EOF
```

### Step 3: Environment Configuration

**Create Production Environment File:**
```bash
# Create production.env
cat > production.env << 'EOF'
# Database Configuration
DATABASE_URL=postgresql://orbital_user:password@orbital-amm-db:5432/orbital_amm
REDIS_URL=redis://orbital-redis:6379

# Blockchain RPC URLs
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
OPTIMISM_RPC_URL=https://mainnet.optimism.io
BASE_RPC_URL=https://mainnet.base.org
POLYGON_RPC_URL=https://polygon-rpc.com

# API Configuration
API_PORT=8080
WEBSOCKET_PORT=8081
JWT_SECRET=your-secure-jwt-secret-32-chars-min
ENCRYPTION_KEY=your-32-byte-encryption-key

# Contract Addresses (Update after deployment)
ETHEREUM_INTENT_ENGINE=0x...
ARBITRUM_INTENT_ENGINE=0x...
OPTIMISM_INTENT_ENGINE=0x...
BASE_INTENT_ENGINE=0x...
POLYGON_INTENT_ENGINE=0x...

# External Services
PROMETHEUS_URL=http://prometheus:9090
GRAFANA_URL=http://grafana:3000

# Security
RATE_LIMIT_REQUESTS=1000
RATE_LIMIT_WINDOW=60
CORS_ORIGIN=https://your-frontend-domain.com
EOF
```

## Phase 2: Smart Contract Deployment (Day 2-3)

### Step 1: Contract Compilation and Deployment

```bash
# 1. Navigate to contracts directory
cd contracts

# 2. Install dependencies
npm install

# 3. Compile contracts
npx hardhat compile

# 4. Deploy to mainnet (start with one chain)
npx hardhat run scripts/deploy.js --network ethereum

# 5. Deploy to L2s
npx hardhat run scripts/deploy.js --network arbitrum
npx hardhat run scripts/deploy.js --network optimism
npx hardhat run scripts/deploy.js --network base
npx hardhat run scripts/deploy.js --network polygon

# 6. Verify contracts
npx hardhat verify --network ethereum CONTRACT_ADDRESS
```

### Step 2: Contract Configuration

```bash
# Update contract addresses in production.env
# Copy the deployed addresses from the deployment logs
```

## Phase 3: Backend Services Deployment (Day 3-4)

### Step 1: Build and Deploy API Service

```bash
# 1. Build Rust API service
cd backend/api
cargo build --release

# 2. Create Docker image
docker build -t orbital-amm-api:latest .

# 3. Deploy to Kubernetes
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbital-amm-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: orbital-amm-api
  template:
    metadata:
      labels:
        app: orbital-amm-api
    spec:
      containers:
      - name: api
        image: orbital-amm-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbital-env
              key: DATABASE_URL
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
---
apiVersion: v1
kind: Service
metadata:
  name: orbital-amm-api-service
spec:
  selector:
    app: orbital-amm-api
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
EOF
```

### Step 2: Deploy Indexer Service

```bash
# 1. Build indexer service
cd backend/indexer
cargo build --release

# 2. Deploy indexer
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbital-amm-indexer
spec:
  replicas: 2
  selector:
    matchLabels:
      app: orbital-amm-indexer
  template:
    metadata:
      labels:
        app: orbital-amm-indexer
    spec:
      containers:
      - name: indexer
        image: orbital-amm-indexer:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbital-env
              key: DATABASE_URL
        resources:
          requests:
            memory: "6Gi"
            cpu: "2000m"
EOF
```

### Step 3: Deploy Solver Service

```bash
# 1. Build solver service
cd core/solver
cargo build --release

# 2. Deploy solver
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbital-amm-solver
spec:
  replicas: 3
  selector:
    matchLabels:
      app: orbital-amm-solver
  template:
    metadata:
      labels:
        app: orbital-amm-solver
    spec:
      containers:
      - name: solver
        image: orbital-amm-solver:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbital-env
              key: DATABASE_URL
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
EOF
```

## Phase 4: Frontend Deployment (Day 4)

### Step 1: Build and Deploy Frontend

```bash
# 1. Navigate to frontend
cd frontend

# 2. Install dependencies
npm install

# 3. Build for production
npm run build

# 4. Deploy to Netlify (Recommended)
# Connect your GitHub repo to Netlify
# Set build command: npm run build
# Set publish directory: out

# Or deploy to Vercel
npx vercel --prod

# Or use traditional hosting
# Upload the 'out' directory to your CDN/web server
```

### Step 2: Configure Domain and SSL

```bash
# 1. Point your domain to the frontend deployment
# 2. Configure SSL certificate (automatic with Netlify/Vercel)
# 3. Update CORS_ORIGIN in production.env with your domain
```

## Phase 5: Monitoring Setup (Day 5)

### Step 1: Deploy Prometheus

```bash
# 1. Install Prometheus operator
kubectl apply -f https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/bundle.yaml

# 2. Create Prometheus instance
kubectl apply -f - <<EOF
apiVersion: monitoring.coreos.com/v1
kind: Prometheus
metadata:
  name: orbital-prometheus
spec:
  serviceAccountName: prometheus
  serviceMonitorSelector:
    matchLabels:
      app: orbital-amm
  resources:
    requests:
      memory: 400Mi
  retention: 30d
EOF
```

### Step 2: Deploy Grafana

```bash
# Deploy Grafana with pre-configured dashboards
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: grafana
spec:
  replicas: 1
  selector:
    matchLabels:
      app: grafana
  template:
    metadata:
      labels:
        app: grafana
    spec:
      containers:
      - name: grafana
        image: grafana/grafana:latest
        ports:
        - containerPort: 3000
        env:
        - name: GF_SECURITY_ADMIN_PASSWORD
          value: "admin"
EOF
```

## Phase 6: Security Hardening (Day 6)

### Step 1: SSL/TLS Configuration

```bash
# 1. Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# 2. Create ClusterIssuer for Let's Encrypt
kubectl apply -f - <<EOF
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@domain.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
EOF
```

### Step 2: Configure WAF and Rate Limiting

```bash
# 1. Install NGINX Ingress with rate limiting
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.1/deploy/static/provider/cloud/deploy.yaml

# 2. Create ingress with rate limiting
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: orbital-amm-ingress
  annotations:
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    secretName: orbital-amm-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: orbital-amm-api-service
            port:
              number: 80
EOF
```

## Phase 7: Go-Live Checklist

### Pre-Launch Verification

```bash
# 1. Health Check All Services
kubectl get pods
kubectl get services
kubectl get ingress

# 2. Test API Endpoints
curl https://api.yourdomain.com/health
curl https://api.yourdomain.com/stats

# 3. Test WebSocket Connection
# Use your frontend to test real-time connections

# 4. Verify Database Connectivity
kubectl exec -it deployment/orbital-amm-api -- /bin/bash
# Test database connection inside pod

# 5. Test Smart Contract Interactions
# Submit a test intent through the frontend

# 6. Monitor Logs
kubectl logs -f deployment/orbital-amm-api
kubectl logs -f deployment/orbital-amm-indexer
kubectl logs -f deployment/orbital-amm-solver
```

### Performance Verification

```bash
# 1. Load Test API
# Install Artillery
npm install -g artillery

# Create load test
cat > loadtest.yml << 'EOF'
config:
  target: https://api.yourdomain.com
  phases:
    - duration: 60
      arrivalRate: 10
scenarios:
  - name: API Health Check
    requests:
      - get:
          url: "/health"
EOF

artillery run loadtest.yml

# 2. Monitor Resource Usage
kubectl top nodes
kubectl top pods

# 3. Check Database Performance
# Connect to database and run EXPLAIN ANALYZE on key queries
```

## Production Maintenance

### Daily Operations

```bash
# 1. Check system health
kubectl get pods --all-namespaces
kubectl get events --sort-by=.metadata.creationTimestamp

# 2. Monitor logs for errors
kubectl logs -f deployment/orbital-amm-api --tail=100
kubectl logs -f deployment/orbital-amm-solver --tail=100

# 3. Check database performance
# Monitor slow queries and connection counts

# 4. Verify external service connectivity
# Check RPC endpoint health and rate limits
```

### Weekly Operations

```bash
# 1. Database backup verification
# Ensure backups are completing successfully

# 2. Security scan
# Run vulnerability scans on container images

# 3. Performance review
# Analyze Grafana dashboards for trends

# 4. Update dependencies
# Check for security updates in base images
```

### Emergency Procedures

```bash
# 1. Scale up for high traffic
kubectl scale deployment orbital-amm-api --replicas=10

# 2. Emergency maintenance mode
# Update ingress to serve maintenance page

# 3. Database failover
# Promote read replica to primary if needed

# 4. Rollback deployment
kubectl rollout undo deployment/orbital-amm-api
```

## Cost Optimization

### Infrastructure Sizing Guidelines

**Minimal Production Setup (Small Scale):**
- Kubernetes: 3 nodes (4 vCPU, 8GB RAM each)
- Database: 1 primary + 1 replica (8 vCPU, 32GB RAM)
- Redis: 3-node cluster (2 vCPU, 4GB RAM each)
- **Estimated Cost: $800-1,200/month**

**Standard Production Setup (Medium Scale):**
- Kubernetes: 6 nodes (8 vCPU, 16GB RAM each)
- Database: 1 primary + 2 replicas (16 vCPU, 64GB RAM)
- Redis: 6-node cluster (4 vCPU, 8GB RAM each)
- **Estimated Cost: $2,500-4,000/month**

**Enterprise Setup (Large Scale):**
- Kubernetes: 12 nodes (16 vCPU, 32GB RAM each)
- Database: 1 primary + 3 replicas (32 vCPU, 128GB RAM)
- Redis: 9-node cluster (8 vCPU, 16GB RAM each)
- **Estimated Cost: $8,000-15,000/month**

## Troubleshooting Guide

### Common Issues

**API Service Won't Start:**
```bash
# Check pod logs
kubectl logs deployment/orbital-amm-api

# Check environment variables
kubectl describe pod orbital-amm-api-xxx

# Verify database connectivity
kubectl exec -it deployment/orbital-amm-api -- ping orbital-amm-db
```

**Database Connection Issues:**
```bash
# Check database status
kubectl get clusters

# Check secrets
kubectl get secret orbital-db-credentials -o yaml

# Test connection manually
kubectl run -it --rm debug --image=postgres:15 --restart=Never -- psql $DATABASE_URL
```

**WebSocket Connection Failures:**
```bash
# Check if WebSocket service is running
kubectl get svc orbital-amm-websocket

# Check port configuration
kubectl describe svc orbital-amm-websocket

# Test WebSocket connection
# Use online WebSocket test tool to connect to wss://api.yourdomain.com/ws
```

This deployment guide provides a complete production setup for your Cross Chain Orbital Intents AMM system. Start with the minimal production setup and scale up as your user base grows.