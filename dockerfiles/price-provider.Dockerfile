# Use Bun official image
# ---- Base image ----
FROM oven/bun:1 AS base

# Set working directory
WORKDIR /app

# Copy dependency files first for caching
COPY infra/price-provider/package.json infra/price-provider/bun.lock ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source code
COPY infra/price-provider/ .

# Expose Fastify port
EXPOSE 3000

# Start server
CMD ["bun", "run", "src/index.ts"]