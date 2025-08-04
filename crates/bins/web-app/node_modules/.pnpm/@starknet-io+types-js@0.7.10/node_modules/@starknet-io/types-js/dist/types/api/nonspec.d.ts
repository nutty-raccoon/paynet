/**
 * Types that are not in spec but required for UX
 */
import type { ADDRESS, BLOCK_HASH, BLOCK_NUMBER, BLOCK_WITH_RECEIPTS, BLOCK_WITH_TXS, BLOCK_WITH_TX_HASHES, BROADCASTED_TXN, CHAIN_ID, CONTRACT_CLASS, CONTRACT_STORAGE_DIFF_ITEM, DEPRECATED_CONTRACT_CLASS, EMITTED_EVENT, EVENT, EVENTS_CHUNK, EVENT_FILTER, FEE_ESTIMATE, FEE_PAYMENT, FELT, MSG_FROM_L1, NONCE_UPDATE, PENDING_BLOCK_WITH_RECEIPTS, PENDING_BLOCK_WITH_TXS, PENDING_BLOCK_WITH_TX_HASHES, PENDING_STATE_UPDATE, PRICE_UNIT, REPLACED_CLASS, RESOURCE_BOUNDS_MAPPING, RESULT_PAGE_REQUEST, SIMULATION_FLAG, STATE_UPDATE, SYNC_STATUS, TRANSACTION_TRACE, TXN, TXN_EXECUTION_STATUS, TXN_HASH, TXN_RECEIPT, TXN_RECEIPT_WITH_BLOCK_INFO, TXN_STATUS } from './components.js';
export type ContractClass = CONTRACT_CLASS | DEPRECATED_CONTRACT_CLASS;
export type SimulateTransaction = {
    transaction_trace: TRANSACTION_TRACE;
    fee_estimation: FEE_ESTIMATE;
};
export type SimulateTransactionResponse = SimulateTransaction[];
export type FeeEstimate = FEE_ESTIMATE;
export type TransactionWithHash = TXN & {
    transaction_hash: TXN_HASH;
};
export type BlockHashAndNumber = {
    block_hash: BLOCK_HASH;
    block_number: BLOCK_NUMBER;
};
export type BlockWithTxs = BLOCK_WITH_TXS | PENDING_BLOCK_WITH_TXS;
export type BlockWithTxHashes = BLOCK_WITH_TX_HASHES | PENDING_BLOCK_WITH_TX_HASHES;
export type BlockWithTxReceipts = BLOCK_WITH_RECEIPTS | PENDING_BLOCK_WITH_RECEIPTS;
export type StateUpdate = STATE_UPDATE | PENDING_STATE_UPDATE;
export type BlockTransactionsTraces = {
    transaction_hash: FELT;
    trace_root: TRANSACTION_TRACE;
}[];
export type Syncing = false | SYNC_STATUS;
export type Events = EVENTS_CHUNK;
export type EmittedEvent = EMITTED_EVENT;
export type Event = EVENT;
export type InvokedTransaction = {
    transaction_hash: TXN_HASH;
};
export type DeclaredTransaction = {
    transaction_hash: TXN_HASH;
    class_hash: FELT;
};
export type DeployedAccountTransaction = {
    transaction_hash: TXN_HASH;
    contract_address: FELT;
};
export type ContractAddress = ADDRESS;
export type Felt = FELT;
export type Nonce = FELT;
export type TransactionHash = TXN_HASH;
export type TransactionTrace = TRANSACTION_TRACE;
export type BlockHash = BLOCK_HASH;
export type TransactionReceipt = TXN_RECEIPT_WITH_BLOCK_INFO;
export type Receipt = TXN_RECEIPT_WITH_BLOCK_INFO & BlockHashAndNumber;
export type PendingReceipt = TXN_RECEIPT;
export type EventFilter = EVENT_FILTER & RESULT_PAGE_REQUEST;
export type SimulationFlags = Array<SIMULATION_FLAG>;
export type L1Message = MSG_FROM_L1;
export type BaseTransaction = BROADCASTED_TXN;
export type ChainId = CHAIN_ID;
export type Transaction = TXN;
export type TransactionStatus = {
    finality_status: TXN_STATUS;
    execution_status?: TXN_EXECUTION_STATUS;
};
export type ResourceBounds = RESOURCE_BOUNDS_MAPPING;
export type FeePayment = FEE_PAYMENT;
export type PriceUnit = PRICE_UNIT;
export type StorageDiffs = Array<CONTRACT_STORAGE_DIFF_ITEM>;
export type DeprecatedDeclaredClasses = Array<FELT>;
export type NonceUpdates = NONCE_UPDATE[];
export type ReplacedClasses = REPLACED_CLASS[];
export declare const ETransactionType: {
    readonly DECLARE: "DECLARE";
    readonly DEPLOY: "DEPLOY";
    readonly DEPLOY_ACCOUNT: "DEPLOY_ACCOUNT";
    readonly INVOKE: "INVOKE";
    readonly L1_HANDLER: "L1_HANDLER";
};
export type ETransactionType = (typeof ETransactionType)[keyof typeof ETransactionType];
export declare const ESimulationFlag: {
    readonly SKIP_VALIDATE: "SKIP_VALIDATE";
    readonly SKIP_FEE_CHARGE: "SKIP_FEE_CHARGE";
};
export type ESimulationFlag = (typeof ESimulationFlag)[keyof typeof ESimulationFlag];
export declare const ETransactionStatus: {
    readonly RECEIVED: "RECEIVED";
    readonly REJECTED: "REJECTED";
    readonly ACCEPTED_ON_L2: "ACCEPTED_ON_L2";
    readonly ACCEPTED_ON_L1: "ACCEPTED_ON_L1";
};
export type ETransactionStatus = (typeof ETransactionStatus)[keyof typeof ETransactionStatus];
export declare const ETransactionFinalityStatus: {
    readonly ACCEPTED_ON_L2: "ACCEPTED_ON_L2";
    readonly ACCEPTED_ON_L1: "ACCEPTED_ON_L1";
};
export type ETransactionFinalityStatus = (typeof ETransactionFinalityStatus)[keyof typeof ETransactionFinalityStatus];
export declare const ETransactionExecutionStatus: {
    readonly SUCCEEDED: "SUCCEEDED";
    readonly REVERTED: "REVERTED";
};
export type ETransactionExecutionStatus = (typeof ETransactionExecutionStatus)[keyof typeof ETransactionExecutionStatus];
export declare const EBlockTag: {
    readonly LATEST: "latest";
    readonly PENDING: "pending";
};
export type EBlockTag = (typeof EBlockTag)[keyof typeof EBlockTag];
export declare const EDataAvailabilityMode: {
    readonly L1: "L1";
    readonly L2: "L2";
};
export type EDataAvailabilityMode = (typeof EDataAvailabilityMode)[keyof typeof EDataAvailabilityMode];
export declare const EDAMode: {
    readonly L1: 0;
    readonly L2: 1;
};
export type EDAMode = (typeof EDAMode)[keyof typeof EDAMode];
/**
 * V_ Transaction versions HexString
 * F_ Fee Transaction Versions HexString (2 ** 128 + TRANSACTION_VERSION)
 */
