import type { ADDRESS, BLOCK_ID, BLOCK_NUMBER, BROADCASTED_DECLARE_TXN, BROADCASTED_DEPLOY_ACCOUNT_TXN, BROADCASTED_INVOKE_TXN, BROADCASTED_TXN, CHAIN_ID, EVENT_FILTER, FELT, FUNCTION_CALL, MSG_FROM_L1, RESULT_PAGE_REQUEST, SIMULATION_FLAG, SIMULATION_FLAG_FOR_ESTIMATE_FEE, STORAGE_KEY, TXN_HASH } from './components.js';
import type * as Errors from './errors.js';
import type { BlockHashAndNumber, BlockTransactionsTraces, BlockWithTxHashes, BlockWithTxReceipts, BlockWithTxs, ContractClass, DeclaredTransaction, DeployedAccountTransaction, Events, FeeEstimate, InvokedTransaction, Nonce, SimulateTransactionResponse, StateUpdate, Syncing, TransactionReceipt, TransactionStatus, TransactionTrace, TransactionWithHash } from './nonspec.js';
export type Methods = ReadMethods & WriteMethods & TraceMethods;
type ReadMethods = {
    starknet_specVersion: {
        params: [];
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
        errors: Errors.CONTRACT_NOT_FOUND | Errors.CONTRACT_ERROR | Errors.BLOCK_NOT_FOUND;
    };
    starknet_estimateFee: {
        params: {
            request: BROADCASTED_TXN[];
            simulation_flags?: [SIMULATION_FLAG_FOR_ESTIMATE_FEE] | [];
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
};
type WriteMethods = {
    starknet_addInvokeTransaction: {
        params: {
            invoke_transaction: BROADCASTED_INVOKE_TXN;
        };
        result: InvokedTransaction;
        errors: Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_MAX_FEE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.DUPLICATE_TX | Errors.UNSUPPORTED_TX_VERSION | Errors.UNEXPECTED_ERROR;
    };
    starknet_addDeclareTransaction: {
        params: {
            declare_transaction: BROADCASTED_DECLARE_TXN;
        };
        result: DeclaredTransaction;
        errors: Errors.CLASS_ALREADY_DECLARED | Errors.COMPILATION_FAILED | Errors.COMPILED_CLASS_HASH_MISMATCH | Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_MAX_FEE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.DUPLICATE_TX | Errors.CONTRACT_CLASS_SIZE_IS_TOO_LARGE | Errors.UNSUPPORTED_TX_VERSION | Errors.UNSUPPORTED_CONTRACT_CLASS_VERSION | Errors.UNEXPECTED_ERROR;
    };
    starknet_addDeployAccountTransaction: {
        params: {
            deploy_account_transaction: BROADCASTED_DEPLOY_ACCOUNT_TXN;
        };
        result: DeployedAccountTransaction;
        errors: Errors.INSUFFICIENT_ACCOUNT_BALANCE | Errors.INSUFFICIENT_MAX_FEE | Errors.INVALID_TRANSACTION_NONCE | Errors.VALIDATION_FAILURE | Errors.NON_ACCOUNT | Errors.CLASS_HASH_NOT_FOUND | Errors.DUPLICATE_TX | Errors.UNSUPPORTED_TX_VERSION | Errors.UNEXPECTED_ERROR;
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
export {};
//# sourceMappingURL=methods.d.ts.map