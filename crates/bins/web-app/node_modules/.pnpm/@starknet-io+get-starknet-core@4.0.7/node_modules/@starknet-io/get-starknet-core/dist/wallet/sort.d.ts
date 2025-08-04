import { WalletProvider } from "../discovery";
import { type StarknetWindowObject } from "@starknet-io/types-js";
export type Sort = string[] | "random" | null | undefined;
export declare const sortBy: <T extends WalletProvider | StarknetWindowObject>(wallets: T[], sort: Sort) => T[];
