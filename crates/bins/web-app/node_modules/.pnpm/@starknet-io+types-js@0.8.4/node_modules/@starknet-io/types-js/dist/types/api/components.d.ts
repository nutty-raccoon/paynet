import type { ABI_TYPE_CONSTRUCTOR, ABI_TYPE_FUNCTION, ABI_TYPE_L1_HANDLER, CALL_TYPE, EBlockTag, EDataAvailabilityMode, ESimulationFlag, ETransactionVersion, EVENT_ABI_TYPE, L1_DA_MODE, PRICE_UNIT_FRI, PRICE_UNIT_WEI, STATE_MUTABILITY_EXTERNAL, STATE_MUTABILITY_VIEW, STATUS_ACCEPTED_ON_L1, STATUS_ACCEPTED_ON_L2, STATUS_PENDING, STATUS_RECEIVED, STATUS_REJECTED, STATUS_REVERTED, STATUS_SUCCEEDED, STRUCT_ABI_TYPE, TXN_TYPE_DECLARE, TXN_TYPE_DEPLOY, TXN_TYPE_DEPLOY_ACCOUNT, TXN_TYPE_INVOKE } from './constants.js';
import { SimpleOneOf } from './expansions/helpless.js';
/**
 * A field element. represented by at most 63 hex digits
 * @pattern ^0x(0|[a-fA-F1-9]{1}[a-fA-F0-9]{0,62})$
 */
export type FELT = string;
/**
 * an ethereum address represented as 40 hex digits
 * @pattern ^0x[a-fA-F0-9]{40}$
 */
export type ETH_ADDRESS = string;
/**
 * A storage key. Represented as up to 62 hex digits, 3 bits, and 5 leading zeroes.
 * @pattern ^0x(0|[0-7]{1}[a-fA-F0-9]{0,62}$)
 */
export type STORAGE_KEY = string;
/**
 * A contract address on Starknet
 */
export type ADDRESS = FELT;
/**
 * string representing an integer number in prefixed hexadecimal format
 * @example "0x.."
 * @pattern ^0x[a-fA-F0-9]+$
 */
export type NUM_AS_HEX = string;
/**
 * 64 bit integers, represented by hex string of length at most 16
 * "pattern": "^0x(0|[a-fA-F1-9]{1}[a-fA-F0-9]{0,15})$"
 */
export type u64 = string;
/**
 * 128 bit integers, represented by hex string of length at most 32
 * "pattern": "^0x(0|[a-fA-F1-9]{1}[a-fA-F0-9]{0,31})$"
 */
export type u128 = string;
export type SIGNATURE = Array<FELT>;
export type BLOCK_NUMBER = number;
export type BLOCK_HASH = FELT;
/**
 * The hash of an Starknet transaction
 */
export type TXN_HASH = FELT;
/**
 * The hash of an Ethereum transaction
 */
export type L1_TXN_HASH = NUM_AS_HEX;
export type CHAIN_ID = NUM_AS_HEX;
export type STATE_MUTABILITY = STATE_MUTABILITY_VIEW | STATE_MUTABILITY_EXTERNAL;
export type FUNCTION_ABI_TYPE = ABI_TYPE_FUNCTION | ABI_TYPE_L1_HANDLER | ABI_TYPE_CONSTRUCTOR;
/**
 * common definition
 */
export type ABI_NAME_AND_TYPE = {
    name: string;
    type: string;
};
/**
 * common outputs
 */
export type ABI_TYPE = {
    type: string;
};
/**
 * Represents the type of an entry point.
 */
export type ENTRY_POINT_TYPE = Uppercase<STATE_MUTABILITY_EXTERNAL> | Uppercase<ABI_TYPE_L1_HANDLER> | Uppercase<ABI_TYPE_CONSTRUCTOR>;
/**
 * Represents the finality status of the transaction, including the case the txn is still in the mempool or failed validation during the block construction phase
 */
export type TXN_STATUS = STATUS_RECEIVED | STATUS_REJECTED | STATUS_ACCEPTED_ON_L2 | STATUS_ACCEPTED_ON_L1;
/**
 * Flags that indicate how to simulate a given transaction. By default, the sequencer behavior is replicated locally (enough funds are expected to be in the account, and the fee will be deducted from the balance before the simulation of the next transaction). To skip the fee charge, use the SKIP_FEE_CHARGE flag.
 */
