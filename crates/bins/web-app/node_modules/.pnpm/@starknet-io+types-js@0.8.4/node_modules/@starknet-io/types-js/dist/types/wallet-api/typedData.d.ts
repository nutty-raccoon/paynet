import { ADDRESS, FELT } from '../api/components.js';
import type { ABI_TYPE_ENUM } from '../api/constants.js';
export type BLOCK_TIMESTAMP = number;
/**
 * Object containing timestamps corresponding to `Execute After` and `Execute Before`
 */
export type TIME_BOUNDS = {
    execute_after: BLOCK_TIMESTAMP;
    execute_before: BLOCK_TIMESTAMP;
};
export declare const TypedDataRevision: {
    readonly ACTIVE: "1";
    readonly LEGACY: "0";
};
export type TypedDataRevision = (typeof TypedDataRevision)[keyof typeof TypedDataRevision];
export type StarknetEnumType = {
    name: string;
    type: ABI_TYPE_ENUM;
    contains: string;
};
export type StarknetMerkleType = {
    name: string;
    type: 'merkletree';
    contains: string;
};
/**
 * SPEC: STARKNET_TYPE
 * A single type, as part of a struct. The `type` field can be any of the EIP-712 supported types.
 * Note that the `uint` and `int` aliases like in Solidity, and fixed point numbers are not supported by the EIP-712
 * standard.
 */
export type StarknetType = {
    name: string;
    type: string;
} | StarknetEnumType | StarknetMerkleType;
/**
 * The EIP712 domain struct. Any of these fields are optional, but it must contain at least one field.
 */
export interface StarknetDomain extends Record<string, unknown> {
    name?: string;
    version?: string;
    chainId?: string | number;
    revision?: string | number;
}
/**
 * SPEC: TYPED_DATA
 * The complete typed data, with all the structs, domain data, primary type of the message, and the message itself.
 */
export interface TypedData {
    types: Record<string, StarknetType[]>;
    primaryType: string;
    domain: StarknetDomain;
    message: object;
}
/**
 * "A typed data object (in the sense of SNIP-12) which represents an outside execution payload, according to SNIP-9
 */
export type OutsideExecutionTypedData = OutsideExecutionTypedDataV1 | OutsideExecutionTypedDataV2;
export type OutsideExecutionTypedDataV1 = {
    types: Record<string, StarknetType[]>;
    primaryType: string;
    domain: StarknetDomain;
    message: OutsideExecutionMessageV1;
};
export type OutsideExecutionTypedDataV2 = {
    types: Record<string, StarknetType[]>;
    primaryType: string;
    domain: StarknetDomain;
    message: OutsideExecutionMessageV2;
};
export type OutsideExecutionMessageV1 = {
    caller: FELT;
    nonce: FELT;
    execute_after: FELT;
    execute_before: FELT;
    calls_len: FELT;
    calls: OutsideCallV1[];
};
export type OutsideCallV1 = {
    to: ADDRESS;
    selector: FELT;
    calldata_len: FELT[];
    calldata: FELT[];
};
export type OutsideExecutionMessageV2 = {
    Caller: FELT;
    Nonce: FELT;
    'Execute After': FELT;
    'Execute Before': FELT;
    Calls: OutsideCallV2[];
};
export type OutsideCallV2 = {
    To: ADDRESS;
    Selector: FELT;
    Calldata: FELT[];
};
//# sourceMappingURL=typedData.d.ts.map