export declare const ETransactionVersion: {
    readonly V0: "0x0";
    readonly V1: "0x1";
    readonly V2: "0x2";
    readonly V3: "0x3";
    readonly F0: "0x100000000000000000000000000000000";
    readonly F1: "0x100000000000000000000000000000001";
    readonly F2: "0x100000000000000000000000000000002";
    readonly F3: "0x100000000000000000000000000000003";
};
export type ETransactionVersion = (typeof ETransactionVersion)[keyof typeof ETransactionVersion];
/**
 * Old Transaction Versions
 */
export declare const ETransactionVersion2: {
    readonly V0: "0x0";
    readonly V1: "0x1";
    readonly V2: "0x2";
    readonly F0: "0x100000000000000000000000000000000";
    readonly F1: "0x100000000000000000000000000000001";
    readonly F2: "0x100000000000000000000000000000002";
};
export type ETransactionVersion2 = (typeof ETransactionVersion2)[keyof typeof ETransactionVersion2];
/**
 * V3 Transaction Versions
 */
export declare const ETransactionVersion3: {
    readonly V3: "0x3";
    readonly F3: "0x100000000000000000000000000000003";
};
export type ETransactionVersion3 = (typeof ETransactionVersion3)[keyof typeof ETransactionVersion3];
//# sourceMappingURL=nonspec.d.ts.map