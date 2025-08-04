export declare function normalizeOptions<T>(enableDefault: boolean, defaultOptions: T, key: string): <U extends boolean | T | undefined>(options: U) => T | false;
