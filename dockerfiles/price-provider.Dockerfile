FROM oven/bun:1 AS base

RUN apt-get update && apt-get install -y curl

WORKDIR /app

COPY infra/price-provider/package.json infra/price-provider/bun.lock ./

RUN bun install --frozen-lockfile

COPY infra/price-provider/ .

ENV PORT=80
EXPOSE ${PORT}

HEALTHCHECK --interval=15s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://0.0.0.0:${PORT}/health || exit 1

CMD ["bun", "run", "src/index.ts"]
