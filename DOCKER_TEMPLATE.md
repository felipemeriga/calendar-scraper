# Docker Template for Rust Microservices

Copy these files to your new microservice project and customize as needed.

## Files to Copy

1. `Dockerfile`
2. `.dockerignore`
3. `docker-compose.yml`
4. `.env.example`

---

## Dockerfile Template

```dockerfile
# Build stage
FROM rust:slim as builder

# Install required dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
# Touch main.rs to force rebuild of the actual code
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests and timezone data
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder
# TODO: Replace YOUR_APP_NAME with your binary name
COPY --from=builder /app/target/release/YOUR_APP_NAME /app/YOUR_APP_NAME

# Copy example configuration files (optional)
COPY .env.example /app/.env.example

# Create a volume for configuration files (optional)
VOLUME ["/app/config"]

# Expose the port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the binary
# TODO: Replace YOUR_APP_NAME with your binary name
CMD ["/app/YOUR_APP_NAME"]
```

**Customization checklist:**
- [ ] Replace `YOUR_APP_NAME` with your actual binary name (from Cargo.toml)
- [ ] Adjust `EXPOSE` port if using different port
- [ ] Add any additional environment variables needed
- [ ] Add volumes for configuration files if needed
- [ ] Adjust dependencies in build stage if needed

---

## .dockerignore Template

```
# Git
.git
.gitignore
.github/

# Rust build artifacts
target/
**/*.rs.bk
*.pdb

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment and configuration files (mount these as volumes)
.env
config.toml
secrets.toml

# Documentation (except README)
*.md
!README.md

# Test files
tests/

# CI/CD
.github/

# Other
*.log
tmp/
.cache/
```

**Customization checklist:**
- [ ] Add any project-specific files to ignore
- [ ] Add configuration files that should be mounted as volumes
- [ ] Add any secret files that should never be in the image

---

## docker-compose.yml Template

```yaml
version: '3.8'

services:
  # TODO: Replace 'your-service' with your service name
  your-service:
    image: your-service:latest
    container_name: your-service
    restart: unless-stopped
    ports:
      - "8080:8080"  # TODO: Adjust port mapping if needed
    environment:
      # TODO: Add your environment variables
      - API_TOKEN=${API_TOKEN:-dev-token}
      - HOST=0.0.0.0
      - PORT=8080
      - RUST_LOG=${RUST_LOG:-info}
      # Add more as needed:
      # - DATABASE_URL=${DATABASE_URL}
      # - REDIS_URL=${REDIS_URL}
    volumes:
      # TODO: Add volume mounts for configuration files
      # - ./config.toml:/app/config/config.toml:ro
      # - ./data:/app/data
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - app-network
    # Uncomment if you have a database service
    # depends_on:
    #   - postgres

  # Uncomment and customize if you need a database
  # postgres:
  #   image: postgres:16-alpine
  #   container_name: your-service-db
  #   restart: unless-stopped
  #   environment:
  #     - POSTGRES_USER=${DB_USER:-postgres}
  #     - POSTGRES_PASSWORD=${DB_PASSWORD:-postgres}
  #     - POSTGRES_DB=${DB_NAME:-your_service}
  #   volumes:
  #     - postgres-data:/var/lib/postgresql/data
  #   networks:
  #     - app-network

  # Uncomment if you need Redis
  # redis:
  #   image: redis:7-alpine
  #   container_name: your-service-redis
  #   restart: unless-stopped
  #   networks:
  #     - app-network

networks:
  app-network:
    driver: bridge

# volumes:
#   postgres-data:
```

**Customization checklist:**
- [ ] Replace `your-service` with your service name
- [ ] Adjust port mappings
- [ ] Add required environment variables
- [ ] Configure volume mounts for config files
- [ ] Enable database/redis if needed
- [ ] Adjust health check endpoint if different

---

## .env.example Template

```bash
# API Configuration
API_TOKEN=dev-token-change-in-production

# Server Configuration
HOST=0.0.0.0
PORT=8080

# Logging
RUST_LOG=info

# Add your service-specific variables below:

# Database (if needed)
# DATABASE_URL=postgresql://user:password@localhost:5432/dbname

# Redis (if needed)
# REDIS_URL=redis://localhost:6379

# External APIs (if needed)
# EXTERNAL_API_KEY=your-api-key
# EXTERNAL_API_URL=https://api.example.com

# OAuth (if needed)
# OAUTH_CLIENT_ID=your-client-id
# OAUTH_CLIENT_SECRET=your-client-secret
# OAUTH_REDIRECT_URL=http://localhost:8080/auth/callback
```

