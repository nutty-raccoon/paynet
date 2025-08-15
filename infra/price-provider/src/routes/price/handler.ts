import type { FastifyReply, FastifyRequest } from "fastify";
import type { Price } from "./price";
import { appCache } from "../..";

export async function getPrices(request: FastifyRequest, reply: FastifyReply) {
    const { currencies: curParam, assets: assetsParam } = request.query as { currencies?: string; assets?: string };
            const prices: Price[] = appCache.get<Price[]>("last_price") ?? [];
    
            let filtered = prices;
            if (curParam) {
            const wanted = curParam.split(",").map(s => s.trim().toLowerCase());
            filtered = filtered
                .map(p => ({
                ...p,
                price: p.price.filter(pr => wanted.includes(pr.currency.toLowerCase())),
                }))
                .filter(p => p.price.length > 0);
            }
    
            if (assetsParam) {
                const wantedTokens = assetsParam.split(",").map(s => s.trim().toLowerCase());
                filtered = filtered.filter(p =>
                    wantedTokens.includes(p.symbol.toLowerCase())
                    || wantedTokens.includes(p.address.toLowerCase())
                );
            }
            return reply.code(200).send({ prices: filtered });
}