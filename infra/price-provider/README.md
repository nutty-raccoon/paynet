# Price Provider API

A REST API built with [Fastify](https://fastify.dev/) and [Bun](https://bun.sh/), providing prices for tokens and managing currencies & tokens dynamically via environment variables.

---

## Features

- **Price Fetching**

  - Periodically fetches token prices (every 5s) from [CoinGecko](https://www.coingecko.com/).

- **Dynamic Configuration**

  - Configure tracked currencies and tokens entirely via environment variables—no need for POST endpoints.

- **Simple & Fast**

  - Built on Fastify with in-memory caching.

---

## Prerequisites

- **Bun** ≥ 1.0 (if running without Docker)
- **Docker** (if running with Docker)
- A **CoinGecko API key** (demo or pro)

---

## Environment Variables

Create an `.env` file (or pass via Docker) with the following:

```env
# CoinGecko API keys
COIN_DEMO_GECKO_API_KEY=your_demo_api_key
COIN_PRO_GECK_API_KEY=your_pro_api_key

# Server configuration (optional; defaults shown)
PORT=3000               # Port the server listens on (default: 3000)
HOST=0.0.0.0            # Host interface (default: 0.0.0.0)

# Price configuration
# List of currency codes to fetch (must be a JSON array of strings)
CURRENCIES=["usd","eur"]

# List of tokens to track (JSON array of objects)
TOKENS=[
  {
    "symbol": "eth",
    "chain": "ethereum",
    "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
  },
  {
    "symbol": "strk",
    "chain": "ethereum",
    "address": "0xca14007eff0db1f8135f4c25b34de49ab0d42766"
  }
]
```

- **CURRENCIES**: JSON array of fiat or crypto currency codes. Examples: `["usd"]`, `["usd","eur"]`.
- **TOKENS**: JSON array where each token has `symbol`, `chain`, and `address` fields.
- **PORT** & **HOST**: Override the default listening port and host interface.

---

## Running Locally (without Docker)

1. Install dependencies:

   ```bash
   bun install
   ```

2. Ensure your `.env` is configured, then start the server:

   ```bash
   bun run src/index.ts
   ```

3. The server will run at `http://${HOST}:${PORT}` (e.g. `http://0.0.0.0:3007`).

---

## Running with Docker

All Dockerfiles for this repository are stored in `./dockerfiles`.

1. Build the Docker image:

   ```bash
   docker build -f ./dockerfiles/price-provider.Dockerfile -t price-provider .
   ```

2. Run the Docker container (example):

   ```bash
   docker run -p 3007:3007 \
     --env-file ./infra/price-provider/.env.local\
     price-provider
   ```

3. The service will be available at `http://${HOST}:${PORT}`.

---

## API Routes

### GET `/tokens`

- **Description**: List all tokens currently tracked (as configured in `TOKENS`).
- **Response**:

  ```json
  {
    "tokens": [
      {
        "symbol": "eth",
        "chain": "ethereum",
        "address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
      },
      {
        "symbol": "strk",
        "chain": "ethereum",
        "address": "0xca14007eff0db1f8135f4c25b34de49ab0d42766"
      }
    ]
  }
  ```

### GET `/currency`

- **Description**: List all fiat currencies used for price comparison (as configured in `CURRENCIES`).
- **Response**:

  ```json
  { "currencies": ["usd", "eur"] }
  ```

### GET `/prices`

- **Description**: Retrieve the most recently cached token prices in the configured currencies.
- **Response**:

  ```json
  {
    "price": [
      {
        "symbol": "eth",
        "address": "0xc02a...",
        "price": { "usd": 3100, "eur": 2800 }
      },
      {
        "symbol": "strk",
        "address": "0xca14...",
        "price": { "usd": 0.8, "eur": 0.72 }
      }
    ]
  }
  ```
