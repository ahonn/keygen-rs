import { invoke } from '@tauri-apps/api/core';

export interface KeygenLicense {
  id: string;
  key: string;
  scheme?: string | null;
  name?: string | null;
  expiry?: string | null;
  status?: string | null;
  uses?: number | null;
  version?: string | null;
  floating?: boolean | null;
  encrypted?: boolean | null;
  strict?: boolean | null;
  max_machines?: number | null;
  max_processes?: number | null;
  max_users?: number | null;
  max_cores?: number | null;
  max_memory?: number | null;
  max_disk?: number | null;
  max_uses?: number | null;
  protected?: boolean | null;
  suspended?: boolean | null;
  require_heartbeat?: boolean | null;
  require_check_in?: boolean | null;
  last_validated?: string | null;
  last_check_out?: string | null;
  last_check_in?: string | null;
  next_check_in?: string | null;
  permissions?: string[] | null;
  policy?: string | null;
  valid: boolean;
  metadata: Record<string, unknown>;
  account_id?: string | null;
  product_id?: string | null;
  group_id?: string | null;
  owner_id?: string | null;
  environment_id?: string | null;
  created?: string | null;
  updated?: string | null;
}

export type LicenseFileAlgorithm =
  | 'aes-256-gcm+ed25519'
  | 'aes-256-gcm+ecdsa-p256'
  | 'aes-256-gcm+rsa-pss-sha256'
  | 'aes-256-gcm+rsa-sha256'
  | 'base64+ed25519'
  | 'base64+ecdsa-p256'
  | 'base64+rsa-pss-sha256'
  | 'base64+rsa-sha256';

export interface LicenseCheckoutOptions {
  ttl?: number | null;
  include?: string[];
  encrypt?: boolean;
  algorithm?: LicenseFileAlgorithm;
}

interface InvokeError {
  code: string;
  detail: string;
}

export class KeygenError extends Error {
  constructor(
    public code: string,
    public detail: string,
  ) {
    super(`Keygen error: ${code} - ${detail}`);
    this.name;
  }
}

const NO_MACHINE_ERROR_CODES = ['NO_MACHINE', 'NO_MACHINES', 'FINGERPRINT_SCOPE_MISMATCH'];

function isInvokeError(err: any): err is InvokeError {
  return typeof err === 'object' && err?.hasOwnProperty('code');
}

function createKeygenLicense(license: Omit<KeygenLicense, 'valid'>, valid: boolean) {
  return {
    ...license,
    valid,
  } as KeygenLicense;
}

export async function getLicenseKey(): Promise<string> {
  try {
    const key = await invoke('plugin:keygen-rs2|get_license_key');
    return key as string;
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function getLicense(): Promise<KeygenLicense | null> {
  try {
    const [license, valid] = await Promise.all([
      invoke<Omit<KeygenLicense, 'valid'> | null>('plugin:keygen-rs2|get_license'),
      invoke<boolean>('plugin:keygen-rs2|is_license_valid'),
    ]);
    return license ? createKeygenLicense(license, valid) : null;
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function validateKey(key: string, entitlements?: string[]): Promise<KeygenLicense> {
  try {
    const license = await invoke<Omit<KeygenLicense, 'valid'>>('plugin:keygen-rs2|validate_key', {
      key,
      entitlements,
    });
    return createKeygenLicense(license, true);
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      const noMachineError = NO_MACHINE_ERROR_CODES.includes(code);
      if (!noMachineError) {
        throw new KeygenError(code, detail);
      }

      await invoke('plugin:keygen-rs2|activate', {});
      const license = await invoke<Omit<KeygenLicense, 'valid'>>('plugin:keygen-rs2|validate_key', {
        key,
        entitlements,
      });
      return createKeygenLicense(license, true);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function deactivate() {
  try {
    await invoke('plugin:keygen-rs2|deactivate');
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function checkoutLicense(options: LicenseCheckoutOptions = {}) {
  try {
    const { ttl, ...rest } = options;
    return await invoke('plugin:keygen-rs2|checkout_license', {
      ...rest,
      ttl: typeof ttl === 'number' ? ttl : undefined,
      perpetual: ttl === null,
    });
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function checkoutMachine(ttl?: number, include?: string[]) {
  try {
    await invoke('plugin:keygen-rs2|checkout_machine', {
      ttl,
      include,
    });
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function resetLicense() {
  try {
    await invoke('plugin:keygen-rs2|reset_license');
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}

export async function getLicenseMetadata(): Promise<Record<string, any> | null> {
  try {
    const metadata = await invoke<Record<string, any> | null>('plugin:keygen-rs2|get_license_metadata');
    return metadata;
  } catch (err) {
    if (isInvokeError(err)) {
      const { code, detail } = err;
      throw new KeygenError(code, detail);
    }
    throw new KeygenError('ERROR', (err as Error).message);
  }
}
