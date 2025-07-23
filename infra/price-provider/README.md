# Price Provider API

A REST API built with [Fastify](https://fastify.dev/) and [Bun](https://bun.sh/), providing prices for tokens and managing currencies & tokens dynamically.

---

## Features

- **Token Management**
  - Add or remove blockchain tokens by chain and address.
- **Currency Management**
  - Add or remove fiat/crypto currencies used as price reference.
- **Price Fetching**
  - Periodically fetches token prices (every 5s) from [CoinGecko](https://www.coingecko.com/).
- **Simple & Fast**
  - Built on Fastify with in-memory caching.

---

## Prerequisites

- **Bun** ≥ 1.0 (if running without Docker)
- **Docker** (if running with Docker)
- A **CoinGecko API key** (demo or pro)

---

## Environment Variables

Create an `.env` file (if not using Docker):

```yaml
COIN_GECKO_API_KEY=your_api_key
```

---

## Running Locally (without Docker)

1. Install dependencies:
   ```bash
   bun install
   Run the server:
   ```
2. Run the server:
   ```bash
   bun run src/index.ts
   Server runs by default on http://localhost:3000
   ```
3. Server runs by default on http://localhost:3000

## Running with Docker

All Dockerfiles for this repository are stored in `../../dockerfiles`.

1. Build the Docker Image
   From `infra/price-provider` directory:

   ```bash
   docker build -t price-provider .
   ```

2. Run the Docker Container
   ```bash
    docker run -p 3000:3000 \
    --env COIN_GECKO_API_KEY=your_api_key \
    price-provider
   ```
   Server runs at http://localhost:3000

## API Routes

### GET `/tokens`

- Description: List all tokens currently tracked.
- Response:

```json
{
  "tokens": [
    { "symbol": "eth", "chain": "ethereum", "address": "0xc02aaa39..." },
    { "symbol": "strk", "chain": "ethereum", "address": "0xca14007..." }
  ]
}
```

### POST `/add/token`

- Body:

```json
{ "address": "0x...", "chain": "ethereum" }
```

- Response: `201 Created` on success, `409 Conflict` if already added, `404 Not Found` if token doesn't exist.

### POST `/del/token`

- Body:

```json
{ "symbol": "eth", "address": "0x...", "chain": "ethereum" }
```

- Response: `201 Created` on success, `404 Not Found` if token doesn't exist.

### GET `/currency`

- Description: List all currencies used for price comparison.

- Response:

```json
{ "currencies": ["usd"] }
```

### POST /add/currency

- Body:

```json
{ "currency": "usd" }
```

- Response: `201 Created` on success, `409 Conflict` if already added, `404 Not Found` if currency not supported by CoinGecko.

### POST `/del/currency`

- Body:

```json
{ "currency": "usd" }
```

- Response: `201 Created` on success, `404 Not Found` if currency doesn't exist.

### GET `/prices`

- Description: Retrieve last cached prices.

- Response:

```json
{
  "price": [
    { "symbol": "eth", "address": "0xc02a...", "price": { "usd": 3100 } },
    { "symbol": "strk", "address": "0xca14...", "price": { "usd": 0.8 } }
  ]
}
```
