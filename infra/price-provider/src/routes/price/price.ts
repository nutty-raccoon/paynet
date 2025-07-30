import type { FastifyInstance } from "fastify";
import { getPrices } from "./handler";
import { getPricesSchema } from "./schemas";

export type Price = {
    symbol: string;
    address: string;
    price: {
        currency: string;
        value: number;
    }[]
}

export async function priceRoutes(fastify: FastifyInstance) {
    // GET
    fastify.get('/prices', { schema: getPricesSchema }, getPrices)
}