export type SIMULATION_FLAG = typeof ESimulationFlag.SKIP_VALIDATE | typeof ESimulationFlag.SKIP_FEE_CHARGE;
/**
 * Data availability mode.
 * Specifies a storage domain in Starknet. Each domain has different guarantees regarding availability
 */
export type DA_MODE = EDataAvailabilityMode;
export type TXN_TYPE = TXN_TYPE_DECLARE | TXN_TYPE_DEPLOY | TXN_TYPE_DEPLOY_ACCOUNT | TXN_TYPE_INVOKE | Uppercase<ABI_TYPE_L1_HANDLER>;
export type TXN_FINALITY_STATUS = STATUS_ACCEPTED_ON_L2 | STATUS_ACCEPTED_ON_L1;
export type TXN_EXECUTION_STATUS = STATUS_SUCCEEDED | STATUS_REVERTED;
export type BLOCK_STATUS = STATUS_PENDING | STATUS_ACCEPTED_ON_L2 | STATUS_ACCEPTED_ON_L1 | STATUS_REJECTED;
export type BLOCK_SELECTOR = SimpleOneOf<{
    block_hash: BLOCK_HASH;
}, {
    block_number: BLOCK_NUMBER;
}>;
export type BLOCK_TAG = EBlockTag;
export type SUBSCRIPTION_BLOCK_TAG = 'latest';
/**
 * An identifier for this subscription stream used to associate events with this subscription.
 * Integer
 */
export type SUBSCRIPTION_ID = string;
export type NEW_TXN_STATUS = {
    transaction_hash: TXN_HASH;
    status: TXN_STATUS_RESULT;
};
/**
 * Data about reorganized blocks, starting and ending block number and hash
 */
export type REORG_DATA = {
    /**
     * Hash of the first known block of the orphaned chain
     */
    starting_block_hash: BLOCK_HASH;
    /**
     * Number of the first known block of the orphaned chain
     */
    starting_block_number: BLOCK_NUMBER;
    /**
     * The last known block of the orphaned chain
     */
    ending_block_hash: BLOCK_HASH;
    /**
     * Number of the last known block of the orphaned chain
     */
    ending_block_number: BLOCK_NUMBER;
};
export type SubscriptionNewHeadsResponse = {
    subscription_id: SUBSCRIPTION_ID;
    result: BLOCK_HEADER;
};
export type SubscriptionEventsResponse = {
    subscription_id: SUBSCRIPTION_ID;
    result: EMITTED_EVENT;
};
export type SubscriptionTransactionsStatusResponse = {
    subscription_id: SUBSCRIPTION_ID;
    result: NEW_TXN_STATUS;
};
export type SubscriptionPendingTransactionsResponse = {
    subscription_id: SUBSCRIPTION_ID;
    result: TXN_HASH | TXN_WITH_HASH;
};
export type SubscriptionReorgResponse = {
    subscription_id: SUBSCRIPTION_ID;
    result: REORG_DATA;
};
export type EVENTS_CHUNK = {
    /**
     * Returns matching events
     */
    events: EMITTED_EVENT[];
    /**
     * Use this token in a subsequent query to obtain the next page. Should not appear if there are no more pages.
     */
    continuation_token?: string;
};
export type RESULT_PAGE_REQUEST = {
    /**
     * The token returned from the previous query. If no token is provided the first page is returned.
     */
    continuation_token?: string;
    /**
     * Chunk size
     */
    chunk_size: number;
};
export type EMITTED_EVENT = EVENT & {
    block_hash: BLOCK_HASH;
    block_number: BLOCK_NUMBER;
    transaction_hash: TXN_HASH;
};
export type EVENT = {
    from_address: ADDRESS;
} & EVENT_CONTENT;
export type EVENT_CONTENT = {
    keys: FELT[];
    data: FELT[];
};
/**
 * The keys to filter over.
 * Per key (by position), designate the possible values to be matched for events to be returned. Empty array designates 'any' value.
 */
export type EVENT_KEYS = FELT[][];
export type EVENT_FILTER = {
    from_block?: BLOCK_ID;
    to_block?: BLOCK_ID;
    address?: ADDRESS;
    keys?: EVENT_KEYS;
};
export type BLOCK_ID = BLOCK_SELECTOR | BLOCK_TAG;
/**
 * same as BLOCK_ID, but without 'pending'
 */
