export const getPricesSchema = {
    querystring: {
        type: "object",
        properties: {
          currencies: { type: "string" },
          assets: { type: "string" },
        },
      },
}