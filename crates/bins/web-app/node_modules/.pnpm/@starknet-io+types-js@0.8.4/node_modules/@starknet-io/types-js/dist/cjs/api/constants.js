"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ETransactionVersion3 = exports.ETransactionVersion2 = exports.ETransactionVersion = exports.EDAMode = exports.EDataAvailabilityMode = exports.EBlockTag = exports.ETransactionExecutionStatus = exports.ETransactionFinalityStatus = exports.ETransactionStatus = exports.ESimulationFlag = exports.ETransactionType = exports.CALL_TYPE = exports.L1_DA_MODE = exports.PRICE_UNIT_FRI = exports.PRICE_UNIT_WEI = exports.STATE_MUTABILITY_EXTERNAL = exports.STATE_MUTABILITY_VIEW = exports.ABI_TYPE_ENUM = exports.ABI_TYPE_L1_HANDLER = exports.ABI_TYPE_CONSTRUCTOR = exports.ABI_TYPE_FUNCTION = exports.EVENT_ABI_TYPE = exports.STRUCT_ABI_TYPE = exports.TXN_TYPE_L1_HANDLER = exports.TXN_TYPE_INVOKE = exports.TXN_TYPE_DEPLOY_ACCOUNT = exports.TXN_TYPE_DEPLOY = exports.TXN_TYPE_DECLARE = exports.STATUS_RECEIVED = exports.STATUS_REJECTED = exports.STATUS_PENDING = exports.STATUS_REVERTED = exports.STATUS_SUCCEEDED = exports.STATUS_ACCEPTED_ON_L1 = exports.STATUS_ACCEPTED_ON_L2 = void 0;
exports.STATUS_ACCEPTED_ON_L2 = 'ACCEPTED_ON_L2';
exports.STATUS_ACCEPTED_ON_L1 = 'ACCEPTED_ON_L1';
exports.STATUS_SUCCEEDED = 'SUCCEEDED';
exports.STATUS_REVERTED = 'REVERTED';
exports.STATUS_PENDING = 'PENDING';
exports.STATUS_REJECTED = 'REJECTED';
exports.STATUS_RECEIVED = 'RECEIVED';
exports.TXN_TYPE_DECLARE = 'DECLARE';
exports.TXN_TYPE_DEPLOY = 'DEPLOY';
exports.TXN_TYPE_DEPLOY_ACCOUNT = 'DEPLOY_ACCOUNT';
exports.TXN_TYPE_INVOKE = 'INVOKE';
exports.TXN_TYPE_L1_HANDLER = 'L1_HANDLER';
exports.STRUCT_ABI_TYPE = 'struct';
exports.EVENT_ABI_TYPE = 'event';
exports.ABI_TYPE_FUNCTION = 'function';
exports.ABI_TYPE_CONSTRUCTOR = 'constructor';
exports.ABI_TYPE_L1_HANDLER = 'l1_handler';
exports.ABI_TYPE_ENUM = 'enum';
exports.STATE_MUTABILITY_VIEW = 'view';
exports.STATE_MUTABILITY_EXTERNAL = 'external';
exports.PRICE_UNIT_WEI = 'WEI';
exports.PRICE_UNIT_FRI = 'FRI';
exports.L1_DA_MODE = {
    BLOB: 'BLOB',
    CALLDATA: 'CALLDATA',
};
exports.CALL_TYPE = {
    DELEGATE: 'DELEGATE',
    LIBRARY_CALL: 'LIBRARY_CALL',
    CALL: 'CALL',
};
exports.ETransactionType = {
    DECLARE: exports.TXN_TYPE_DECLARE,
    DEPLOY: exports.TXN_TYPE_DEPLOY,
    DEPLOY_ACCOUNT: exports.TXN_TYPE_DEPLOY_ACCOUNT,
    INVOKE: exports.TXN_TYPE_INVOKE,
    L1_HANDLER: exports.TXN_TYPE_L1_HANDLER,
};
exports.ESimulationFlag = {
    SKIP_VALIDATE: 'SKIP_VALIDATE',
    SKIP_FEE_CHARGE: 'SKIP_FEE_CHARGE',
};
exports.ETransactionStatus = {
    RECEIVED: exports.STATUS_RECEIVED,
    REJECTED: exports.STATUS_REJECTED,
    ACCEPTED_ON_L2: exports.STATUS_ACCEPTED_ON_L2,
    ACCEPTED_ON_L1: exports.STATUS_ACCEPTED_ON_L1,
};
exports.ETransactionFinalityStatus = {
    ACCEPTED_ON_L2: exports.STATUS_ACCEPTED_ON_L2,
    ACCEPTED_ON_L1: exports.STATUS_ACCEPTED_ON_L1,
};
exports.ETransactionExecutionStatus = {
    SUCCEEDED: exports.STATUS_SUCCEEDED,
    REVERTED: exports.STATUS_REVERTED,
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
    V0: exports.ETransactionVersion.V0,
    V1: exports.ETransactionVersion.V1,
    V2: exports.ETransactionVersion.V2,
    F0: exports.ETransactionVersion.F0,
    F1: exports.ETransactionVersion.F1,
    F2: exports.ETransactionVersion.F2,
};
exports.ETransactionVersion3 = {
    V3: exports.ETransactionVersion.V3,
    F3: exports.ETransactionVersion.F3,
};
//# sourceMappingURL=constants.js.map