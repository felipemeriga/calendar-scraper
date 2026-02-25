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
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/calendar-scraper /app/calendar-scraper

# Copy example configuration files
COPY calendars.toml.example /app/calendars.toml.example
COPY .env.example /app/.env.example

# Create a volume for configuration files
VOLUME ["/app/config"]

# Expose the port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV CALENDARS_CONFIG=/app/config/calendars.toml

# Run the binary
CMD ["/app/calendar-scraper"]
