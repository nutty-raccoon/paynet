import type { ADDRESS, BLOCK_ID, BLOCK_NUMBER, BROADCASTED_DECLARE_TXN, BROADCASTED_DEPLOY_ACCOUNT_TXN, BROADCASTED_INVOKE_TXN, BROADCASTED_TXN, CHAIN_ID, CONTRACT_STORAGE_KEYS, EVENT_FILTER, EVENT_KEYS, FELT, FUNCTION_CALL, L1_TXN_HASH, MSG_FROM_L1, RESULT_PAGE_REQUEST, SIMULATION_FLAG, SIMULATION_FLAG_FOR_ESTIMATE_FEE, STORAGE_KEY, SUBSCRIPTION_BLOCK_ID, SUBSCRIPTION_ID, SubscriptionEventsResponse, SubscriptionNewHeadsResponse, SubscriptionPendingTransactionsResponse, SubscriptionReorgResponse, SubscriptionTransactionsStatusResponse, TXN_HASH } from './components.js';
import type * as Errors from './errors.js';
import type { CASM_COMPILED_CONTRACT_CLASS } from './executable.js';
import type { BlockHashAndNumber, BlockTransactionsTraces, BlockWithTxHashes, BlockWithTxReceipts, BlockWithTxs, ContractClass, DeclaredTransaction, DeployedAccountTransaction, Events, FeeEstimate, InvokedTransaction, L1L2MessagesStatus, Nonce, SimulateTransactionResponse, StateUpdate, StorageProof, Syncing, TransactionReceipt, TransactionStatus, TransactionTrace, TransactionWithHash } from './nonspec.js';
export type Methods = ReadMethods & WriteMethods & TraceMethods;
type ReadMethods = {
    starknet_specVersion: {
        params: [];
        /**
         * Semver of Starknet's JSON-RPC spec being used
         * @example 0.7.1
         */
        result: string;
    };
    starknet_getBlockWithTxHashes: {
        params: {
            block_id: BLOCK_ID;
        };
        result: BlockWithTxHashes;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_getBlockWithTxs: {
        params: {
            block_id: BLOCK_ID;
        };
        result: BlockWithTxs;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_getBlockWithReceipts: {
        params: {
            block_id: BLOCK_ID;
        };
        result: BlockWithTxReceipts;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_getStateUpdate: {
        params: {
            block_id: BLOCK_ID;
        };
        result: StateUpdate;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_getStorageAt: {
        params: {
            contract_address: ADDRESS;
            key: STORAGE_KEY;
            block_id: BLOCK_ID;
        };
        result: FELT;
        errors: Errors.CONTRACT_NOT_FOUND | Errors.BLOCK_NOT_FOUND;
    };
    starknet_getTransactionStatus: {
        params: {
            transaction_hash: TXN_HASH;
        };
        result: TransactionStatus;
        errors: Errors.TXN_HASH_NOT_FOUND;
    };
    starknet_getTransactionByHash: {
        params: {
            transaction_hash: TXN_HASH;
        };
        result: TransactionWithHash;
        errors: Errors.TXN_HASH_NOT_FOUND;
    };
    starknet_getTransactionByBlockIdAndIndex: {
        params: {
            block_id: BLOCK_ID;
            index: number;
        };
        result: TransactionWithHash;
        errors: Errors.BLOCK_NOT_FOUND | Errors.INVALID_TXN_INDEX;
    };
    starknet_getTransactionReceipt: {
        params: {
            transaction_hash: TXN_HASH;
        };
        result: TransactionReceipt;
        errors: Errors.TXN_HASH_NOT_FOUND;
    };
    starknet_getClass: {
        params: {
            block_id: BLOCK_ID;
            class_hash: FELT;
        };
        result: ContractClass;
        errors: Errors.BLOCK_NOT_FOUND | Errors.CLASS_HASH_NOT_FOUND;
    };
    starknet_getClassHashAt: {
        params: {
            block_id: BLOCK_ID;
            contract_address: ADDRESS;
        };
        result: FELT;
        errors: Errors.BLOCK_NOT_FOUND | Errors.CONTRACT_NOT_FOUND;
    };
    starknet_getClassAt: {
        params: {
            block_id: BLOCK_ID;
            contract_address: ADDRESS;
        };
        result: ContractClass;
        errors: Errors.BLOCK_NOT_FOUND | Errors.CONTRACT_NOT_FOUND;
    };
    starknet_getBlockTransactionCount: {
        params: {
            block_id: BLOCK_ID;
        };
        result: number;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_call: {
        params: {
            request: FUNCTION_CALL;
            block_id: BLOCK_ID;
        };
        result: FELT[];
        errors: Errors.CONTRACT_NOT_FOUND | Errors.ENTRYPOINT_NOT_FOUND | Errors.CONTRACT_ERROR | Errors.BLOCK_NOT_FOUND;
    };
    /**
     * Estimate the fee for Starknet transactions
     * Estimates the resources required by a given sequence of transactions when applied on a given state. If one of the transactions reverts or fails due to any reason (e.g. validation failure or an internal error), a TRANSACTION_EXECUTION_ERROR is returned.
     */
    starknet_estimateFee: {
        params: {
            request: BROADCASTED_TXN[];
            simulation_flags: [SIMULATION_FLAG_FOR_ESTIMATE_FEE] | [];
            block_id: BLOCK_ID;
        };
        result: FeeEstimate[];
        errors: Errors.TRANSACTION_EXECUTION_ERROR | Errors.BLOCK_NOT_FOUND;
    };
    starknet_estimateMessageFee: {
        params: {
            message: MSG_FROM_L1;
            block_id: BLOCK_ID;
        };
        result: FeeEstimate;
        errors: Errors.CONTRACT_ERROR | Errors.BLOCK_NOT_FOUND;
    };
    starknet_blockNumber: {
        params: [];
        result: BLOCK_NUMBER;
        errors: Errors.NO_BLOCKS;
    };
    starknet_blockHashAndNumber: {
        params: [];
        result: BlockHashAndNumber;
        errors: Errors.NO_BLOCKS;
    };
    starknet_chainId: {
        params: [];
        result: CHAIN_ID;
    };
    starknet_syncing: {
        params: [];
        /**
         * false if the node is not syncing, or an object with data about the syncing status
         */
        result: Syncing;
    };
    starknet_getEvents: {
        params: {
            filter: EVENT_FILTER & RESULT_PAGE_REQUEST;
        };
        result: Events;
        errors: Errors.PAGE_SIZE_TOO_BIG | Errors.INVALID_CONTINUATION_TOKEN | Errors.BLOCK_NOT_FOUND | Errors.TOO_MANY_KEYS_IN_FILTER;
    };
    starknet_getNonce: {
        params: {
            block_id: BLOCK_ID;
            contract_address: ADDRESS;
        };
        result: Nonce;
        errors: Errors.BLOCK_NOT_FOUND | Errors.CONTRACT_NOT_FOUND;
    };
    /**
     * Given an l1 tx hash, returns the associated l1_handler tx hashes and statuses for all L1 -> L2 messages sent by the l1 transaction, ordered by the l1 tx sending order
     */
    starknet_getMessagesStatus: {
        params: {
            /**
             * The hash of the L1 transaction that sent L1->L2 messages
             */
            transaction_hash: L1_TXN_HASH;
        };
        result: L1L2MessagesStatus;
        errors: Errors.TXN_HASH_NOT_FOUND;
    };
    /**
     * Get merkle paths in one of the state tries: global state, classes, individual contract
     */
    starknet_getStorageProof: {
        params: {
            /**
             * The hash of the requested block, or number (height) of the requested block, or a block tag
             */
            block_id: SUBSCRIPTION_BLOCK_ID;
            /**
             * a list of the class hashes for which we want to prove membership in the classes trie
             */
            class_hashes?: FELT[];
            /**
             * a list of contracts for which we want to prove membership in the global state trie
             */
            contract_addresses?: ADDRESS[];
            /**
             * a list of (contract_address, storage_keys) pairs
             */
            contracts_storage_keys?: CONTRACT_STORAGE_KEYS[];
        };
        /**
         * The requested storage proofs. Note that if a requested leaf has the default value, the path to it may end in an edge node whose path is not a prefix of the requested leaf, thus effectively proving non-membership
         */
        result: StorageProof;
        errors: Errors.BLOCK_NOT_FOUND | Errors.STORAGE_PROOF_NOT_SUPPORTED;
    };
    /**
     * Get the CASM code resulting from compiling a given class
     */
    starknet_getCompiledCasm: {
        params: {
            /**
             * The hash of the contract class whose CASM will be returned
             */
            class_hash: FELT;
        };
        result: CASM_COMPILED_CONTRACT_CLASS;
        errors: Errors.COMPILATION_ERROR | Errors.CLASS_HASH_NOT_FOUND;
    };
};
type WriteMethods = {
    starknet_addInvokeTransaction: {
        params: {
            invoke_transaction: BROADCASTED_INVOKE_TXN;
        };
        result: InvokedTransaction;
        errors: Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_RESOURCES_FOR_VALIDATE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.DUPLICATE_TX | Errors.UNSUPPORTED_TX_VERSION | Errors.UNEXPECTED_ERROR;
    };
    starknet_addDeclareTransaction: {
        params: {
            declare_transaction: BROADCASTED_DECLARE_TXN;
        };
        result: DeclaredTransaction;
        errors: Errors.CLASS_ALREADY_DECLARED | Errors.COMPILATION_FAILED | Errors.COMPILED_CLASS_HASH_MISMATCH | Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_RESOURCES_FOR_VALIDATE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.DUPLICATE_TX | Errors.CONTRACT_CLASS_SIZE_IS_TOO_LARGE | Errors.UNSUPPORTED_TX_VERSION | Errors.UNSUPPORTED_CONTRACT_CLASS_VERSION | Errors.UNEXPECTED_ERROR;
    };
    starknet_addDeployAccountTransaction: {
        params: {
            deploy_account_transaction: BROADCASTED_DEPLOY_ACCOUNT_TXN;
        };
        result: DeployedAccountTransaction;
        errors: Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_RESOURCES_FOR_VALIDATE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.CLASS_HASH_NOT_FOUND | Errors.DUPLICATE_TX | Errors.UNSUPPORTED_TX_VERSION | Errors.UNEXPECTED_ERROR;
    };
};
type TraceMethods = {
    starknet_traceTransaction: {
        params: {
            transaction_hash: TXN_HASH;
        };
        result: TransactionTrace;
        errors: Errors.TXN_HASH_NOT_FOUND | Errors.NO_TRACE_AVAILABLE;
    };
    starknet_traceBlockTransactions: {
        params: {
            block_id: BLOCK_ID;
        };
        result: BlockTransactionsTraces;
        errors: Errors.BLOCK_NOT_FOUND;
    };
    starknet_simulateTransactions: {
        params: {
            block_id: BLOCK_ID;
            transactions: Array<BROADCASTED_TXN>;
            simulation_flags: Array<SIMULATION_FLAG>;
        };
        result: SimulateTransactionResponse;
        errors: Errors.BLOCK_NOT_FOUND | Errors.TRANSACTION_EXECUTION_ERROR;
    };
};
export type WebSocketMethods = {
    /**
     * New block headers subscription.
     * Creates a WebSocket stream which will fire events for new block headers.
     */
    starknet_subscribeNewHeads: {
        params: {
            /**
             * The block to get notifications from, default is latest, limited to 1024 blocks back
             */
            block_id?: SUBSCRIPTION_BLOCK_ID;
        };
        result: SUBSCRIPTION_ID;
        errors: Errors.TOO_MANY_BLOCKS_BACK | Errors.BLOCK_NOT_FOUND;
        events: ['starknet_subscriptionNewHeads', 'starknet_subscriptionReorg'];
    };
    /**
     * New events subscription.
     * Creates a WebSocket stream which will fire events for new Starknet events with applied filters.
     */
    starknet_subscribeEvents: {
        params: {
            /**
             * Filter events by from_address which emitted the event
             */
            from_address?: ADDRESS;
            keys?: EVENT_KEYS;
            /**
             * The block to get notifications from, default is latest, limited to 1024 blocks back
             */
            block_id?: SUBSCRIPTION_BLOCK_ID;
        };
        result: SUBSCRIPTION_ID;
        errors: Errors.TOO_MANY_KEYS_IN_FILTER | Errors.TOO_MANY_BLOCKS_BACK | Errors.BLOCK_NOT_FOUND;
        events: ['starknet_subscriptionEvents', 'starknet_subscriptionReorg'];
    };
    /**
     * New transaction status subscription.
     * Creates a WebSocket stream which at first fires an event with the current known transaction status, followed by events for every transaction status update
     */
    starknet_subscribeTransactionStatus: {
        params: {
            transaction_hash: FELT;
        };
        result: SUBSCRIPTION_ID;
        events: ['starknet_subscriptionTransactionStatus', 'starknet_subscriptionReorg'];
    };
    /**
     * New Pending Transactions subscription.
     * Creates a WebSocket stream which will fire events when a new pending transaction is added. While there is no mempool, this notifies of transactions in the pending block.
     */
    starknet_subscribePendingTransactions: {
        params: {
            /**
             * "Get all transaction details, and not only the hash. If not provided, only hash is returned. Default is false"
             */
            transaction_details?: Boolean;
            /**
             * Filter transactions to only receive notification from address list
             */
            sender_address?: ADDRESS[];
        };
        result: SUBSCRIPTION_ID;
        errors: Errors.TOO_MANY_ADDRESSES_IN_FILTER;
        events: ['starknet_subscriptionPendingTransactions'];
    };
    /**
     * Close a previously opened ws stream, with the corresponding subscription id
     */
    starknet_unsubscribe: {
        params: {
            subscription_id: SUBSCRIPTION_ID;
        };
        result: Boolean;
        errors: Errors.INVALID_SUBSCRIPTION_ID;
    };
};
/**
 * Server -> Client events over WebSockets
 */
export type WebSocketEvents = {
    starknet_subscriptionReorg: SubscriptionReorgResponse;
    starknet_subscriptionNewHeads: SubscriptionNewHeadsResponse;
    starknet_subscriptionEvents: SubscriptionEventsResponse;
    starknet_subscriptionTransactionStatus: SubscriptionTransactionsStatusResponse;
    starknet_subscriptionPendingTransactions: SubscriptionPendingTransactionsResponse;
};
export {};
//# sourceMappingURL=methods.d.ts.map