export type Price = {
    symbol: string;
    price: {
        currency: string;
        value: number;
    }[];
};