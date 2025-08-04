"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ETransactionVersion3 = exports.ETransactionVersion2 = exports.ETransactionVersion = exports.EDAMode = exports.EDataAvailabilityMode = exports.EBlockTag = exports.ETransactionExecutionStatus = exports.ETransactionFinalityStatus = exports.ETransactionStatus = exports.ESimulationFlag = exports.ETransactionType = void 0;
exports.ETransactionType = {
    DECLARE: 'DECLARE',
    DEPLOY: 'DEPLOY',
    DEPLOY_ACCOUNT: 'DEPLOY_ACCOUNT',
    INVOKE: 'INVOKE',
    L1_HANDLER: 'L1_HANDLER',
};
exports.ESimulationFlag = {
    SKIP_VALIDATE: 'SKIP_VALIDATE',
    SKIP_FEE_CHARGE: 'SKIP_FEE_CHARGE',
};
exports.ETransactionStatus = {
    RECEIVED: 'RECEIVED',
    REJECTED: 'REJECTED',
    ACCEPTED_ON_L2: 'ACCEPTED_ON_L2',
    ACCEPTED_ON_L1: 'ACCEPTED_ON_L1',
};
exports.ETransactionFinalityStatus = {
    ACCEPTED_ON_L2: 'ACCEPTED_ON_L2',
    ACCEPTED_ON_L1: 'ACCEPTED_ON_L1',
};
exports.ETransactionExecutionStatus = {
    SUCCEEDED: 'SUCCEEDED',
    REVERTED: 'REVERTED',
};
exports.EBlockTag = {
    LATEST: 'latest',
    PENDING: 'pending',
};
exports.EDataAvailabilityMode = {
    L1: 'L1',
    L2: 'L2',
};
exports.EDAMode = {
    L1: 0,
    L2: 1,
};
exports.ETransactionVersion = {
    V0: '0x0',
    V1: '0x1',
    V2: '0x2',
    V3: '0x3',
    F0: '0x100000000000000000000000000000000',
    F1: '0x100000000000000000000000000000001',
    F2: '0x100000000000000000000000000000002',
    F3: '0x100000000000000000000000000000003',
};
exports.ETransactionVersion2 = {
    V0: '0x0',
    V1: '0x1',
    V2: '0x2',
    F0: '0x100000000000000000000000000000000',
    F1: '0x100000000000000000000000000000001',
    F2: '0x100000000000000000000000000000002',
};
exports.ETransactionVersion3 = {
    V3: '0x3',
    F3: '0x100000000000000000000000000000003',
};
//# sourceMappingURL=nonspec.js.map