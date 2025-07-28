export type Price = {
    symbol: string;
    address: string;
    price: {
        currency: string;
        value: number;
    }[];
};