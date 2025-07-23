import Fastify from 'fastify'
import dotenv from 'dotenv';
import NodeCache from 'node-cache';
import Coingecko from '@coingecko/coingecko-typescript';
import { fetch_price } from './services/priceService';
import { tokenRoutes } from './routes/token/token';
import { priceRoutes } from './routes/price/price';
import { currencyRoutes } from './routes/currency/currency';

dotenv.config();

export const myCache = new NodeCache();
export const fastify = Fastify({
  logger: true
});

if (!process.env.COIN_GECKO_API_KEY) {
  throw new Error("Missing env var: COIN_GECKO_API_KEY");
}
const api_key = process.env.COIN_GECKO_API_KEY;
const PORT = process.env.PORT ? parseInt(process.env.PORT) : 3000;

// Set Coingecko SDK
export const client = new Coingecko({
//   proAPIKey: api_key,
  demoAPIKey: api_key, // Optional, for Demo API access
  environment: 'demo', // 'pro' to initialize the client with Pro API access
});

export type Token = {
    symbol: string;
    chain: string;
    address: string;
}

// Set default tokens
export let tokens: Token[] = [{symbol: "eth", chain: "ethereum", address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"}, {symbol: "strk", chain: "ethereum", address: "0xca14007eff0db1f8135f4c25b34de49ab0d42766"}];
myCache.set("tokens", tokens);

// Set default currencies
export let currencies: string[] = ["usd"];
myCache.set("currencies", currencies);

setInterval(fetch_price, 5000);

fastify.register(currencyRoutes);
fastify.register(priceRoutes);
fastify.register(tokenRoutes);

// Start server
try {
  await fastify.listen({ port: PORT, host: '0.0.0.0' });
} catch (err) {
  fastify.log.error(err);
  process.exit(1);
}