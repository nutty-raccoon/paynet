import Fastify from 'fastify'
import dotenv from 'dotenv';
import NodeCache from 'node-cache';
import Coingecko from '@coingecko/coingecko-typescript';
import { fetchPrice } from './services/priceService';
import { tokenRoutes } from './routes/token/token';
import { priceRoutes } from './routes/price/price';
import { currencyRoutes } from './routes/currency/currency';
import { setEnv } from './setEnv';

export type Token = {
    symbol: string;
    chain: string;
    address: string;
}

dotenv.config();

const env = await setEnv();

export const myCache = new NodeCache();
export const fastify = Fastify({
  logger: true
});

// Set Coingecko SDK
export const client = new Coingecko({
  ...(env.isPro
    ? { proAPIKey: env.apiKey }
    : { demoAPIKey: env.apiKey }), 
  environment: env.isPro ? "pro" : "demo",
})

// Set default tokens
myCache.set("tokens", env.tokens);
myCache.set("currencies", env.currencies);

setInterval(fetchPrice, 50000);

fastify.register(currencyRoutes);
fastify.register(priceRoutes);
fastify.register(tokenRoutes);

// Start server
try {
  await fastify.listen({ port: env.port, host: env.host });
} catch (err) {
  fastify.log.error(err);
  process.exit(1);
}