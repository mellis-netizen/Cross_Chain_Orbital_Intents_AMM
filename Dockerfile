# Multi-stage build for production-optimized container
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY core/bridge/Cargo.toml ./core/bridge/
COPY core/engine/Cargo.toml ./core/engine/
COPY core/solver/Cargo.toml ./core/solver/
COPY orbital-math/Cargo.toml ./orbital-math/
COPY contracts/intents/Cargo.toml ./contracts/intents/

# Copy source code
COPY . .

# Build the application in release mode
RUN cargo build --release --bin api

# Final stage - minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /app orbital

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api ./api

# Copy configuration files
COPY --chown=orbital:orbital config/ ./config/
COPY --chown=orbital:orbital scripts/health-check.sh ./health-check.sh

# Make scripts executable
RUN chmod +x ./health-check.sh

# Switch to non-root user
USER orbital

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ./health-check.sh

# Set environment variables
ENV RUST_LOG=info
ENV SERVER_ADDRESS=0.0.0.0:8080

# Run the application
CMD ["./api"]