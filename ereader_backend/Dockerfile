# Build stage
FROM rust:1.92-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf

# Create a new empty shell project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies (cache layer)
RUN cargo build --release --bin api_server 2>/dev/null || true

# Copy source
COPY . .

# Build application
RUN cargo build --release --bin api_server

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create app user
RUN addgroup -g 1000 ereader && \
    adduser -u 1000 -G ereader -s /bin/sh -D ereader

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/api_server /app/api_server

# Copy config files
COPY config /app/config

# Create data directories
RUN mkdir -p /app/data/files /app/data/covers /app/data/temp && \
    chown -R ereader:ereader /app

# Switch to non-root user
USER ereader

# Environment variables
ENV RUST_LOG=info
ENV EREADER__SERVER__HOST=0.0.0.0
ENV EREADER__SERVER__PORT=3000

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/v1/health || exit 1

# Run the binary
CMD ["/app/api_server"]
