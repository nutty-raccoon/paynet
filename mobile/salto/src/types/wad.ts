export type Wads = string;

export enum WadType {
    IN = "IN",
    OUT = "OUT",
}

export enum WadStatus {
    PENDING = "PENDING",
    CANCELLED = "CANCELLED",
    FINISHED = "FINISHED",
    FAILED = "FAILED",
    PARTIAL = "PARTIAL",
}

export interface WadHistoryItem {
    id: string;
    wadType: WadType;
    status: WadStatus;
    totalAmountJson: string;
    memo?: string;
    createdAt: number;
    modifiedAt: number;
}
