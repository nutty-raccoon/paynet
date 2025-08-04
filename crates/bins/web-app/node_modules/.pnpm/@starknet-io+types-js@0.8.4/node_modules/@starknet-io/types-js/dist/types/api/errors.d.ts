import type { CONTRACT_EXECUTION_ERROR } from './components.js';
import type { STATUS_RECEIVED, STATUS_REJECTED } from './constants.js';
export interface FAILED_TO_RECEIVE_TXN {
    code: 1;
    message: 'Failed to write transaction';
}
export interface NO_TRACE_AVAILABLE {
    code: 10;
    message: 'No trace available for transaction';
    data: {
        status: STATUS_RECEIVED | STATUS_REJECTED;
    };
}
export interface CONTRACT_NOT_FOUND {
    code: 20;
    message: 'Contract not found';
}
export interface ENTRYPOINT_NOT_FOUND {
    code: 21;
    message: 'Requested entrypoint does not exist in the contract';
}
export interface BLOCK_NOT_FOUND {
    code: 24;
    message: 'Block not found';
}
export interface INVALID_TXN_INDEX {
    code: 27;
    message: 'Invalid transaction index in a block';
}
export interface CLASS_HASH_NOT_FOUND {
    code: 28;
    message: 'Class hash not found';
}
export interface TXN_HASH_NOT_FOUND {
    code: 29;
    message: 'Transaction hash not found';
}
export interface PAGE_SIZE_TOO_BIG {
    code: 31;
    message: 'Requested page size is too big';
}
export interface NO_BLOCKS {
    code: 32;
    message: 'There are no blocks';
}
export interface INVALID_CONTINUATION_TOKEN {
    code: 33;
    message: 'The supplied continuation token is invalid or unknown';
}
export interface TOO_MANY_KEYS_IN_FILTER {
    code: 34;
    message: 'Too many keys provided in a filter';
}
export interface CONTRACT_ERROR {
    code: 40;
    message: 'Contract error';
    data: {
        /**
         * the execution trace up to the point of failure
         */
        revert_error: CONTRACT_EXECUTION_ERROR;
    };
}
export interface TRANSACTION_EXECUTION_ERROR {
    code: 41;
    message: 'Transaction execution error';
    data: {
        transaction_index: number;
        execution_error: string;
    };
}
export interface STORAGE_PROOF_NOT_SUPPORTED {
    code: 42;
    message: "the node doesn't support storage proofs for blocks that are too far in the past";
}
export interface CLASS_ALREADY_DECLARED {
    code: 51;
    message: 'Class already declared';
}
export interface INVALID_TRANSACTION_NONCE {
    code: 52;
    message: 'Invalid transaction nonce';
}
export interface INSUFFICIENT_RESOURCES_FOR_VALIDATE {
    code: 53;
    message: "The transaction's resources don't cover validation or the minimal transaction fee";
}
export interface INSUFFICIENT_ACCOUNT_BALANCE {
    code: 54;
    message: "Account balance is smaller than the transaction's max_fee";
}
export interface VALIDATION_FAILURE {
    code: 55;
    message: 'Account validation failed';
    data: string;
}
export interface COMPILATION_FAILED {
    code: 56;
    message: 'Compilation failed';
    data: 'string';
}
export interface CONTRACT_CLASS_SIZE_IS_TOO_LARGE {
    code: 57;
    message: 'Contract class size it too large';
}
export interface NON_ACCOUNT {
    code: 58;
    message: 'Sender address in not an account contract';
}
export interface DUPLICATE_TX {
    code: 59;
    message: 'A transaction with the same hash already exists in the mempool';
}
export interface COMPILED_CLASS_HASH_MISMATCH {
    code: 60;
    message: 'the compiled class hash did not match the one supplied in the transaction';
}
export interface UNSUPPORTED_TX_VERSION {
    code: 61;
    message: 'the transaction version is not supported';
}
export interface UNSUPPORTED_CONTRACT_CLASS_VERSION {
    code: 62;
    message: 'the contract class version is not supported';
}
export interface UNEXPECTED_ERROR {
    code: 63;
    message: 'An unexpected error occurred';
    data: string;
}
export interface INVALID_SUBSCRIPTION_ID {
    code: 66;
    message: 'Invalid subscription id';
}
export interface TOO_MANY_ADDRESSES_IN_FILTER {
    code: 67;
    message: 'Too many addresses in filter sender_address filter';
}
export interface TOO_MANY_BLOCKS_BACK {
    code: 68;
    message: 'Cannot go back more than 1024 blocks';
}
export interface COMPILATION_ERROR {
    code: 100;
    message: 'Failed to compile the contract';
    /**
     * "More data about the compilation failure
     */
    data: {
        compilation_error: string;
    };
}
//# sourceMappingURL=errors.d.ts.map