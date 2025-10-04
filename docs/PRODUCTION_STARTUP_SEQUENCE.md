# 🚀 Production Startup Sequence - Cross Chain Orbital Intents AMM

## Executive Summary

This document provides the exact sequence to spin up your Cross Chain Orbital Intents AMM in production. Choose your deployment method based on your needs and infrastructure.

## 🎯 Three Deployment Options

### Option 1: Quick Local Production Test (1-2 hours)
**Best for:** Testing, development, small-scale deployment
**Infrastructure:** Docker Compose on single server
**Cost:** $50-200/month

### Option 2: Cloud Production (1-2 days)
**Best for:** Small to medium scale production
**Infrastructure:** Managed Kubernetes (DigitalOcean/AWS)
**Cost:** $800-2,500/month

### Option 3: Enterprise Production (3-5 days)
**Best for:** Large scale, high availability
**Infrastructure:** Multi-region Kubernetes with advanced monitoring
**Cost:** $2,500-10,000/month

---

## 🔥 OPTION 1: Quick Local Production Test

### Prerequisites
```bash
# Install Docker and Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/1.29.2/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### Step 1: Environment Setup (5 minutes)
```bash
# 1. Clone and navigate to project
git clone https://github.com/your-org/Cross_Chain_Orbital_Intents_AMM.git
cd Cross_Chain_Orbital_Intents_AMM

# 2. Create production environment file
cat > .env.production << 'EOF'
# Database
POSTGRES_PASSWORD=secure_password_change_me

# JWT Security
JWT_SECRET=your-super-secure-jwt-secret-at-least-32-characters-long

# Blockchain RPC URLs (Get from Infura/Alchemy)
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc
OPTIMISM_RPC_URL=https://mainnet.optimism.io
BASE_RPC_URL=https://mainnet.base.org
POLYGON_RPC_URL=https://polygon-rpc.com

# Solver Configuration
SOLVER_PRIVATE_KEY=0x1234...your_solver_private_key_here

# Frontend
CORS_ORIGIN=http://localhost:3000

# Monitoring
GRAFANA_PASSWORD=admin
EOF

# 3. Create necessary directories
mkdir -p monitoring/prometheus monitoring/grafana/dashboards monitoring/grafana/datasources nginx
```

### Step 2: Configuration Files (10 minutes)
```bash
# Create Prometheus configuration
cat > monitoring/prometheus.yml << 'EOF'
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'orbital-api'
    static_configs:
      - targets: ['api:8080']
  
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
  
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
EOF

# Create NGINX configuration
cat > nginx/nginx.conf << 'EOF'
events {
    worker_connections 1024;
}