**Customization checklist:**
- [ ] Add all environment variables your service needs
- [ ] Document each variable with comments
- [ ] Use safe default values for development
- [ ] Mark production-only variables clearly

---

## Building and Running

### Local Development
```bash
# Copy environment template
cp .env.example .env

# Edit .env with your values
nano .env

# Build the image
docker build -t your-service:latest .

# Run with docker-compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Production Deployment

```bash
# Build the image
docker build -t your-registry/your-service:v1.0.0 .

# Tag as latest
docker tag your-service:v1.0.0 your-registry/your-service:latest

# Login to registry
docker login your-registry

# Push
docker push your-registry/your-service:v1.0.0
docker push your-registry/your-service:latest

# On production server
docker pull your-registry/your-service:latest
docker-compose up -d
```

### Docker Hub Deployment

```bash
# Build
docker build -t your-dockerhub-user/your-service:latest .

# Login
docker login

# Push
docker push your-dockerhub-user/your-service:latest

# On server
docker pull your-dockerhub-user/your-service:latest
docker run -d \
  --name your-service \
  --restart unless-stopped \
  -p 8080:8080 \
  -e API_TOKEN=your-token \
  -v /path/to/config:/app/config:ro \
  your-dockerhub-user/your-service:latest
```

---

## Multi-Stage Build Optimization

The Dockerfile uses multi-stage build to:

1. **Build stage** - Compiles Rust code
   - Uses full Rust image with build tools
   - Caches dependencies separately from source code
   - Results in faster rebuilds when only source changes

2. **Runtime stage** - Minimal runtime image
   - Uses debian:bookworm-slim (smaller base)
   - Only copies the compiled binary
   - Includes only runtime dependencies (ca-certificates, tzdata)
   - Results in ~120MB final image (vs ~1.5GB with full Rust image)

### Build Performance Tips

```bash
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker build -t your-service:latest .

# Build with cache from registry
docker build \
  --cache-from your-registry/your-service:latest \
  -t your-service:latest .

# Multi-platform build (for ARM and x86)
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t your-service:latest .
```

---

## Health Checks

Your service should implement a `/health` endpoint:

```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "your-service-name",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

This is used by:
- Docker health checks
- Load balancers
- Kubernetes liveness/readiness probes
- Monitoring systems

---

## Security Best Practices

### 1. Don't Run as Root

Add to Dockerfile before CMD:
```dockerfile
RUN useradd -m -u 1000 appuser
USER appuser
```

### 2. Use Specific Image Tags

```dockerfile
# Bad - version can change
FROM rust:slim

# Good - pinned version
FROM rust:1.75-slim

# Better - with digest
FROM rust:1.75-slim@sha256:abc123...
```

### 3. Scan Images

```bash
# Use Docker Scout
docker scout cves your-service:latest

# Or Trivy
trivy image your-service:latest
```

### 4. Minimize Attack Surface

- Only install necessary packages
- Remove package manager cache
- Use `.dockerignore` to avoid leaking sensitive files
- Don't copy `.git` directory

---

## Troubleshooting

### Build Fails

```bash
# Clean build cache
docker builder prune

# Build without cache
docker build --no-cache -t your-service:latest .

# Check disk space
docker system df
```

### Container Won't Start

```bash
# View logs
docker logs your-service

# Interactive shell in container
docker run -it --entrypoint /bin/bash your-service:latest

# Check if binary exists
docker run -it your-service:latest ls -la /app/
```

### Can't Connect to Service

```bash
# Check if port is exposed
docker port your-service

# Check if service is listening on 0.0.0.0 (not 127.0.0.1)
docker exec your-service netstat -tuln

# Check firewall rules on host
```

---

## Next Steps

1. Copy this template to your new project
2. Customize all TODO items
3. Test build locally: `docker build -t test:latest .`
4. Test run locally: `docker run -p 8080:8080 test:latest`
5. Push to registry when ready

For more details, see `DEVELOPMENT_GUIDELINES.md`
