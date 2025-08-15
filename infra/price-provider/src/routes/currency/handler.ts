import type { FastifyReply, FastifyRequest } from "fastify";
import { client, appCache } from "../..";

export async function addCurrency(request: FastifyRequest, reply: FastifyReply) {
    const { currency } = request.body as { currency: string };
    
    const currencies: string[] | undefined = appCache.get("currencies");
    if (!currencies) {
        throw new Error("Cache doesn't set.");
    }

    const exist = currencies.includes(currency);
    if (exist) {
        return reply.code(409).send({ error: "Currency already added." });
    }

    const res = await client.simple.supportedVsCurrencies.get();
    const newCurrency = res.includes(currency);
    if (!newCurrency) {
        return reply.code(404).send({ error: "The currency doesn't exist on CoinGecko." });
    }

    currencies.push(currency);
    appCache.set("currencies", currencies);

    return reply.code(201).send({ status: "success" });
}

export async function delCurrency(request: FastifyRequest, reply: FastifyReply) {
    const { currency } = request.body as { currency: string };
    
    const currencies: string[] | undefined = appCache.get("currencies");
    if (!currencies) {
        throw new Error("Cache doesn't set.");
    }

    const exist = currencies.includes(currency);
    if (!exist) {
        return reply.code(404).send({ error: "The currency doesn't exist." });
    }

    const newCurrencies = currencies.filter(item => item !== currency);
    appCache.set("currencies", newCurrencies);

    return reply.code(201).send({ status: "success" });
}
