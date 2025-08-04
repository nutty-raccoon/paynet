import { USER_TRANSACTION, TOKEN_DATA, EXECUTION_PARAMETERS, EXECUTABLE_USER_TRANSACTION } from './components';
import * as Errors from './errors';
import { BuildTransactionResponse, ExecuteResponse } from './nonspec';
type ReadMethods = {
    paymaster_isAvailable: {
        params: [];
        result: boolean;
    };
    paymaster_buildTransaction: {
        params: {
            transaction: USER_TRANSACTION;
            parameters: EXECUTION_PARAMETERS;
        };
        result: BuildTransactionResponse;
        errors: Errors.INVALID_ADDRESS | Errors.CLASS_HASH_NOT_SUPPORTED | Errors.INVALID_DEPLOYMENT_DATA | Errors.TOKEN_NOT_SUPPORTED | Errors.INVALID_TIME_BOUNDS | Errors.UNKNOWN_ERROR | Errors.TRANSACTION_EXECUTION_ERROR;
    };
    paymaster_getSupportedTokens: {
        params: {};
        result: TOKEN_DATA[];
    };
};
type WriteMethods = {
    paymaster_executeTransaction: {
        params: {
            transaction: EXECUTABLE_USER_TRANSACTION;
            parameters: EXECUTION_PARAMETERS;
        };
        result: ExecuteResponse;
        errors: Errors.INVALID_ADDRESS | Errors.CLASS_HASH_NOT_SUPPORTED | Errors.INVALID_DEPLOYMENT_DATA | Errors.INVALID_SIGNATURE | Errors.UNKNOWN_ERROR | Errors.MAX_AMOUNT_TOO_LOW | Errors.TRANSACTION_EXECUTION_ERROR;
    };
};
export type Methods = ReadMethods & WriteMethods;
export {};
//# sourceMappingURL=methods.d.ts.map