export type SUBSCRIPTION_BLOCK_ID = BLOCK_SELECTOR | SUBSCRIPTION_BLOCK_TAG;
export type SYNC_STATUS = {
    starting_block_hash: BLOCK_HASH;
    starting_block_num: BLOCK_NUMBER;
    current_block_hash: BLOCK_HASH;
    current_block_num: BLOCK_NUMBER;
    highest_block_hash: BLOCK_HASH;
    highest_block_num: BLOCK_NUMBER;
};
export type NEW_CLASSES = {
    class_hash: FELT;
    compiled_class_hash: FELT;
};
export type REPLACED_CLASS = {
    class_hash: FELT;
    contract_address: FELT;
};
export type NONCE_UPDATE = {
    contract_address: ADDRESS;
    nonce: FELT;
};
export type STATE_DIFF = {
    storage_diffs: CONTRACT_STORAGE_DIFF_ITEM[];
    deprecated_declared_classes: FELT[];
    declared_classes: NEW_CLASSES[];
    deployed_contracts: DEPLOYED_CONTRACT_ITEM[];
    replaced_classes: REPLACED_CLASS[];
    nonces: NONCE_UPDATE[];
};
export type PENDING_STATE_UPDATE = {
    old_root: FELT;
    state_diff: STATE_DIFF;
    block_hash: never;
};
export type STATE_UPDATE = {
    block_hash: BLOCK_HASH;
    old_root: FELT;
    new_root: FELT;
    state_diff: STATE_DIFF;
};
export type BLOCK_BODY_WITH_TX_HASHES = {
    transactions: TXN_HASH[];
};
export type BLOCK_BODY_WITH_TXS = {
    transactions: TXN_WITH_HASH[];
};
export type BLOCK_BODY_WITH_RECEIPTS = {
    transactions: {
        transaction: TXN;
        receipt: TXN_RECEIPT;
    }[];
};
export type BLOCK_HEADER = {
    block_hash: BLOCK_HASH;
    parent_hash: BLOCK_HASH;
    block_number: BLOCK_NUMBER;
    new_root: FELT;
    timestamp: number;
    sequencer_address: FELT;
    l1_gas_price: RESOURCE_PRICE;
    l2_gas_price: RESOURCE_PRICE;
    l1_data_gas_price: RESOURCE_PRICE;
    l1_da_mode: L1_DA_MODE;
    starknet_version: string;
};
export type PENDING_BLOCK_HEADER = {
    parent_hash: BLOCK_HASH;
    timestamp: number;
    sequencer_address: FELT;
    l1_gas_price: RESOURCE_PRICE;
    l2_gas_price: RESOURCE_PRICE;
    l1_data_gas_price: RESOURCE_PRICE;
    l1_da_mode: L1_DA_MODE;
    starknet_version: string;
};
export type BLOCK_WITH_TX_HASHES = {
    status: BLOCK_STATUS;
} & BLOCK_HEADER & BLOCK_BODY_WITH_TX_HASHES;
export type BLOCK_WITH_TXS = {
    status: BLOCK_STATUS;
} & BLOCK_HEADER & BLOCK_BODY_WITH_TXS;
export type BLOCK_WITH_RECEIPTS = {
    status: BLOCK_STATUS;
} & BLOCK_HEADER & BLOCK_BODY_WITH_RECEIPTS;
export type PENDING_BLOCK_WITH_TX_HASHES = BLOCK_BODY_WITH_TX_HASHES & PENDING_BLOCK_HEADER;
export type PENDING_BLOCK_WITH_TXS = BLOCK_BODY_WITH_TXS & PENDING_BLOCK_HEADER;
export type PENDING_BLOCK_WITH_RECEIPTS = BLOCK_BODY_WITH_RECEIPTS & PENDING_BLOCK_HEADER;
export type DEPLOYED_CONTRACT_ITEM = {
    address: FELT;
    class_hash: FELT;
};
export type CONTRACT_STORAGE_DIFF_ITEM = {
    /**
     * The contract address for which the storage changed
     */
    address: FELT;
    /**
     * The changes in the storage of the contract
     */
    storage_entries: StorageDiffItem[];
};
export type StorageDiffItem = {
    /**
     * The key of the changed value
     */
    key: FELT;
    /**
     * The new value applied to the given address
     */
    value: FELT;
};
export type TXN = INVOKE_TXN | L1_HANDLER_TXN | DECLARE_TXN | DEPLOY_TXN | DEPLOY_ACCOUNT_TXN;
export type TXN_WITH_HASH = TXN & {
    transaction_hash: TXN_HASH;
};
export type DECLARE_TXN = DECLARE_TXN_V0 | DECLARE_TXN_V1 | DECLARE_TXN_V2 | DECLARE_TXN_V3;
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type DECLARE_TXN_V0 = {
    type: TXN_TYPE_DECLARE;
    sender_address: ADDRESS;
    max_fee: FELT;
    version: typeof ETransactionVersion.V0 | typeof ETransactionVersion.F0;
    signature: SIGNATURE;
    class_hash: FELT;
};
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type DECLARE_TXN_V1 = {
    type: TXN_TYPE_DECLARE;
    sender_address: ADDRESS;
    max_fee: FELT;
    version: typeof ETransactionVersion.V1 | typeof ETransactionVersion.F1;
    signature: SIGNATURE;
    nonce: FELT;
    class_hash: FELT;
};
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type DECLARE_TXN_V2 = {
    type: TXN_TYPE_DECLARE;
    sender_address: ADDRESS;
    compiled_class_hash: FELT;
    max_fee: FELT;
    version: typeof ETransactionVersion.V2 | typeof ETransactionVersion.F2;
    signature: SIGNATURE;
    nonce: FELT;
    class_hash: FELT;
};
export type DECLARE_TXN_V3 = {
    type: TXN_TYPE_DECLARE;
    sender_address: ADDRESS;
    compiled_class_hash: FELT;
    version: typeof ETransactionVersion.V3 | typeof ETransactionVersion.F3;
    signature: SIGNATURE;
    nonce: FELT;
    class_hash: FELT;
    resource_bounds: RESOURCE_BOUNDS_MAPPING;
    tip: u64;
    paymaster_data: FELT[];
    account_deployment_data: FELT[];
    nonce_data_availability_mode: DA_MODE;
    fee_data_availability_mode: DA_MODE;
};
export type BROADCASTED_TXN = BROADCASTED_INVOKE_TXN | BROADCASTED_DECLARE_TXN | BROADCASTED_DEPLOY_ACCOUNT_TXN;
export type BROADCASTED_INVOKE_TXN = INVOKE_TXN_V3;
export type BROADCASTED_DEPLOY_ACCOUNT_TXN = DEPLOY_ACCOUNT_TXN_V3;
export type BROADCASTED_DECLARE_TXN = BROADCASTED_DECLARE_TXN_V3;
export type BROADCASTED_DECLARE_TXN_V3 = {
    type: TXN_TYPE_DECLARE;
    sender_address: ADDRESS;
    compiled_class_hash: FELT;
    version: typeof ETransactionVersion.V3 | typeof ETransactionVersion.F3;
    signature: SIGNATURE;
    nonce: FELT;
    contract_class: CONTRACT_CLASS;
    resource_bounds: RESOURCE_BOUNDS_MAPPING;
    tip: u64;
    paymaster_data: FELT[];
    account_deployment_data: FELT[];
    nonce_data_availability_mode: DA_MODE;
    fee_data_availability_mode: DA_MODE;
};
export type DEPLOY_ACCOUNT_TXN = DEPLOY_ACCOUNT_TXN_V1 | DEPLOY_ACCOUNT_TXN_V3;
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type DEPLOY_ACCOUNT_TXN_V1 = {
    type: TXN_TYPE_DEPLOY_ACCOUNT;
    max_fee: FELT;
    version: typeof ETransactionVersion.V1 | typeof ETransactionVersion.F1;
    signature: SIGNATURE;
    nonce: FELT;
    contract_address_salt: FELT;
    constructor_calldata: FELT[];
    class_hash: FELT;
};
export type DEPLOY_ACCOUNT_TXN_V3 = {
    type: TXN_TYPE_DEPLOY_ACCOUNT;
    version: typeof ETransactionVersion.V3 | typeof ETransactionVersion.F3;
    signature: SIGNATURE;
    nonce: FELT;
    contract_address_salt: FELT;
    constructor_calldata: FELT[];
    class_hash: FELT;
    resource_bounds: RESOURCE_BOUNDS_MAPPING;
    tip: u64;
    paymaster_data: FELT[];
    nonce_data_availability_mode: DA_MODE;
    fee_data_availability_mode: DA_MODE;
};
export type DEPLOY_TXN = {
    type: TXN_TYPE_DEPLOY;
    version: FELT;
    contract_address_salt: FELT;
    constructor_calldata: FELT[];
    class_hash: FELT;
};
export type INVOKE_TXN = INVOKE_TXN_V0 | INVOKE_TXN_V1 | INVOKE_TXN_V3;
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type INVOKE_TXN_V0 = {
    type: TXN_TYPE_INVOKE;
    max_fee: FELT;
    version: typeof ETransactionVersion.V0 | typeof ETransactionVersion.F0;
    signature: SIGNATURE;
    contract_address: ADDRESS;
    entry_point_selector: FELT;
    calldata: FELT[];
};
/**
 * @deprecated Starknet 0.14 will not support this transaction
 */
