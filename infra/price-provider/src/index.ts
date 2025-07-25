import Fastify from 'fastify'
import dotenv from 'dotenv';
import NodeCache from 'node-cache';
import Coingecko from '@coingecko/coingecko-typescript';
import { fetchPrice } from './services/priceService';
import { tokenRoutes } from './routes/token/token';
import { priceRoutes } from './routes/price/price';
import { currencyRoutes } from './routes/currency/currency';

dotenv.config();

export const myCache = new NodeCache();
export const fastify = Fastify({
  logger: true
});

if (!process.env.COIN_PRO_GECKO_API_KEY && !process.env.COIN_DEMO_GECKO_API_KEY) {
  throw new Error("Missing env var: COIN_DEMO_GECKO_API_KEY or COIN_PRO_GECKO_API_KEY");
}
const isPro = !!process.env.COIN_PRO_GECKO_API_KEY
let apiKey = isPro ? process.env.COIN_PRO_GECKO_API_KEY! : process.env.COIN_DEMO_GECKO_API_KEY!;
const PORT = process.env.PORT ? parseInt(process.env.PORT) : 3000;

// Set Coingecko SDK
export const client = new Coingecko({
//   proAPIKey: apiKey,
  ...(isPro
    ? { proAPIKey: apiKey }
    : { demoAPIKey: apiKey }), // Optional, for Demo API access
  environment: isPro ? "pro" : "demo", // 'pro' to initialize the client with Pro API access
})

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

setInterval(fetchPrice, 5000);

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