import Fastify from 'fastify'
import dotenv from 'dotenv';
import NodeCache from 'node-cache';
import Coingecko from '@coingecko/coingecko-typescript';
import { fetchPrice } from './services/priceService';
import { tokenRoutes } from './routes/token/token';
import { priceRoutes } from './routes/price/price';
import { currencyRoutes } from './routes/currency/currency';
import { readEnv } from './readEnv';

export type Token = {
    symbol: string;
    chain: string;
    address: string;
}

// Rotating client wrapper for multiple demo API keys
class RotatingCoingeckoClient {
  private clients: Coingecko[];
  private currentIndex: number = 0;

  constructor(demoKeys: string[]) {
    this.clients = demoKeys.map(key => new Coingecko({
      demoAPIKey: key,
      environment: "demo"
    }));
  }

  private getNextClient(): Coingecko {
    const client = this.clients[this.currentIndex];
    this.currentIndex = (this.currentIndex + 1) % this.clients.length;
    return client;
  }

  // Proxy all method calls to the rotating client
  get simple() {
    return this.getNextClient().simple;
  }

  get coins() {
    return this.getNextClient().coins;
  }
}

dotenv.config();

const env = await readEnv();

export const appCache = new NodeCache();
export const fastify = Fastify({
  logger: true
});

// Set Coingecko SDK with rotation support
export const client = env.isPro
  ? new Coingecko({
      proAPIKey: env.apiKey,
      environment: "pro"
    })
  : env.demoApiKeys && env.demoApiKeys.length > 1
    ? new RotatingCoingeckoClient(env.demoApiKeys) as unknown as Coingecko
    : new Coingecko({
        demoAPIKey: env.apiKey,
        environment: "demo"
      });

// Set default tokens
appCache.set("tokens", env.tokens);
appCache.set("currencies", env.currencies);

setInterval(fetchPrice, 5000);

fastify.get('/health', async (request, reply) => {
  const lastUpdate: string | undefined = appCache.get("last_update");
  const status = lastUpdate ? 'ok' : 'error';
  const code = status === 'ok' ? 200 : 503;
  return reply.code(code).send({ lastUpdate, status });
});

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