export type INVOKE_TXN_V1 = {
    type: TXN_TYPE_INVOKE;
    sender_address: ADDRESS;
    calldata: FELT[];
    max_fee: FELT;
    version: typeof ETransactionVersion.V1 | typeof ETransactionVersion.F1;
    signature: SIGNATURE;
    nonce: FELT;
};
export type INVOKE_TXN_V3 = {
    type: TXN_TYPE_INVOKE;
    sender_address: ADDRESS;
    calldata: FELT[];
    version: typeof ETransactionVersion.V3 | typeof ETransactionVersion.F3;
    signature: SIGNATURE;
    nonce: FELT;
    resource_bounds: RESOURCE_BOUNDS_MAPPING;
    tip: u64;
    paymaster_data: FELT[];
    account_deployment_data: FELT[];
    nonce_data_availability_mode: DA_MODE;
    fee_data_availability_mode: DA_MODE;
};
export type L1_HANDLER_TXN = {
    version: typeof ETransactionVersion.V0;
    type: Uppercase<ABI_TYPE_L1_HANDLER>;
    nonce: NUM_AS_HEX;
} & FUNCTION_CALL;
export type COMMON_RECEIPT_PROPERTIES = {
    transaction_hash: TXN_HASH;
    actual_fee: FEE_PAYMENT;
    finality_status: TXN_FINALITY_STATUS;
    messages_sent: MSG_TO_L1[];
    events: EVENT[];
    execution_resources: EXECUTION_RESOURCES;
} & SimpleOneOf<SUCCESSFUL_COMMON_RECEIPT_PROPERTIES, REVERTED_COMMON_RECEIPT_PROPERTIES>;
type SUCCESSFUL_COMMON_RECEIPT_PROPERTIES = {
    execution_status: STATUS_SUCCEEDED;
};
type REVERTED_COMMON_RECEIPT_PROPERTIES = {
    execution_status: STATUS_REVERTED;
    revert_reason: string;
};
export type INVOKE_TXN_RECEIPT = {
    type: TXN_TYPE_INVOKE;
} & COMMON_RECEIPT_PROPERTIES;
export type DECLARE_TXN_RECEIPT = {
    type: TXN_TYPE_DECLARE;
} & COMMON_RECEIPT_PROPERTIES;
export type DEPLOY_ACCOUNT_TXN_RECEIPT = {
    type: TXN_TYPE_DEPLOY_ACCOUNT;
    contract_address: FELT;
} & COMMON_RECEIPT_PROPERTIES;
export type DEPLOY_TXN_RECEIPT = {
    type: TXN_TYPE_DEPLOY;
    contract_address: FELT;
} & COMMON_RECEIPT_PROPERTIES;
export type L1_HANDLER_TXN_RECEIPT = {
    type: Uppercase<ABI_TYPE_L1_HANDLER>;
    message_hash: NUM_AS_HEX;
} & COMMON_RECEIPT_PROPERTIES;
export type TXN_RECEIPT = INVOKE_TXN_RECEIPT | L1_HANDLER_TXN_RECEIPT | DECLARE_TXN_RECEIPT | DEPLOY_TXN_RECEIPT | DEPLOY_ACCOUNT_TXN_RECEIPT;
export type TXN_RECEIPT_WITH_BLOCK_INFO = TXN_RECEIPT & ({
    /**
     * "If this field is missing, it means the receipt belongs to the pending block"
     */
    block_hash: BLOCK_HASH;
    /**
     * "If this field is missing, it means the receipt belongs to the pending block"
     */
    block_number: BLOCK_NUMBER;
} | {
    /**
     * "If this field is missing, it means the receipt belongs to the pending block"
     */
    block_hash: never;
    /**
     * "If this field is missing, it means the receipt belongs to the pending block"
     */
    block_number: never;
});
export type MSG_TO_L1 = {
    from_address: FELT;
    to_address: FELT;
    payload: FELT[];
};
export type MSG_FROM_L1 = {
    from_address: ETH_ADDRESS;
    to_address: ADDRESS;
    entry_point_selector: FELT;
    payload: FELT[];
};
export type FUNCTION_CALL = {
    contract_address: ADDRESS;
    entry_point_selector: FELT;
    calldata: FELT[];
};
export type CONTRACT_CLASS = {
    sierra_program: FELT[];
    contract_class_version: string;
    entry_points_by_type: {
        CONSTRUCTOR: SIERRA_ENTRY_POINT[];
        EXTERNAL: SIERRA_ENTRY_POINT[];
        L1_HANDLER: SIERRA_ENTRY_POINT[];
    };
    abi: string;
};
export type DEPRECATED_CONTRACT_CLASS = {
    program: string;
    entry_points_by_type: {
        CONSTRUCTOR: DEPRECATED_CAIRO_ENTRY_POINT[];
        EXTERNAL: DEPRECATED_CAIRO_ENTRY_POINT[];
        L1_HANDLER: DEPRECATED_CAIRO_ENTRY_POINT[];
    };
    abi: CONTRACT_ABI;
};
export type DEPRECATED_CAIRO_ENTRY_POINT = {
    /**
     * "The offset of the entry point in the program"
     */
    offset: NUM_AS_HEX;
    /**
     * A unique identifier of the entry point (function) in the program
     */
    selector: FELT;
};
export type SIERRA_ENTRY_POINT = {
    selector: FELT;
    function_idx: number;
};
export type CONTRACT_ABI = readonly CONTRACT_ABI_ENTRY[];
export type CONTRACT_ABI_ENTRY = FUNCTION_ABI_ENTRY | EVENT_ABI_ENTRY | STRUCT_ABI_ENTRY;
export type STRUCT_ABI_ENTRY = {
    /**
     * Struct ABI type
     */
    type: STRUCT_ABI_TYPE;
    /**
     * Struct name
     */
    name: string;
    size: number;
    members: STRUCT_MEMBER[];
};
export type STRUCT_MEMBER = TYPED_PARAMETER & {
    /**
     * offset of this property within the struct
     */
    offset: number;
};
export type EVENT_ABI_ENTRY = {
    /**
     * Event ABI type
     */
    type: EVENT_ABI_TYPE;
    /**
     * Event name
     */
    name: string;
    keys: TYPED_PARAMETER[];
    data: TYPED_PARAMETER[];
};
export type FUNCTION_STATE_MUTABILITY = STATE_MUTABILITY_VIEW;
export type FUNCTION_ABI_ENTRY = {
    /**
     * Function ABI type
     */
    type: FUNCTION_ABI_TYPE;
    /**
     * Function name
     */
    name: string;
    /**
     * Typed parameter
     */
    inputs: TYPED_PARAMETER[];
    /**
     * Typed parameter
     */
    outputs: TYPED_PARAMETER[];
    /**
     * Function state mutability
     */
    stateMutability?: FUNCTION_STATE_MUTABILITY;
};
export type TYPED_PARAMETER = {
    /**
     * Parameter name
     */
    name: string;
    /**
     * Parameter type
     */
    type: string;
};
export type SIMULATION_FLAG_FOR_ESTIMATE_FEE = typeof ESimulationFlag.SKIP_VALIDATE;
export type PRICE_UNIT = PRICE_UNIT_WEI | PRICE_UNIT_FRI;
export type FEE_ESTIMATE = {
    /**
     * The Ethereum gas consumption of the transaction, charged for L1->L2 messages and, depending on the block's DA_MODE, state diffs.
     * Prev. name gas_consumed
     */
    l1_gas_consumed: number;
    /**
     * The gas price (in wei or fri, depending on the tx version) that was used in the cost estimation.
     * Prev. name gas_price
     */
    l1_gas_price: number;
    /**
     * The L2 gas consumption of the transaction.
     */
    l2_gas_consumed: number;
    /**
     * The L2 gas price (in wei or fri, depending on the tx version) that was used in the cost estimation.
     */
    l2_gas_price: number;
    /**
     * The Ethereum data gas consumption of the transaction.
     * Prev. name data_gas_consumed
     */
    l1_data_gas_consumed: number;
    /**
     * The data gas price (in wei or fri, depending on the tx version) that was used in the cost estimation.
     * Prev. name data_gas_price
     */
    l1_data_gas_price: number;
    overall_fee: number;
    unit: PRICE_UNIT;
};
export type FEE_PAYMENT = {
    amount: FELT;
    unit: PRICE_UNIT;
};
export type RESOURCE_BOUNDS_MAPPING = {
    /**
     * The max amount and max price per unit of L1 gas used in this tx
     */
    l1_gas: RESOURCE_BOUNDS;
    /**
     * The max amount and max price per unit of L1 blob gas used in this tx
     */
    l1_data_gas: RESOURCE_BOUNDS;
    /**
     * The max amount and max price per unit of L2 gas used in this tx
     */
    l2_gas: RESOURCE_BOUNDS;
};
export type RESOURCE_BOUNDS = {
    max_amount: u64;
    max_price_per_unit: u128;
};
export type RESOURCE_PRICE = {
    price_in_fri: FELT;
    price_in_wei: FELT;
};
/**
 * the resources consumed by the transaction
 */
