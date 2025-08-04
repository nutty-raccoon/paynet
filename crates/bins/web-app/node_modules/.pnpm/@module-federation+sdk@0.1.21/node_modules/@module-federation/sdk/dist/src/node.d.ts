export declare function createScriptNode(url: string, cb: (error?: Error, scriptContext?: any) => void, attrs?: Record<string, any>, createScriptHook?: (url: string) => any | void): void;
export declare function loadScriptNode(url: string, info: {
    attrs?: Record<string, any>;
    createScriptHook?: (url: string) => void;
}): Promise<void>;
