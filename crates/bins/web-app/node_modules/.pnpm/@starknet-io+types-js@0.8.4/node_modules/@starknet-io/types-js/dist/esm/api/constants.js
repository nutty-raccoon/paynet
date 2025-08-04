export const STATUS_ACCEPTED_ON_L2 = 'ACCEPTED_ON_L2';
export const STATUS_ACCEPTED_ON_L1 = 'ACCEPTED_ON_L1';
export const STATUS_SUCCEEDED = 'SUCCEEDED';
export const STATUS_REVERTED = 'REVERTED';
export const STATUS_PENDING = 'PENDING';
export const STATUS_REJECTED = 'REJECTED';
export const STATUS_RECEIVED = 'RECEIVED';
export const TXN_TYPE_DECLARE = 'DECLARE';
export const TXN_TYPE_DEPLOY = 'DEPLOY';
export const TXN_TYPE_DEPLOY_ACCOUNT = 'DEPLOY_ACCOUNT';
export const TXN_TYPE_INVOKE = 'INVOKE';
export const TXN_TYPE_L1_HANDLER = 'L1_HANDLER';
export const STRUCT_ABI_TYPE = 'struct';
export const EVENT_ABI_TYPE = 'event';
export const ABI_TYPE_FUNCTION = 'function';
export const ABI_TYPE_CONSTRUCTOR = 'constructor';
export const ABI_TYPE_L1_HANDLER = 'l1_handler';
export const ABI_TYPE_ENUM = 'enum';
export const STATE_MUTABILITY_VIEW = 'view';
export const STATE_MUTABILITY_EXTERNAL = 'external';
export const PRICE_UNIT_WEI = 'WEI';
export const PRICE_UNIT_FRI = 'FRI';
export const L1_DA_MODE = {
    BLOB: 'BLOB',
    CALLDATA: 'CALLDATA',
};
/**
 * Represents the type of a function call.
 */
export const CALL_TYPE = {
    DELEGATE: 'DELEGATE',
    LIBRARY_CALL: 'LIBRARY_CALL',
    CALL: 'CALL',
};
// Enums Derived From Spec Types (require manual check for changes)
export const ETransactionType = {
    DECLARE: TXN_TYPE_DECLARE,
    DEPLOY: TXN_TYPE_DEPLOY,
    DEPLOY_ACCOUNT: TXN_TYPE_DEPLOY_ACCOUNT,
    INVOKE: TXN_TYPE_INVOKE,
    L1_HANDLER: TXN_TYPE_L1_HANDLER,
};
export const ESimulationFlag = {
    SKIP_VALIDATE: 'SKIP_VALIDATE',
    SKIP_FEE_CHARGE: 'SKIP_FEE_CHARGE',
};
export const ETransactionStatus = {
    RECEIVED: STATUS_RECEIVED,
    REJECTED: STATUS_REJECTED,
    ACCEPTED_ON_L2: STATUS_ACCEPTED_ON_L2,
    ACCEPTED_ON_L1: STATUS_ACCEPTED_ON_L1,
};
export const ETransactionFinalityStatus = {
    ACCEPTED_ON_L2: STATUS_ACCEPTED_ON_L2,
    ACCEPTED_ON_L1: STATUS_ACCEPTED_ON_L1,
};
export const ETransactionExecutionStatus = {
    SUCCEEDED: STATUS_SUCCEEDED,
    REVERTED: STATUS_REVERTED,
};
export const EBlockTag = {
    LATEST: 'latest',
    PENDING: 'pending',
};
// 'L1' | 'L2'
export const EDataAvailabilityMode = {
    L1: 'L1',
    L2: 'L2',
};
// 0 | 1
export const EDAMode = {
    L1: 0,
    L2: 1,
};
/**
 * V_ Transaction versions HexString
 * F_ Fee Transaction Versions HexString (2 ** 128 + TRANSACTION_VERSION)
 */
export const ETransactionVersion = {
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    V0: '0x0',
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    V1: '0x1',
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    V2: '0x2',
    V3: '0x3',
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    F0: '0x100000000000000000000000000000000',
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    F1: '0x100000000000000000000000000000001',
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    F2: '0x100000000000000000000000000000002',
    F3: '0x100000000000000000000000000000003',
};
/**
 * Old Transaction Versions
 */
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export const ETransactionVersion2 = {
    V0: ETransactionVersion.V0,
    V1: ETransactionVersion.V1,
    V2: ETransactionVersion.V2,
    F0: ETransactionVersion.F0,
    F1: ETransactionVersion.F1,
    F2: ETransactionVersion.F2,
};
/**
 * V3 Transaction Versions
 */
export const ETransactionVersion3 = {
    V3: ETransactionVersion.V3,
    F3: ETransactionVersion.F3,
};
//# sourceMappingURL=constants.js.map