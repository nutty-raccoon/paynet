import type { Token } from ".";

let env = process.env;

export type Env = {
    tokens: Token[],
    currencies: string[],
    isPro: boolean,
    apiKey: string,
    host: string,
    port: number,
};

export async function setEnv(): Promise<Env> {
    if (!env.COIN_PRO_GECKO_API_KEY && !env.COIN_DEMO_GECKO_API_KEY) {
        throw new Error("Missing env var: COIN_DEMO_GECKO_API_KEY or COIN_PRO_GECKO_API_KEY");
    }
    if (!env.TOKENS || !env.CURRENCIES) {
        throw new Error("Missing env var: CURRENCIES or TOKENS");
    }

    const isPro = !!env.COIN_PRO_GECKO_API_KEY
    const apiKey = isPro ? env.COIN_PRO_GECKO_API_KEY! : env.COIN_DEMO_GECKO_API_KEY!;

    const host = env.HOST ? env.HOST : "localhost"
    const port = env.PORT ? parseInt(env.PORT) : 3000;

    const rawTokens = env.TOKENS;
    const rawCurrencies = env.CURRENCIES;
    const tokens: Token[] = JSON.parse(rawTokens);
    const currencies: string[] = JSON.parse(rawCurrencies);

    return {
        tokens,
        currencies,
        isPro,
        apiKey,
        host,
        port,
    }
}