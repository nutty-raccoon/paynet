FROM oven/bun:1.1.10 as builder

WORKDIR /app

RUN apt-get update && apt-get install -y unzip wget && rm -rf /var/lib/apt/lists/*

# Download the repository (main branch) using wget
RUN wget -q https://github.com/cartridge-gg/explorer/archive/refs/heads/main.zip \
    && unzip -q main.zip \
    && rm main.zip

WORKDIR /app/explorer-main

# Install dependencies
RUN bun install

#------------

FROM oven/bun:latest

COPY --from=builder /app/explorer-main /app

WORKDIR /app

# Run the application in dev mode
CMD ["bun", "run", "dev", "--host"]
