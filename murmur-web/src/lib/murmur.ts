// Thin TypeScript surface over the Murmur WASM crypto core. The web client never
// touches MLS directly — it drives `WasmAccount`, so web and mobile share one
// implementation. All ciphertext crosses as `Uint8Array`.
import init, { WasmAccount } from "../crypto/murmur_wasm.js";
import wasmUrl from "../crypto/murmur_wasm_bg.wasm?url";

let readyPromise: Promise<unknown> | null = null;

/** Initialize the WASM module exactly once. Must resolve before constructing a channel. */
export function ensureCryptoReady(): Promise<unknown> {
  if (!readyPromise) readyPromise = init(wasmUrl);
  return readyPromise;
}

export interface DeliveryReceipt {
  /** The opaque MLS ciphertext that left this client. */
  ciphertext: Uint8Array;
  /** True if the peer successfully decrypted — proof of a real E2EE round-trip. */
  delivered: boolean;
}

/**
 * A demo of a real end-to-end-encrypted MLS channel between two local accounts.
 * Sending encrypts under the sender's group; the peer decrypts to confirm a true
 * round-trip. Both halves run the same audited core — nothing here is faked.
 */
export class SecureChannel {
  private readonly me: WasmAccount;
  private readonly peer: WasmAccount;
  private readonly gid: Uint8Array;
  private readonly peerGid: Uint8Array;

  constructor(myName: string, peerName: string) {
    this.me = new WasmAccount(myName);
    this.peer = new WasmAccount(peerName);
    const peerKeyPackage = this.peer.keyPackage();
    this.gid = this.me.createGroup();
    const welcome = this.me.addMember(this.gid, peerKeyPackage);
    this.peerGid = this.peer.joinGroup(welcome);
  }

  send(text: string): DeliveryReceipt {
    const plaintext = new TextEncoder().encode(text);
    const ciphertext = this.me.send(this.gid, plaintext);
    const received = this.peer.receive(this.peerGid, ciphertext);
    const delivered =
      received != null && new TextDecoder().decode(received) === text;
    return { ciphertext, delivered };
  }

  /** A human-verifiable channel fingerprint derived from the MLS exporter secret. */
  fingerprint(): string {
    const bytes = this.me.exporterSecret(this.gid, "murmur/fingerprint", 8);
    return Array.from(bytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
      .toUpperCase()
      .replace(/(.{4})(?=.)/g, "$1 ");
  }

  memberCount(): number {
    return this.me.memberCount(this.gid);
  }
}

export function toHex(bytes: Uint8Array, max = 48): string {
  const slice = Array.from(bytes.slice(0, max))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join(" ");
  return bytes.length > max ? `${slice} …` : slice;
}
