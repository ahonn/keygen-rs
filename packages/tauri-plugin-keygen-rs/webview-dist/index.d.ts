export interface KeygenLicense {
    id: string;
    key: string;
    name: string;
    expiry: string;
    status: string;
    policy: string;
    valid: boolean;
    metadata?: Record<string, any>;
}
export declare class KeygenError extends Error {
    code: string;
    detail: string;
    constructor(code: string, detail: string);
}
export declare function getLicenseKey(): Promise<string>;
export declare function getLicense(): Promise<KeygenLicense>;
export declare function validateKey(key: string, entitlements?: string[]): Promise<KeygenLicense>;
export declare function deactivate(): Promise<void>;
export declare function checkoutLicense(ttl?: number, include?: string[]): Promise<void>;
export declare function checkoutMachine(ttl?: number, include?: string[]): Promise<void>;
export declare function resetLicense(): Promise<void>;
export declare function getLicenseMetadata(): Promise<Record<string, any> | null>;
