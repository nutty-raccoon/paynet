import {fastify} from "../index";
import { client, myCache, tokens } from "..";

export async function fetchPrice() {
    try{
        let currencies: string[] | undefined = myCache.get("currencies");
        if (!currencies) {
            throw new Error ("No currencies set.");
        }

        let addresses = tokens.map(token => token.address).join(",");
        let allCurrencies: string = currencies.join(",");

        // any type because the default type is not good
        const res: any = await client.simple.tokenPrice.getID("ethereum", { vs_currencies: allCurrencies, contract_addresses: addresses });
        console.log(res);
        const newCache = tokens.map(token => {
            const newPrice: {currency: string, value: number}[] = currencies.map((currency) =>{return {currency, value: res[token.address][currency]}});

            return {
                symbol: token.symbol,
                address: token.address,
                price: newPrice,
            };
        });

        myCache.set("last_price", newCache);

        fastify.log.info("Price has been updated.");
    } catch (err) {
        console.error("Error: ", err);
    }
}