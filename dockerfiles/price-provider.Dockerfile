FROM oven/bun:1 AS base

RUN apt-get update && apt-get install -y curl

WORKDIR /app

COPY infra/price-provider/package.json infra/price-provider/bun.lock ./

RUN bun install --frozen-lockfile

COPY infra/price-provider/ .

EXPOSE 3007

HEALTHCHECK --interval=3s --timeout=10s --start-period=20s --retries=5 \
    CMD curl -f http://0.0.0.0:3000/health || exit 1

CMD ["bun", "run", "src/index.ts"]