import type { FastifyInstance } from 'fastify';
import { myCache, tokens } from '../..';
import { addTokenSchema, delTokenSchema } from './schemas';
import { addToken, delToken } from './handler';

export async function tokenRoutes(fastify: FastifyInstance) {
    // GET
    fastify.get('/tokens', async function handler (request, reply) {
        const tokens = myCache.get("tokens");
        return reply.code(200).send({ tokens });
    });

    // POST
    fastify.post('/del/token', { schema: delTokenSchema }, delToken);
    fastify.post('/add/token', { schema: addTokenSchema }, addToken);
}