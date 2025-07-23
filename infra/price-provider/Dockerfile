# Use Bun official image
FROM oven/bun:1

# Set working directory
WORKDIR /app

# Copy dependency files first for caching
COPY package.json bun.lock ./

# Install dependencies using Bun
RUN bun install --frozen-lockfile

# Copy the source code
COPY . .

# Expose the port (default is 3000 in index.ts)
EXPOSE 3000

# Start the server
CMD ["bun", "run", "src/index.ts"]