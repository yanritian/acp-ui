// ACP Protocol — encryption utilities and message helpers.
//
// The canonical ACPMessage type is defined in `./spec.ts` (JSON-RPC 2.0 format).
// This module re-exports it and adds AES-GCM encryption helpers.

import type { ACPMessage } from './spec';
export type { ACPMessage } from './spec';

// Re-export spec.ts constants and helpers for convenience
export {
  ACP_METHODS,
  ACP_ERROR_CODES,
  DEFAULT_ACP_CAPABILITIES,
  negotiateCapabilities,
} from './spec';
export type { ACPCapabilities, ACPFeature, ACPError } from './spec';

// ---------- Encryption utilities (AES-GCM) ----------

const ALGORITHM = 'AES-GCM';
const IV_LENGTH = 12; // 96 bits recommended for AES-GCM

/**
 * Import a raw AES-GCM key from a Uint8Array.
 *
 * The key material must be exactly 32 bytes (256 bits).
 */
async function importKey(key: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    key,
    { name: ALGORITHM, length: 256 },
    false,
    ['encrypt', 'decrypt']
  );
}

function generateIV(): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(IV_LENGTH));
}

/**
 * Convert Uint8Array to base64 safely (avoids stack overflow on large payloads).
 */
function uint8ToBase64(data: Uint8Array): string {
  let binary = '';
  const chunkSize = 8192;
  for (let i = 0; i < data.length; i += chunkSize) {
    const chunk = data.subarray(i, i + chunkSize);
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary);
}

/**
 * Convert base64 string to Uint8Array.
 */
function base64ToUint8Array(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

/**
 * Encrypt the message params field using AES-GCM.
 *
 * The returned message has the `params` field replaced with
 * `{ iv: string (base64), ciphertext: string (base64) }` and
 * the encryption metadata set.
 */
export async function encryptMessage(
  msg: ACPMessage,
  key: Uint8Array
): Promise<ACPMessage> {
  const cryptoKey = await importKey(key);
  const iv = generateIV();

  const plaintext = new TextEncoder().encode(
    JSON.stringify(msg.params ?? msg.result ?? {})
  );

  const ciphertext = await crypto.subtle.encrypt(
    { name: ALGORITHM, iv },
    cryptoKey,
    plaintext
  );

  return {
    ...msg,
    params: {
      iv: uint8ToBase64(iv),
      ciphertext: uint8ToBase64(new Uint8Array(ciphertext)),
    },
    encryption: {
      algorithm: 'aes-256-gcm',
      keyId: 'default',
      iv: uint8ToBase64(iv),
    },
  };
}

/**
 * Decrypt an encrypted message.
 *
 * Expects `msg.params` to contain `{ iv: string, ciphertext: string }` in
 * base64 encoding, and `msg.encryption` to be set.
 */
export async function decryptMessage(
  msg: ACPMessage,
  key: Uint8Array
): Promise<ACPMessage> {
  if (!msg.encryption) {
    return msg;
  }

  const cryptoKey = await importKey(key);

  const payload = msg.params as { iv?: string; ciphertext?: string } | undefined;
  const iv = payload?.iv ? base64ToUint8Array(payload.iv) : null;
  const ciphertext = payload?.ciphertext
    ? base64ToUint8Array(payload.ciphertext)
    : null;

  if (!iv || !ciphertext) {
    throw new Error('Invalid encrypted payload: missing iv or ciphertext');
  }

  const decrypted = await crypto.subtle.decrypt(
    { name: ALGORITHM, iv },
    cryptoKey,
    ciphertext
  );

  const plaintext = new TextDecoder().decode(decrypted);

  return {
    ...msg,
    params: JSON.parse(plaintext) as Record<string, unknown>,
    encryption: undefined,
  };
}

// ---------- Message helpers ----------

/**
 * Create a JSON-RPC 2.0 request-style ACPMessage.
 */
export function createACPMessage(
  method: string,
  params: Record<string, unknown>,
  options?: { id?: string | number; sessionId?: string; source?: 'client' | 'server' | 'agent' }
): ACPMessage {
  return {
    jsonrpc: '2.0',
    id: options?.id ?? crypto.randomUUID(),
    method,
    params,
    sessionId: options?.sessionId ?? '',
    timestamp: Date.now(),
    source: options?.source ?? 'client',
  };
}

/** Generate a 256-bit AES key for encryption/decryption. */
export async function generateEncryptionKey(): Promise<Uint8Array> {
  const key = await crypto.subtle.generateKey(
    { name: ALGORITHM, length: 256 },
    true,
    ['encrypt', 'decrypt']
  );

  const raw = await crypto.subtle.exportKey('raw', key);
  return new Uint8Array(raw);
}
