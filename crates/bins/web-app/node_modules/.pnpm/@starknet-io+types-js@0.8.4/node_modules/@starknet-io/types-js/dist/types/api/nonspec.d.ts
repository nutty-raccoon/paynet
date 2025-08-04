/**
 * Types that are not in spec but required for UX
 */
import type { ADDRESS, BLOCK_HASH, BLOCK_NUMBER, BLOCK_WITH_RECEIPTS, BLOCK_WITH_TXS, BLOCK_WITH_TX_HASHES, BROADCASTED_TXN, CHAIN_ID, CONTRACT_CLASS, CONTRACT_STORAGE_DIFF_ITEM, DEPRECATED_CONTRACT_CLASS, EMITTED_EVENT, EVENT, EVENTS_CHUNK, EVENT_FILTER, FEE_ESTIMATE, FEE_PAYMENT, FELT, MSG_FROM_L1, NODE_HASH_TO_NODE_MAPPING, NONCE_UPDATE, PENDING_BLOCK_WITH_RECEIPTS, PENDING_BLOCK_WITH_TXS, PENDING_BLOCK_WITH_TX_HASHES, PENDING_STATE_UPDATE, PRICE_UNIT, REPLACED_CLASS, RESOURCE_BOUNDS_MAPPING, RESULT_PAGE_REQUEST, SIMULATION_FLAG, STATE_UPDATE, SYNC_STATUS, TRANSACTION_TRACE, TXN, TXN_HASH, TXN_RECEIPT_WITH_BLOCK_INFO, TXN_STATUS, TXN_STATUS_RESULT, TXN_WITH_HASH } from './components.js';
import { CASM_COMPILED_CONTRACT_CLASS } from './executable.js';
import { OneOf } from './expansions/helpless.js';
import { IsInBlock, IsPending } from './expansions/transactionReceipt.js';
export type ContractClass = OneOf<[CONTRACT_CLASS, DEPRECATED_CONTRACT_CLASS]>;
export type SimulateTransaction = {
    transaction_trace: TRANSACTION_TRACE;
    fee_estimation: FEE_ESTIMATE;
};
export type SimulateTransactionResponse = SimulateTransaction[];
export type FeeEstimate = FEE_ESTIMATE;
export type TransactionWithHash = TXN_WITH_HASH;
export type BlockHashAndNumber = {
    block_hash: BLOCK_HASH;
    block_number: BLOCK_NUMBER;
};
export type BlockWithTxs = OneOf<[BLOCK_WITH_TXS, PENDING_BLOCK_WITH_TXS]>;
export type BlockWithTxHashes = OneOf<[BLOCK_WITH_TX_HASHES, PENDING_BLOCK_WITH_TX_HASHES]>;
export type BlockWithTxReceipts = OneOf<[BLOCK_WITH_RECEIPTS, PENDING_BLOCK_WITH_RECEIPTS]>;
export type StateUpdate = OneOf<[STATE_UPDATE, PENDING_STATE_UPDATE]>;
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
export type L1L2MessagesStatus = Array<L1L2MessageStatus>;
export type StorageProof = {
    classes_proof: NODE_HASH_TO_NODE_MAPPING;
    contracts_proof: {
        /**
         * The nodes in the union of the paths from the contracts tree root to the requested leaves
         */
        nodes: NODE_HASH_TO_NODE_MAPPING;
        /**
         * The nonce and class hash for each requested contract address, in the order in which they appear in the request. These values are needed to construct the associated leaf node
         */
        contract_leaves_data: {
            nonce: FELT;
            class_hash: FELT;
            storage_root: FELT;
        }[];
    };
    contracts_storage_proofs: NODE_HASH_TO_NODE_MAPPING[];
    global_roots: {
        contracts_tree_root: FELT;
        classes_tree_root: FELT;
        /**
         * the associated block hash (needed in case the caller used a block tag for the block_id parameter)
         */
        block_hash: FELT;
    };
};
export type CompiledCasm = CASM_COMPILED_CONTRACT_CLASS;
export type ContractAddress = ADDRESS;
export type Felt = FELT;
export type Nonce = FELT;
export type TransactionHash = TXN_HASH;
export type TransactionTrace = TRANSACTION_TRACE;
export type BlockHash = BLOCK_HASH;
/**
 * All Type Transaction Receipt
 */
export type TransactionReceipt = TXN_RECEIPT_WITH_BLOCK_INFO;
/**
 * All Type Transaction Receipt from production block
 */
export type TransactionReceiptProductionBlock = IsInBlock<TransactionReceipt>;
/**
 * All Type Transaction Receipt from pending block
 */
export type TransactionReceiptPendingBlock = IsPending<TransactionReceipt>;
export type EventFilter = EVENT_FILTER & RESULT_PAGE_REQUEST;
export type SimulationFlags = Array<SIMULATION_FLAG>;
export type L1Message = MSG_FROM_L1;
export type BaseTransaction = BROADCASTED_TXN;
export type ChainId = CHAIN_ID;
export type Transaction = TXN;
export type TransactionStatus = TXN_STATUS_RESULT;
export type ResourceBounds = RESOURCE_BOUNDS_MAPPING;
export type FeePayment = FEE_PAYMENT;
export type PriceUnit = PRICE_UNIT;
/**
 * Ethereum l1_handler tx hash and status for L1 -> L2 messages sent by the l1 transaction
 */
export type L1L2MessageStatus = {
    /**
     * l1_handler tx hash
     */
    transaction_hash: TXN_HASH;
    /**
     * finality status of the L1 -> L2 messages sent by the l1 transaction
     */
    finality_status: TXN_STATUS;
    /**
     * the failure reason, only appears if finality_status is REJECTED
     */
    failure_reason?: string;
};
export type StorageDiffs = Array<CONTRACT_STORAGE_DIFF_ITEM>;
export type DeprecatedDeclaredClasses = Array<FELT>;
export type NonceUpdates = NONCE_UPDATE[];
export type ReplacedClasses = REPLACED_CLASS[];
//# sourceMappingURL=nonspec.d.ts.map