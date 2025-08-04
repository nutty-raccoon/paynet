import { ADDRESS, FELT, SIGNATURE } from '../api/components';
import { OutsideExecutionTypedData, TIME_BOUNDS } from '../wallet-api';
/**
 * 256 bit unsigned integers, represented by a hex string of length at most 64
 * @pattern ^0x(0|[a-fA-F1-9]{1}[a-fA-F0-9]{0,63})$
 */
export type u256 = string;
/**
 * The object that defines an invocation of a function in a contract
 */
export type CALL = {
    to: ADDRESS;
    selector: FELT;
    calldata: FELT[];
};
/**
 * A unique identifier corresponding to an `execute` request to the paymaster
 */
export type TRACKING_ID = FELT;
/**
 * User transaction
 */
export type USER_DEPLOY_TRANSACTION = {
    type: 'deploy';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
};
export type USER_INVOKE_TRANSACTION = {
    type: 'invoke';
    invoke: USER_INVOKE;
};
export type USER_INVOKE = {
    user_address: ADDRESS;
    calls: CALL[];
};
export type USER_DEPLOY_AND_INVOKE_TRANSACTION = {
    type: 'deploy_and_invoke';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
    invoke: USER_INVOKE;
};
export type USER_TRANSACTION = USER_DEPLOY_TRANSACTION | USER_INVOKE_TRANSACTION | USER_DEPLOY_AND_INVOKE_TRANSACTION;
/**
 * User transaction
 */
export type EXECUTABLE_USER_DEPLOY_TRANSACTION = {
    type: 'deploy';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
};
export type EXECUTABLE_USER_INVOKE_TRANSACTION = {
    type: 'invoke';
    invoke: EXECUTABLE_USER_INVOKE;
};
export type EXECUTABLE_USER_INVOKE = {
    user_address: ADDRESS;
    typed_data: OutsideExecutionTypedData;
    signature: SIGNATURE;
};
export type EXECUTABLE_USER_DEPLOY_AND_INVOKE_TRANSACTION = {
    type: 'deploy_and_invoke';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
    invoke: EXECUTABLE_USER_INVOKE;
};
export type EXECUTABLE_USER_TRANSACTION = EXECUTABLE_USER_DEPLOY_TRANSACTION | EXECUTABLE_USER_INVOKE_TRANSACTION | EXECUTABLE_USER_DEPLOY_AND_INVOKE_TRANSACTION;
/**
 * Execution parameters
 */
export type SPONSORED_TRANSACTION = {
    mode: 'sponsored';
};
export type GASLESS_TRANSACTION = {
    mode: 'default';
    gas_token: FELT;
};
export type FEE_MODE = SPONSORED_TRANSACTION | GASLESS_TRANSACTION;
export type EXECUTION_PARAMETERS_V1 = {
    version: '0x1';
    fee_mode: FEE_MODE;
    time_bounds?: TIME_BOUNDS;
};
export type EXECUTION_PARAMETERS = EXECUTION_PARAMETERS_V1;
/**
 * Data required to deploy an account at an address
 */
export type ACCOUNT_DEPLOYMENT_DATA = {
    address: ADDRESS;
    class_hash: FELT;
    salt: FELT;
    calldata: FELT[];
    sigdata?: FELT[];
    version: 1;
};
/**
 * Object containing data about the token: contract address, number of decimals and current price in STRK
 */
export type TOKEN_DATA = {
    token_address: ADDRESS;
    decimals: number;
    price_in_strk: u256;
};
export type FEE_ESTIMATE = {
    gas_token_price_in_strk: FELT;
    estimated_fee_in_strk: FELT;
    estimated_fee_in_gas_token: FELT;
    suggested_max_fee_in_strk: FELT;
    suggested_max_fee_in_gas_token: FELT;
};
//# sourceMappingURL=components.d.ts.map