import type { FastifyInstance } from "fastify";
import { myCache } from "../..";

type Price = {
    symbol: string;
    address: string;
    price: {
        currency: string;
        value: number;
    }[]
}

export async function priceRoutes(fastify: FastifyInstance) {
    // GET
    fastify.get('/prices', async function handler (request, reply) {
        const prices: Price | undefined = myCache.get("last_price");
        console.log(prices);
        return reply.code(200).send({ prices });
    })
}