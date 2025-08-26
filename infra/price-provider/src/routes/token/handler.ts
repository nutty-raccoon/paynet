import { client, appCache, type Token } from "../../index";
import type { FastifyRequest, FastifyReply } from 'fastify';

function tokenAlradyExist(a: Token, b: {chain: string, address: string}) {
    return a.address === b.address && a.chain === b.chain;
}

export async function addToken(request: FastifyRequest, reply: FastifyReply) {
    const { address, chain } = request.body as { address: string, chain: string };
    
    const tokens: Token[] | undefined = appCache.get("tokens");
    if (!tokens) {
        throw new Error("Cache doesn't set.");
    }

    const exist = tokens.some(t => tokenAlradyExist(t, {chain, address}));
    if (exist) {
        return reply.code(409).send({ error: "Token already added." });
    }

    const res = await client.coins.list.get({include_platform: true, status: "active"});
    let newToken = res.filter(obj => obj.platforms && obj.platforms[chain] === address);
    if (newToken.length === 0) {
        return reply.code(404).send({ error: "The token doesn't exist on CoinGecko." });
    }

    if (!newToken[0]?.symbol) throw new Error("The token is incomplete.");
    tokens.push({
        symbol: newToken[0].symbol,
        chain,
        address
    });
    appCache.set("tokens", tokens);

    return reply.code(201).send({ status: "success" });
}

function isSameToken(a: Token, b: Token) {
    return a.symbol === b.symbol && a.chain === b.chain && a.address === b.address;
}

export async function delToken(request: FastifyRequest, reply: FastifyReply) {
    const { symbol, address, chain } = request.body as { symbol: string, address: string, chain: string };
    const delToken = {symbol, chain, address};

    const tokens: Token[] | undefined = appCache.get("tokens");
    if (!tokens) {
        throw new Error("Cache doesn't set.");
    }

    const exist = tokens.some(t => isSameToken(t, delToken));
    if (!exist) {
        return reply.code(404).send({ error: "The token doesn't exist." });
    }

    const newTokens = tokens.filter(t => !isSameToken(t, delToken));
    appCache.set("tokens", newTokens);

    return reply.code(201).send({ status: "success" });
}