export type EXECUTION_RESOURCES = {
    /**
     * l1 gas consumed by this transaction, used for l2-->l1 messages and state updates if blobs are not used.
     * integer
     */
    l1_gas: number;
    /**
     * data gas consumed by this transaction, 0 if blobs are not used
     * integer
     */
    l1_data_gas: number;
    /**
     * l2 gas consumed by this transaction, used for computation and calldata
     * Integer
     */
    l2_gas: number;
};
/**
 * a node in the Merkle-Patricia tree, can be a leaf, binary node, or an edge node
 */
export type MERKLE_NODE = SimpleOneOf<BINARY_NODE, EDGE_NODE>;
/**
 * an internal node whose both children are non-zero
 */
export type BINARY_NODE = {
    /**
     * the hash of the left child
     */
    left: FELT;
    /**
     * the hash of the right child
     */
    right: FELT;
};
/**
 * represents a path to the highest non-zero descendant node
 */
export type EDGE_NODE = {
    /**
     * an integer whose binary representation represents the path from the current node to its highest non-zero descendant (bounded by 2^251)
     */
    path: NUM_AS_HEX;
    /**
     * the length of the path (bounded by 251)
     */
    length: number;
    /**
     * the hash of the unique non-zero maximal-height descendant node
     */
    child: FELT;
};
/**
 * a node_hash -> node mapping of all the nodes in the union of the paths between the requested leaves and the root
 */
