import type { FastifyReply, FastifyRequest } from "fastify";
import { client, myCache } from "../..";

export async function addCurrency(request: FastifyRequest, reply: FastifyReply) {
    const { currency } = request.body as { currency: string };
    
    const currencies: string[] | undefined = myCache.get("currencies");
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
    myCache.set("currencies", currencies);

    return reply.code(201).send({ status: "success" });
}

export async function delCurrency(request: FastifyRequest, reply: FastifyReply) {
    const { currency } = request.body as { currency: string };
    
    const currencies: string[] | undefined = myCache.get("currencies");
    if (!currencies) {
        throw new Error("Cache doesn't set.");
    }

    const exist = currencies.includes(currency);
    if (!exist) {
        return reply.code(404).send({ error: "The currency doesn't exist." });
    }

    const newCurrencies = currencies.filter(item => item !== currency);
    myCache.set("currencies", newCurrencies);

    return reply.code(201).send({ status: "success" });
}
