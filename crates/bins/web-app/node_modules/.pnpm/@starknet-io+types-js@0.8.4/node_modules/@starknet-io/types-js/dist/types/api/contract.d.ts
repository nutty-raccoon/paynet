/**
 * 'starknet-specs' (OpenRpc protocol types)
 * https://github.com/starkware-libs/starknet-specs
 */
import type { ABI_NAME_AND_TYPE, ABI_TYPE, STATE_MUTABILITY } from './components.js';
import type { ABI_TYPE_CONSTRUCTOR, ABI_TYPE_ENUM, ABI_TYPE_FUNCTION, ABI_TYPE_L1_HANDLER, EVENT_ABI_TYPE, STRUCT_ABI_TYPE } from './constants.js';
import { SimpleOneOf } from './expansions/helpless.js';
/**
 * Cairo v>=2 Contract ABI
 */
export type ABI = Array<FUNCTION | CONSTRUCTOR | L1_HANDLER | EVENT | STRUCT | ENUM | INTERFACE | IMPL>;
export type FUNCTION = {
    type: ABI_TYPE_FUNCTION;
    /**
     * the function's name
     */
    name: string;
    /**
     * the arguments name and type.
     */
    inputs: ABI_NAME_AND_TYPE[];
    /**
     * the output type.
     */
    outputs: ABI_TYPE[];
    state_mutability: STATE_MUTABILITY;
};
export type CONSTRUCTOR = {
    type: ABI_TYPE_CONSTRUCTOR;
    name: ABI_TYPE_CONSTRUCTOR;
    inputs: ABI_NAME_AND_TYPE[];
};
export type L1_HANDLER = {
    type: ABI_TYPE_L1_HANDLER;
    name: string;
    inputs: ABI_NAME_AND_TYPE[];
    outputs: ABI_TYPE[];
    state_mutability: STATE_MUTABILITY;
};
export type EVENT = {
    type: EVENT_ABI_TYPE;
    /**
     * the name of the (Cairo) type associated with the event
     */
    name: string;
} & SimpleOneOf<ENUM_EVENT, STRUCT_EVENT>;
export type STRUCT_EVENT = {
    kind: STRUCT_ABI_TYPE;
    /**
     * struct members
     */
    members: EVENT_FIELD[];
};
export type ENUM_EVENT = {
    kind: ABI_TYPE_ENUM;
    /**
     * enum variants
     */
    variants: EVENT_FIELD[];
};
export type STRUCT = {
    type: STRUCT_ABI_TYPE;
    /**
     * the (Cairo) struct name, including namespacing
     */
    name: string;
    /**
     * name of the struct member.
     * type of the struct member, including namespacing.
     */
    members: ABI_NAME_AND_TYPE[];
};
export type ENUM = {
    type: ABI_TYPE_ENUM;
    /**
     * the (Cairo) enum name, including namespacing
     */
    name: string;
    /**
     * name of the enum variant.
     * type of the enum variant, including namespacing.
     */
    variants: ABI_NAME_AND_TYPE[];
};
export type INTERFACE = {
    type: 'interface';
    name: string;
    items: FUNCTION[];
};
export type IMPL = {
    type: 'impl';
    /**
     * the name of an impl containing contract entry points
     */
    name: string;
    /**
     * the name of the trait corresponding to this impl
     */
    interface_name: string;
};
export type EVENT_KIND = STRUCT_ABI_TYPE | ABI_TYPE_ENUM;
export type EVENT_FIELD = {
    /**
     * the name of the struct member or enum variant
     */
    name: string;
    /**
     * the Cairo type of the member or variant, including namespacing
     */
    type: string;
    kind: 'key' | 'data' | 'nested' | 'flat';
};
//# sourceMappingURL=contract.d.ts.map