export type NODE_HASH_TO_NODE_MAPPING = Array<{
    node_hash: FELT;
    node: MERKLE_NODE;
}>;
/**
 * structured error that can later be processed by wallets or sdks.
 * error frame or the error raised during execution
 */
export type CONTRACT_EXECUTION_ERROR = CONTRACT_EXECUTION_ERROR_INNER;
/**
 * structured error that can later be processed by wallets or sdks.
 * error frame or the error raised during execution
 */
export type CONTRACT_EXECUTION_ERROR_INNER = {
    contract_address: ADDRESS;
    class_hash: FELT;
    selector: FELT;
    error: CONTRACT_EXECUTION_ERROR;
} | string;
/**
 * Represents a transaction trace including the execution details.
 */
export type TRANSACTION_TRACE = INVOKE_TXN_TRACE | DECLARE_TXN_TRACE | DEPLOY_ACCOUNT_TXN_TRACE | L1_HANDLER_TXN_TRACE;
/**
 * Represents a transaction trace for an invoke transaction.
 */
export type INVOKE_TXN_TRACE = {
    type: TXN_TYPE_INVOKE;
    execute_invocation: SimpleOneOf<FUNCTION_INVOCATION, {
        revert_reason: string;
    }>;
    validate_invocation?: FUNCTION_INVOCATION;
    fee_transfer_invocation?: FUNCTION_INVOCATION;
    state_diff?: STATE_DIFF;
    execution_resources: EXECUTION_RESOURCES;
};
/**
 * Represents a transaction trace for a declare transaction.
 */