http {
    upstream api {
        server api:8080;
    }
    
    upstream websocket {
        server api:8081;
    }

    server {
        listen 80;
        
        location /api/ {
            proxy_pass http://api/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
        
        location /ws {
            proxy_pass http://websocket;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
        }
    }
}
EOF

# Create Grafana datasource
cat > monitoring/grafana/datasources/prometheus.yml << 'EOF'
apiVersion: 1
datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
EOF
```

### Step 3: Deploy Smart Contracts (30 minutes)
```bash
# 1. Navigate to contracts directory
cd contracts

# 2. Install dependencies
npm install

# 3. Create deployment configuration
cat > hardhat.config.js << 'EOF'
require("@nomicfoundation/hardhat-toolbox");

const PRIVATE_KEY = process.env.DEPLOYER_PRIVATE_KEY || "0x1234567890123456789012345678901234567890123456789012345678901234";

module.exports = {
  solidity: "0.8.19",
  networks: {
    ethereum: {
      url: process.env.ETHEREUM_RPC_URL,
      accounts: [PRIVATE_KEY]
    },
    arbitrum: {
      url: process.env.ARBITRUM_RPC_URL,
      accounts: [PRIVATE_KEY]
    }
  }
};
EOF

# 4. Deploy contracts (start with testnet)
export DEPLOYER_PRIVATE_KEY=your_deployer_private_key
npx hardhat run scripts/deploy.js --network arbitrum

# 5. Save contract addresses
echo "Copy the deployed contract addresses and update your .env.production file"
```

### Step 4: Build and Start Services (15 minutes)
```bash
# 1. Return to project root
cd ..

# 2. Build all services
docker-compose -f docker-compose.prod.yml build

# 3. Start infrastructure services first
docker-compose -f docker-compose.prod.yml up -d postgres redis

# 4. Wait for database to be ready
sleep 30

# 5. Start application services
docker-compose -f docker-compose.prod.yml up -d api indexer solver

# 6. Start monitoring
docker-compose -f docker-compose.prod.yml up -d prometheus grafana nginx

# 7. Check all services are running
docker-compose -f docker-compose.prod.yml ps
```

### Step 5: Verification (10 minutes)
```bash
# 1. Check API health
curl http://localhost/api/health

# 2. Check WebSocket
# Open browser to http://localhost:3001 (Grafana)
# Login: admin / admin

# 3. Check database
docker-compose -f docker-compose.prod.yml exec postgres psql -U orbital_user -d orbital_amm -c "\dt"

# 4. View logs
docker-compose -f docker-compose.prod.yml logs api
docker-compose -f docker-compose.prod.yml logs solver
```

**✅ Your system is now running locally in production mode!**

---

## 🌐 OPTION 2: Cloud Production (DigitalOcean)

### Prerequisites
```bash
# Install DigitalOcean CLI
snap install doctl

# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Authenticate with DigitalOcean
doctl auth init
```

### Step 1: Infrastructure Setup (30 minutes)
```bash
# 1. Create Kubernetes cluster
doctl kubernetes cluster create orbital-amm-prod \
  --region nyc1 \
  --version 1.28.2-do.0 \
  --count 3 \
  --size s-4vcpu-8gb \
  --auto-upgrade=true

# 2. Configure kubectl
doctl kubernetes cluster kubeconfig save orbital-amm-prod

# 3. Verify cluster
kubectl get nodes
```

### Step 2: Database Setup (20 minutes)
```bash
# 1. Create database cluster
doctl databases create orbital-db \
  --engine postgres \
  --version 15 \
  --region nyc1 \
  --size db-s-2vcpu-4gb \
  --num-nodes 2

# 2. Create Redis cluster
doctl databases create orbital-redis \
  --engine redis \
  --version 7 \
  --region nyc1 \
  --size db-s-1vcpu-1gb

# 3. Get connection details
doctl databases connection orbital-db
doctl databases connection orbital-redis

# 4. Create Kubernetes secrets
kubectl create secret generic orbital-db-secret \
  --from-literal=url="postgresql://username:password@host:port/database?sslmode=require"

kubectl create secret generic orbital-redis-secret \
  --from-literal=url="redis://username:password@host:port"
```

### Step 3: Deploy Applications (30 minutes)
```bash
# 1. Create deployment manifests
cat > k8s-deployment.yml << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbital-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: orbital-api
  template:
    metadata:
      labels:
        app: orbital-api
    spec:
      containers:
      - name: api
        image: your-registry/orbital-api:latest
        ports:
        - containerPort: 8080
        - containerPort: 8081
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbital-db-secret
              key: url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: orbital-redis-secret
              key: url
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
  name: orbital-api-service
spec:
  selector:
    app: orbital-api
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: websocket
    port: 8081
    targetPort: 8081
  type: LoadBalancer
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orbital-solver
spec:
  replicas: 2
  selector:
    matchLabels:
      app: orbital-solver
  template:
    metadata:
      labels:
        app: orbital-solver
    spec:
      containers:
      - name: solver
        image: your-registry/orbital-solver:latest
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: orbital-db-secret
              key: url
        resources:
          requests:
            memory: "8Gi"
            cpu: "4000m"
EOF

# 2. Apply deployments
kubectl apply -f k8s-deployment.yml

# 3. Check deployment status
kubectl get pods
kubectl get services
```

### Step 4: Configure Domain and SSL (20 minutes)
```bash
# 1. Get LoadBalancer IP
kubectl get service orbital-api-service

# 2. Point your domain to the LoadBalancer IP
# Update DNS records: api.yourdomain.com -> LoadBalancer IP

# 3. Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# 4. Create SSL certificate
cat > ssl-certificate.yml << 'EOF'
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
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: orbital-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    secretName: orbital-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: orbital-api-service
            port:
              number: 80
EOF

kubectl apply -f ssl-certificate.yml
```

**✅ Your system is now running in cloud production!**

---

## 🏢 OPTION 3: Enterprise Production

For enterprise deployment, follow Option 2 with these additions:

### Enhanced Features
- Multi-region deployment
- Advanced monitoring with Datadog/New Relic
- Backup and disaster recovery
- Enhanced security with Vault
- CI/CD pipeline with GitLab/Jenkins

### Additional Steps
```bash
# 1. Multi-region setup
doctl kubernetes cluster create orbital-amm-us-east \
  --region nyc1 --count 3 --size s-8vcpu-16gb

doctl kubernetes cluster create orbital-amm-eu-west \
  --region lon1 --count 3 --size s-8vcpu-16gb

# 2. Install monitoring stack
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm install prometheus prometheus-community/kube-prometheus-stack

# 3. Configure backup
kubectl apply -f backup-configuration.yml

# 4. Setup CI/CD pipeline
# Configure GitLab CI or GitHub Actions for automated deployment
```

---

## 📊 Post-Deployment Checklist

### Immediate Verification (First 24 hours)
- [ ] All services are running and healthy
- [ ] API endpoints respond correctly
- [ ] WebSocket connections work
- [ ] Database is accepting connections
- [ ] Smart contracts are deployed and verified
- [ ] Monitoring dashboards show data
- [ ] SSL certificates are working
- [ ] Domain DNS is properly configured

### Production Monitoring Setup
```bash
# 1. Set up alerts
cat > alerts.yml << 'EOF'
groups:
- name: orbital.rules
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status="500"}[5m]) > 0.1
    for: 5m
    annotations:
      summary: High error rate detected
      
  - alert: DatabaseConnectionFailed
    expr: up{job="postgres"} == 0
    for: 1m
    annotations:
      summary: Database connection failed
