import type { Balance } from "./node";

export type Wads = string;

export enum WadType {
    IN = "IN",
    OUT = "OUT",
}

export enum WadStatus {
    PENDING = "PENDING",
    FINISHED = "FINISHED",
    FAILED = "FAILED",
}

export interface WadHistoryItem {
    id: string;
    type: WadType;
    status: WadStatus;
    amounts: Balance[];
    memo?: string;
    createdAt: number;
    modifiedAt: number;
}
