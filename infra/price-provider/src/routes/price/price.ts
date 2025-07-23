import type { FastifyInstance } from "fastify";
import { myCache } from "../..";

export async function priceRoutes(fastify: FastifyInstance) {
    // GET
    fastify.get('/prices', async function handler (request, reply) {
        const price = myCache.get("last_price");
        return reply.code(200).send({ price });
    })
}