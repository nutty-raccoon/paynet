import type { FastifyInstance } from "fastify";
import { appCache } from "../..";
// import { addCurrency, delCurrency } from "./handler";
// import { addCurrencySchema, delCurrencySchema } from "./schemas";

export async function currencyRoutes(fastify: FastifyInstance) {
    // GET
    fastify.get('/currencies', async function handler (request, reply) {
        const currencies = appCache.get("currencies");
        return reply.code(200).send({ currencies });
    });

    // POST
    // fastify.post('/del/currency', { schema: delCurrencySchema }, delCurrency);
    // fastify.post('/add/currency', { schema: addCurrencySchema }, addCurrency);
}