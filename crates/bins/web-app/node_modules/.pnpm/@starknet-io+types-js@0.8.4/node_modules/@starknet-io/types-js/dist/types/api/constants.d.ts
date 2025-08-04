export type STATUS_ACCEPTED_ON_L2 = 'ACCEPTED_ON_L2';
export declare const STATUS_ACCEPTED_ON_L2 = "ACCEPTED_ON_L2";
export type STATUS_ACCEPTED_ON_L1 = 'ACCEPTED_ON_L1';
export declare const STATUS_ACCEPTED_ON_L1 = "ACCEPTED_ON_L1";
export type STATUS_SUCCEEDED = 'SUCCEEDED';
export declare const STATUS_SUCCEEDED = "SUCCEEDED";
export type STATUS_REVERTED = 'REVERTED';
export declare const STATUS_REVERTED = "REVERTED";
export type STATUS_PENDING = 'PENDING';
export declare const STATUS_PENDING = "PENDING";
export type STATUS_REJECTED = 'REJECTED';
export declare const STATUS_REJECTED = "REJECTED";
export type STATUS_RECEIVED = 'RECEIVED';
export declare const STATUS_RECEIVED = "RECEIVED";
export type TXN_TYPE_DECLARE = 'DECLARE';
export declare const TXN_TYPE_DECLARE = "DECLARE";
export type TXN_TYPE_DEPLOY = 'DEPLOY';
export declare const TXN_TYPE_DEPLOY = "DEPLOY";
export type TXN_TYPE_DEPLOY_ACCOUNT = 'DEPLOY_ACCOUNT';
export declare const TXN_TYPE_DEPLOY_ACCOUNT = "DEPLOY_ACCOUNT";
export type TXN_TYPE_INVOKE = 'INVOKE';
export declare const TXN_TYPE_INVOKE = "INVOKE";
export type TXN_TYPE_L1_HANDLER = 'L1_HANDLER';
export declare const TXN_TYPE_L1_HANDLER = "L1_HANDLER";
export type STRUCT_ABI_TYPE = 'struct';
export declare const STRUCT_ABI_TYPE = "struct";
export type EVENT_ABI_TYPE = 'event';
export declare const EVENT_ABI_TYPE = "event";
export type ABI_TYPE_FUNCTION = 'function';
export declare const ABI_TYPE_FUNCTION = "function";
export type ABI_TYPE_CONSTRUCTOR = 'constructor';
export declare const ABI_TYPE_CONSTRUCTOR = "constructor";
export type ABI_TYPE_L1_HANDLER = 'l1_handler';
export declare const ABI_TYPE_L1_HANDLER = "l1_handler";
export type ABI_TYPE_ENUM = 'enum';
export declare const ABI_TYPE_ENUM = "enum";
export type STATE_MUTABILITY_VIEW = 'view';
export declare const STATE_MUTABILITY_VIEW = "view";
export type STATE_MUTABILITY_EXTERNAL = 'external';
export declare const STATE_MUTABILITY_EXTERNAL = "external";
export type PRICE_UNIT_WEI = 'WEI';
export declare const PRICE_UNIT_WEI = "WEI";
export type PRICE_UNIT_FRI = 'FRI';
export declare const PRICE_UNIT_FRI = "FRI";
export declare const L1_DA_MODE: {
    readonly BLOB: "BLOB";
    readonly CALLDATA: "CALLDATA";
};
export type L1_DA_MODE = (typeof L1_DA_MODE)[keyof typeof L1_DA_MODE];
/**
 * Represents the type of a function call.
 */
export declare const CALL_TYPE: {
    readonly DELEGATE: "DELEGATE";
    readonly LIBRARY_CALL: "LIBRARY_CALL";
    readonly CALL: "CALL";
};
export type CALL_TYPE = (typeof CALL_TYPE)[keyof typeof CALL_TYPE];
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
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly V0: "0x0";
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly V1: "0x1";
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly V2: "0x2";
    readonly V3: "0x3";
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly F0: "0x100000000000000000000000000000000";
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly F1: "0x100000000000000000000000000000001";
    /**
     * @deprecated Starknet 0.14 will not support this transaction
     */
    readonly F2: "0x100000000000000000000000000000002";
    readonly F3: "0x100000000000000000000000000000003";
};
export type ETransactionVersion = (typeof ETransactionVersion)[keyof typeof ETransactionVersion];
/**
 * Old Transaction Versions
 */
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export declare const ETransactionVersion2: {
    readonly V0: "0x0";
    readonly V1: "0x1";
    readonly V2: "0x2";
    readonly F0: "0x100000000000000000000000000000000";
    readonly F1: "0x100000000000000000000000000000001";
    readonly F2: "0x100000000000000000000000000000002";
};
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type ETransactionVersion2 = (typeof ETransactionVersion2)[keyof typeof ETransactionVersion2];
/**
 * V3 Transaction Versions
 */
export declare const ETransactionVersion3: {
    readonly V3: "0x3";
    readonly F3: "0x100000000000000000000000000000003";
};
export type ETransactionVersion3 = (typeof ETransactionVersion3)[keyof typeof ETransactionVersion3];
//# sourceMappingURL=constants.d.ts.map