EOF

# 2. Configure notifications (Slack/PagerDuty)
# Set up webhook URLs for alerts
```

### Security Hardening
```bash
# 1. Network policies
kubectl apply -f network-policies.yml

# 2. Pod security policies
kubectl apply -f pod-security-policies.yml

# 3. RBAC configuration
kubectl apply -f rbac-config.yml

# 4. Regular security scans
# Set up Trivy or similar for container scanning
```

### Performance Optimization
```bash
# 1. Enable horizontal pod autoscaling
kubectl autoscale deployment orbital-api --cpu-percent=70 --min=3 --max=20

# 2. Configure vertical pod autoscaling
kubectl apply -f vpa-config.yml

# 3. Set up resource quotas
kubectl apply -f resource-quotas.yml
```

---

## 🎯 Production Maintenance

### Daily Tasks
- Check system health dashboards
- Review error logs
- Monitor resource usage
- Verify backup completion

### Weekly Tasks
- Update security patches
- Review performance metrics
- Test disaster recovery procedures
- Update documentation

### Monthly Tasks
- Capacity planning review
- Security audit
- Cost optimization review
- Dependency updates

---

## 🆘 Emergency Procedures

### System Down
```bash
# 1. Check overall system status
kubectl get pods --all-namespaces

# 2. Check specific service
kubectl describe pod orbital-api-xxx

# 3. Scale up if needed
kubectl scale deployment orbital-api --replicas=10

# 4. Check logs
kubectl logs -f deployment/orbital-api
```

### Database Issues
```bash
# 1. Check database status
kubectl get pods -l app=postgres

# 2. Force failover if needed
# Follow your database provider's failover procedures

# 3. Scale API pods down during maintenance
kubectl scale deployment orbital-api --replicas=0
```

### High Traffic Load
```bash
# 1. Immediate scaling
kubectl scale deployment orbital-api --replicas=20

# 2. Check resource limits
kubectl top nodes
kubectl top pods

# 3. Enable burst scaling
# Adjust HPA settings for emergency scaling
```

---

## 🎉 Success Metrics

Your production deployment is successful when:

- ✅ **Uptime**: >99.5% availability
- ✅ **Performance**: <200ms API response time (95th percentile)
- ✅ **Throughput**: >1000 requests/second
- ✅ **Success Rate**: >99% transaction success
- ✅ **Security**: No critical vulnerabilities
- ✅ **Monitoring**: All alerts configured and tested

**Congratulations! Your Cross Chain Orbital Intents AMM is now live in production! 🚀**