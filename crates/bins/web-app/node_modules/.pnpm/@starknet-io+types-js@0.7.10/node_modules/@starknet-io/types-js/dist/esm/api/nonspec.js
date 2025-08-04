// Enums Derived From Spec Types (require manual check for changes)
export const ETransactionType = {
    DECLARE: 'DECLARE',
    DEPLOY: 'DEPLOY',
    DEPLOY_ACCOUNT: 'DEPLOY_ACCOUNT',
    INVOKE: 'INVOKE',
    L1_HANDLER: 'L1_HANDLER',
};
export const ESimulationFlag = {
    SKIP_VALIDATE: 'SKIP_VALIDATE',
    SKIP_FEE_CHARGE: 'SKIP_FEE_CHARGE',
};
export const ETransactionStatus = {
    RECEIVED: 'RECEIVED',
    REJECTED: 'REJECTED',
    ACCEPTED_ON_L2: 'ACCEPTED_ON_L2',
    ACCEPTED_ON_L1: 'ACCEPTED_ON_L1',
};
export const ETransactionFinalityStatus = {
    ACCEPTED_ON_L2: 'ACCEPTED_ON_L2',
    ACCEPTED_ON_L1: 'ACCEPTED_ON_L1',
};
export const ETransactionExecutionStatus = {
    SUCCEEDED: 'SUCCEEDED',
    REVERTED: 'REVERTED',
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
    V0: '0x0',
    V1: '0x1',
    V2: '0x2',
    V3: '0x3',
    F0: '0x100000000000000000000000000000000',
    F1: '0x100000000000000000000000000000001',
    F2: '0x100000000000000000000000000000002',
    F3: '0x100000000000000000000000000000003',
};
/**
 * Old Transaction Versions
 */
export const ETransactionVersion2 = {
    V0: '0x0',
    V1: '0x1',
    V2: '0x2',
    F0: '0x100000000000000000000000000000000',
    F1: '0x100000000000000000000000000000001',
    F2: '0x100000000000000000000000000000002',
};
/**
 * V3 Transaction Versions
 */
export const ETransactionVersion3 = {
    V3: '0x3',
    F3: '0x100000000000000000000000000000003',
};
//# sourceMappingURL=nonspec.js.map