/**
 * Types that are not in spec but required for UX
 */
import { TXN_HASH } from '../api';
import { OutsideExecutionTypedData } from '../wallet-api';
import { ACCOUNT_DEPLOYMENT_DATA, EXECUTION_PARAMETERS, FEE_ESTIMATE, TRACKING_ID } from './components';
export type BuildDeployTransactionResponse = {
    type: 'deploy';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
    parameters: EXECUTION_PARAMETERS;
    fee: FEE_ESTIMATE;
};
export type BuildInvokeTransactionResponse = {
    type: 'invoke';
    typed_data: OutsideExecutionTypedData;
    parameters: EXECUTION_PARAMETERS;
    fee: FEE_ESTIMATE;
};
export type BuildDeployAndInvokeTransactionResponse = {
    type: 'deploy_and_invoke';
    deployment: ACCOUNT_DEPLOYMENT_DATA;
    typed_data: OutsideExecutionTypedData;
    parameters: EXECUTION_PARAMETERS;
    fee: FEE_ESTIMATE;
};
export type BuildTransactionResponse = BuildDeployTransactionResponse | BuildInvokeTransactionResponse | BuildDeployAndInvokeTransactionResponse;
export type ExecuteResponse = {
    tracking_id: TRACKING_ID;
    transaction_hash: TXN_HASH;
};
export type AccountDeploymentData = ACCOUNT_DEPLOYMENT_DATA;
//# sourceMappingURL=nonspec.d.ts.map