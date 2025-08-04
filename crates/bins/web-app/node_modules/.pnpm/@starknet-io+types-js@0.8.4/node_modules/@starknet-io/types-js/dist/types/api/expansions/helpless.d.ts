type MergeTypes<TypesArray extends any[], Res = {}> = TypesArray extends [infer Head, ...infer Rem] ? MergeTypes<Rem, Res & Head> : Res;
type OnlyFirst<F, S> = F & {
    [Key in keyof Omit<S, keyof F>]?: undefined;
};
/**
 * Exclusive one of types (no intersection allowed)
 *  OneOf<[A, B]> = exclusive A OR B, where (all props of A) OR (all props of B)
 *  A | B = inclusive A OR B, where all prop of A and any prop of B  and vice versa
 */
export type OneOf<TypesArray extends any[], Res = never, AllProperties = MergeTypes<TypesArray>> = TypesArray extends [infer Head, ...infer Rem] ? OneOf<Rem, Res | OnlyFirst<Head, AllProperties>, AllProperties> : Res;
export type SimpleOneOf<F, S> = OnlyFirst<F, S> | OnlyFirst<S, F>;
export {};
/**
 * Search in spec oneOf
 * go one by one and fix one missing, but before there need to be solution for it
 *
 * current ts oneOf produce [key: string]: undefined; [key: number]: undefined; artifacts
 */
//# sourceMappingURL=helpless.d.ts.map