export type DECLARE_TXN_TRACE = {
    type: TXN_TYPE_DECLARE;
    validate_invocation?: FUNCTION_INVOCATION;
    fee_transfer_invocation?: FUNCTION_INVOCATION;
    state_diff?: STATE_DIFF;
    execution_resources: EXECUTION_RESOURCES;
};
/**
 * Represents a transaction trace for a deploy account transaction.
 */
export type DEPLOY_ACCOUNT_TXN_TRACE = {
    type: TXN_TYPE_DEPLOY_ACCOUNT;
    constructor_invocation: FUNCTION_INVOCATION;
    validate_invocation?: FUNCTION_INVOCATION;
    fee_transfer_invocation?: FUNCTION_INVOCATION;
    state_diff?: STATE_DIFF;
    execution_resources: EXECUTION_RESOURCES;
};
/**
 * Represents a transaction trace for an L1 handler transaction.
 */
export type L1_HANDLER_TXN_TRACE = {
    type: Uppercase<ABI_TYPE_L1_HANDLER>;
    function_invocation: FUNCTION_INVOCATION;
    state_diff?: STATE_DIFF;
    execution_resources: EXECUTION_RESOURCES;
};
/**
 * Represents a nested function call.
 */
export type NESTED_CALL = FUNCTION_INVOCATION;
/**
 * Represents a function invocation along with its execution details.
 */
