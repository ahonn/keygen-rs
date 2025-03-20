import { invoke } from '@tauri-apps/api/core';

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

export async function getLicense(): Promise<KeygenLicense> {
  try {
    const [license, valid] = await Promise.all([
      invoke<Omit<KeygenLicense, 'valid'>>('plugin:keygen-rs2|get_license'),
      invoke<boolean>('plugin:keygen-rs2|is_license_valid'),
    ]);
    return createKeygenLicense(license, valid);
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

export async function checkoutLicense(ttl?: number, include?: string[]) {
  try {
    await invoke('plugin:keygen-rs2|checkout_license', {
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

