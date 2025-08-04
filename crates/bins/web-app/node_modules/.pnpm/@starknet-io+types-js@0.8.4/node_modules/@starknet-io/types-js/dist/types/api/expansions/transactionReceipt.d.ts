/**
 * Possible permutations of transaction.
 *  BLOCK TYPE
 *  TYPE OF TRANSACTION
 *  EXECUTION (Reverted or not)
 *  FINALITY (Rejected on not) Receipt do not have Rejected
 */
export type IsPending<T> = Extract<T, {
    block_hash: never;
    block_number: never;
}>;
export type IsInBlock<T> = T extends {
    block_hash: string;
    block_number: number;
} ? T extends {
    block_hash: never;
} ? never : T : never;
export type IsType<T, ETransactionType> = Extract<T, {
    type: ETransactionType;
}>;
export type IsSucceeded<T> = Extract<T, {
    execution_status: 'SUCCEEDED';
}>;
export type IsReverted<T> = Extract<T, {
    execution_status: 'REVERTED';
}>;
//# sourceMappingURL=transactionReceipt.d.ts.map