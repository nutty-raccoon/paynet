import {fastify, type Token} from "../index";
import { client, appCache } from "..";

export async function fetchPrice() {
    try{
        let currencies: string[] | undefined = appCache.get("currencies");
        if (!currencies) {
            throw new Error ("No currency set.");
        }
        const tokens: Record<string, Token[]> | undefined = appCache.get("tokens");
        if (!tokens) {
            throw new Error ("No token set.")
        }
        let allCurrencies: string = currencies.join(",");

        let newCache: {
            symbol: string;
            address: string;
            price: {
                currency: string;
                value: number;
            }[];
        }[] = [];
        
        for (const chain in tokens) {
            let list = tokens[chain]!;
            let addresses = list.map(token => token.address).join(",");

            // any type because the default type is not good
            const res: any = await client.simple.tokenPrice.getID(chain, { vs_currencies: allCurrencies, contract_addresses: addresses });
            list.forEach(token => {
                const newPrice: {currency: string, value: number}[] = currencies.map((currency) =>{return {currency, value: res[token.address][currency]}});

                newCache.push({
                    symbol: token.symbol,
                    address: token.address,
                    price: newPrice,
                });
            });
        }

        appCache.set("last_price", newCache);

        const now = new Date().toISOString();
        appCache.set("last_update", now);

        fastify.log.info("Price has been updated at " + now);
    } catch (err) {
        console.error("Error: ", err);
    }
}