export type FUNCTION_INVOCATION = FUNCTION_CALL & {
    /**
     * The address of the invoking contract. 0 for the root invocation
     */
    caller_address: FELT;
    /**
     * The hash of the class being called
     */
    class_hash: FELT;
    entry_point_type: ENTRY_POINT_TYPE;
    call_type: CALL_TYPE;
    /**
     * The value returned from the function invocation
     */
    result: FELT[];
    /**
     * The calls made by this invocation
     */
    calls: NESTED_CALL[];
    /**
     * The events emitted in this invocation
     */
    events: ORDERED_EVENT[];
    /**
     * The messages sent by this invocation to L1
     */
    messages: ORDERED_MESSAGE[];
    /**
     * Resources consumed by the call tree rooted at this given call (including the root)
     */
    execution_resources: INNER_CALL_EXECUTION_RESOURCES;
    /**
     * true if this inner call panicked
     */
    is_reverted: boolean;
};
/**
 * the resources consumed by an inner call (does not account for state diffs since data is squashed across the transaction)
 */
export type INNER_CALL_EXECUTION_RESOURCES = {
    /**
     * l1 gas consumed by this transaction, used for l2-->l1 messages and state updates if blobs are not used
     */
    l1_gas: number;
    /**
     * l2 gas consumed by this transaction, used for computation and calldata
     */
    l2_gas: number;
};
/**
 * Represents an ordered event alongside its order within the transaction.
 */
export type ORDERED_EVENT = {
    order: number;
    event: EVENT;
};
/**
 * Represents an ordered message alongside its order within the transaction.
 */
export type ORDERED_MESSAGE = {
    order: number;
    message: MSG_TO_L1;
};
/**
 * Transaction status result, including finality status and execution status
 */
export type TXN_STATUS_RESULT = {
    finality_status: TXN_STATUS;
    execution_status?: TXN_EXECUTION_STATUS;
    /**
     * the failure reason, only appears if finality_status is REJECTED or execution_status is REVERTED
     */
    failure_reason?: string;
};
/**
 * (contract_address, storage_keys) pairs
 */
export type CONTRACT_STORAGE_KEYS = {
    contract_address: ADDRESS;
    storage_keys: FELT[];
};
export {};
//# sourceMappingURL=components.d.ts.map