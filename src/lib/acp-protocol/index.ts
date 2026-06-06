// ACP (Agent Control Protocol) - standard message format and encryption utilities.

export type ACPMessageType =
  | 'task_submit'
  | 'task_progress'
  | 'task_complete'
  | 'task_error'
  | 'approval_request'
  | 'approval_response'
  | 'agent_register'
  | 'agent_deregister'
  | 'agent_status'
  | 'sync_event'
  | 'heartbeat';

export interface ACPMessage {
  version: '1.0';
  messageId: string;
  timestamp: number;
  type: ACPMessageType;
  payload: Record<string, unknown>;
  encrypted?: boolean;
}

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
 * Encrypt the message payload in-place.
 *
 * The returned message has `encrypted = true` and the payload field replaced
 * with `{ iv: string (base64), ciphertext: string (base64) }`.
 */
export async function encryptMessage(
  msg: ACPMessage,
  key: Uint8Array
): Promise<ACPMessage> {
  const cryptoKey = await importKey(key);
  const iv = generateIV();

  const plaintext = new TextEncoder().encode(JSON.stringify(msg.payload));

  const ciphertext = await crypto.subtle.encrypt(
    { name: ALGORITHM, iv },
    cryptoKey,
    plaintext
  );

  return {
    ...msg,
    payload: {
      iv: btoa(String.fromCharCode(...iv)),
      ciphertext: btoa(String.fromCharCode(...new Uint8Array(ciphertext))),
    },
    encrypted: true,
  };
}

/**
 * Decrypt an encrypted message.
 *
 * Expects `msg.payload` to contain `{ iv: string, ciphertext: string }` in
 * base64 encoding, and `msg.encrypted === true`.
 */
export async function decryptMessage(
  msg: ACPMessage,
  key: Uint8Array
): Promise<ACPMessage> {
  if (!msg.encrypted) {
    return msg; // Already decrypted / never encrypted.
  }

  const cryptoKey = await importKey(key);

  const payload = msg.payload as { iv?: string; ciphertext?: string };
  const iv = payload.iv ? Uint8Array.from(atob(payload.iv), (c) => c.charCodeAt(0)) : null;
  const ciphertext = payload.ciphertext
    ? Uint8Array.from(atob(payload.ciphertext), (c) => c.charCodeAt(0))
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
    payload: JSON.parse(plaintext) as Record<string, unknown>,
    encrypted: false,
  };
}

// ---------- Message helpers ----------

/** Create a new ACPMessage with auto-generated ID and timestamp. */
export function createACPMessage(
  type: ACPMessageType,
  payload: Record<string, unknown>
): ACPMessage {
  return {
    version: '1.0',
    messageId: crypto.randomUUID(),
    timestamp: Date.now(),
    type,
    payload,
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
