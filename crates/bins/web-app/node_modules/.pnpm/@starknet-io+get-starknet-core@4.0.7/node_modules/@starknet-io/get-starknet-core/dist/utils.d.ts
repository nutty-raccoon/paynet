/**
 * @see https://github.com/GoogleChrome/web-vitals/blob/main/src/lib/generateUniqueID.ts
 */
export declare const generateUID: () => string;
export declare const shuffle: <T extends any[]>(arr: T) => T;
type AllowPromise<T> = Promise<T> | T;
export declare const pipe: <T>(...fns: ((arg: T) => AllowPromise<T>)[]) => (arg: T) => Promise<T>;
export declare function ensureKeysArray<T extends object>(keysGuard: {
    [k in keyof T]: true;
}): (keyof T)[];
export declare const ssrSafeWindow: Window | null;
export {};
