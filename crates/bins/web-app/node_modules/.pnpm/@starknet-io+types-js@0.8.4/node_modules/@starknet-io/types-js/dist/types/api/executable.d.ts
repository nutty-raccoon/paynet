import type { FELT, NUM_AS_HEX } from './components.js';
/**
 * Starknet get compiled CASM result
 */
export type CASM_COMPILED_CONTRACT_CLASS = {
    entry_points_by_type: {
        CONSTRUCTOR: CASM_ENTRY_POINT[];
        EXTERNAL: CASM_ENTRY_POINT[];
        L1_HANDLER: CASM_ENTRY_POINT[];
    };
    bytecode: FELT[];
    prime: NUM_AS_HEX;
    compiler_version: string;
    /**
     * Array of 2-tuple of pc value and an array of hints to execute.
     */
    hints: Array<[number | Array<HINT>, number | Array<HINT>]>;
    /**
     * a list of sizes of segments in the bytecode, each segment is hashed individually when computing the bytecode hash.
     * Integer
     */
    bytecode_segment_lengths?: number;
};
export type CASM_ENTRY_POINT = {
    offset: number;
    selector: FELT;
    builtins: string[];
};
export type CellRef = {
    register: 'AP' | 'FP';
    offset: number;
};
export type Deref = {
    Deref: CellRef;
};
export type DoubleDeref = {
    /**
     * A (CellRef, offset) tuple
     */
    DoubleDeref: [CellRef | number];
};
export type Immediate = {
    Immediate: NUM_AS_HEX;
};
export type BinOp = {
    BinOp: {
        op: 'Add' | 'Mul';
        a: CellRef;
        b: Deref | Immediate;
    };
};
export type ResOperand = Deref | DoubleDeref | Immediate | BinOp;
export type HINT = DEPRECATED_HINT | CORE_HINT | STARKNET_HINT;
export type DEPRECATED_HINT = 'AssertCurrentAccessIndicesIsEmpty' | 'AssertAllKeysUsed' | 'AssertLeAssertThirdArcExcluded' | {
    AssertAllAccessesUsed: {
        n_used_accesses: CellRef;
    };
} | {
    AssertLtAssertValidInput: {
        a: ResOperand;
        b: ResOperand;
    };
} | {
    Felt252DictRead: {
        dict_ptr: ResOperand;
        key: ResOperand;
        value_dst: CellRef;
    };
} | {
    Felt252DictWrite: {
        dict_ptr: ResOperand;
        key: ResOperand;
        value: ResOperand;
    };
};
export type CORE_HINT = {
    AllocSegment: {
        dst: CellRef;
    };
} | {
    TestLessThan: {
        lhs: ResOperand;
        rhs: ResOperand;
        dst: CellRef;
    };
} | {
    TestLessThanOrEqual: {
        lhs: ResOperand;
        rhs: ResOperand;
        dst: CellRef;
    };
} | {
    TestLessThanOrEqualAddress: {
        lhs: ResOperand;
        rhs: ResOperand;
        dst: CellRef;
    };
} | {
    WideMul128: {
        lhs: ResOperand;
        rhs: ResOperand;
        high: CellRef;
        low: CellRef;
    };
} | {
    DivMod: {
        lhs: ResOperand;
        rhs: ResOperand;
        quotient: CellRef;
        remainder: CellRef;
    };
} | {
    Uint256DivMod: {
        dividend0: ResOperand;
        dividend1: ResOperand;
        divisor0: ResOperand;
        divisor1: ResOperand;
        quotient0: CellRef;
        quotient1: CellRef;
        remainder0: CellRef;
        remainder1: CellRef;
    };
} | {
    Uint512DivModByUint256: {
        dividend0: ResOperand;
        dividend1: ResOperand;
        dividend2: ResOperand;
        dividend3: ResOperand;
        divisor0: ResOperand;
        divisor1: ResOperand;
        quotient0: CellRef;
        quotient1: CellRef;
        quotient2: CellRef;
        quotient3: CellRef;
        remainder0: CellRef;
        remainder1: CellRef;
    };
} | {
    SquareRoot: {
        value: ResOperand;
        dst: CellRef;
    };
} | {
    Uint256SquareRoot: {
        value_low: ResOperand;
        value_high: ResOperand;
        sqrt0: CellRef;
        sqrt1: CellRef;
        remainder_low: CellRef;
        remainder_high: CellRef;
        sqrt_mul_2_minus_remainder_ge_u128: CellRef;
    };
} | {
    LinearSplit: {
        value: ResOperand;
        scalar: ResOperand;
        max_x: ResOperand;
        x: CellRef;
        y: CellRef;
    };
} | {
    AllocFelt252Dict: {
        segment_arena_ptr: ResOperand;
    };
} | {
    Felt252DictEntryInit: {
        dict_ptr: ResOperand;
        key: ResOperand;
    };
} | {
    Felt252DictEntryUpdate: {
        dict_ptr: ResOperand;
        value: ResOperand;
    };
} | {
    GetSegmentArenaIndex: {
        dict_end_ptr: ResOperand;
        dict_index: CellRef;
    };
} | {
    InitSquashData: {
        dict_accesses: ResOperand;
        ptr_diff: ResOperand;
        n_accesses: ResOperand;
        big_keys: CellRef;
        first_key: CellRef;
    };
} | {
    GetCurrentAccessIndex: {
        range_check_ptr: ResOperand;
    };
} | {
    ShouldSkipSquashLoop: {
        should_skip_loop: CellRef;
    };
} | {
    GetCurrentAccessDelta: {
        index_delta_minus1: CellRef;
    };
} | {
    ShouldContinueSquashLoop: {
        should_continue: CellRef;
    };
} | {
    GetNextDictKey: {
        next_key: CellRef;
    };
} | {
    AssertLeFindSmallArcs: {
        range_check_ptr: ResOperand;
        a: ResOperand;
        b: ResOperand;
    };
} | {
    AssertLeIsFirstArcExcluded: {
        skip_exclude_a_flag: CellRef;
    };
} | {
    AssertLeIsSecondArcExcluded: {
        skip_exclude_b_minus_a: CellRef;
    };
} | {
    RandomEcPoint: {
        x: CellRef;
        y: CellRef;
    };
} | {
    FieldSqrt: {
        val: ResOperand;
        sqrt: CellRef;
    };
} | {
    DebugPrint: {
        start: ResOperand;
        end: ResOperand;
    };
} | {
    AllocConstantSize: {
        size: ResOperand;
        dst: CellRef;
    };
} | {
    U256InvModN: {
        b0: ResOperand;
        b1: ResOperand;
        n0: ResOperand;
        n1: ResOperand;
        g0_or_no_inv: CellRef;
        g1_option: CellRef;
        s_or_r0: CellRef;
        s_or_r1: CellRef;
        t_or_k0: CellRef;
        t_or_k1: CellRef;
    };
} | {
    EvalCircuit: {
        n_add_mods: ResOperand;
        add_mod_builtin: ResOperand;
        n_mul_mods: ResOperand;
        mul_mod_builtin: ResOperand;
    };
};
export type STARKNET_HINT = {
    SystemCall: {
        system: ResOperand;
    };
} | {
    Cheatcode: {
        selector: NUM_AS_HEX;
        input_start: ResOperand;
        input_end: ResOperand;
        output_start: CellRef;
        output_end: CellRef;
    };
};
//# sourceMappingURL=executable.d.ts.map