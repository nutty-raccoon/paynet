import {fastify} from "../index";
import { client, myCache, tokens } from "..";

export async function fetch_price() {
    try{
        let addresses = tokens.map(token => token.address).join(",");
        const res: any = await client.simple.tokenPrice.getID("ethereum", { vs_currencies: 'usd', contract_addresses: addresses });
        const new_cache = tokens.map(token => {
            const new_price = res[token.address];
            return {
                symbol: token.symbol,
                address: token.address,
                price: new_price,
            };
        });
        myCache.set("last_price", new_cache);
        fastify.log.info("Pice has been updated.");
    } catch (err) {
        console.error("Error: ", err);
    }
}