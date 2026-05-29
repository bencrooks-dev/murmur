// Single-account Murmur client: owns one MLS account (WASM) plus a relay
// connection, and orchestrates the real handshake — publish key package, fetch a
// peer's, create the group, deliver the Welcome, then exchange ciphertext. All
// protocol logic stays in the Rust core; this just wires it to the network.
import init, { WasmAccount } from "../crypto/murmur_wasm.js";
import wasmUrl from "../crypto/murmur_wasm_bg.wasm?url";
import { RelayClient } from "./relay";

let readyPromise: Promise<unknown> | null = null;
export function ensureCryptoReady(): Promise<unknown> {
  if (!readyPromise) readyPromise = init(wasmUrl);
  return readyPromise;
}

function hex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

// Relay endpoint. Override with VITE_RELAY_URL at build time for public hosting
// (e.g. wss://relay.example.com/ws). Otherwise derive from the page origin:
// wss:// when served over https, ws:// for local/LAN dev on port 8787.
export const RELAY_URL: string =
  (import.meta.env.VITE_RELAY_URL as string | undefined) ||
  `${location.protocol === "https:" ? "wss" : "ws"}://${location.hostname}:8787/ws`;

export class MurmurClient {
  readonly user: string;
  private account: WasmAccount;
  private relay: RelayClient;
  private groupBytes = new Map<string, Uint8Array>();

  /** A message arrived from a peer on a group. */
  onMessage?: (group: string, text: string) => void;
  /** A channel became usable (we joined via a Welcome). */
  onChannelReady?: (group: string, peer: string) => void;

  constructor(user: string, relayUrl: string = RELAY_URL) {
    this.user = user;
    this.account = new WasmAccount(user);
    this.relay = new RelayClient(relayUrl);
    this.relay.onMessage = (g, body) => this.onCipher(g, body);
    this.relay.onWelcome = (from, body) => this.onWelcomeReceived(from, body);
  }

  async connect(): Promise<void> {
    await this.relay.connect();
    this.relay.register(this.user);
    this.relay.publishKp(this.user, this.account.keyPackage());
  }

  /** Begin a channel with `peer` (who must be online and have published a key package). */
  async startChat(peer: string): Promise<string> {
    const kp = await this.relay.fetchKp(peer);
    if (!kp) throw new Error(`"${peer}" is not online (no published key package)`);
    const gid = this.account.createGroup();
    const welcome = this.account.addMember(gid, kp);
    const ghex = hex(gid);
    this.groupBytes.set(ghex, gid);
    this.relay.sub(ghex);
    this.relay.welcome(peer, welcome);
    return ghex;
  }

  send(group: string, text: string) {
    const gid = this.groupBytes.get(group);
    if (!gid) return;
    const ct = this.account.send(gid, new TextEncoder().encode(text));
    this.relay.sendCipher(group, ct);
  }

  fingerprint(group: string): string {
    const gid = this.groupBytes.get(group);
    if (!gid) return "";
    const bytes = this.account.exporterSecret(gid, "murmur/fingerprint", 8);
    return hex(bytes).toUpperCase().replace(/(.{4})(?=.)/g, "$1 ");
  }

  private onWelcomeReceived(from: string, body: Uint8Array) {
    const gid = this.account.joinGroup(body);
    const ghex = hex(gid);
    this.groupBytes.set(ghex, gid);
    this.relay.sub(ghex);
    this.onChannelReady?.(ghex, from);
  }

  private onCipher(group: string, body: Uint8Array) {
    const gid = this.groupBytes.get(group);
    if (!gid) return;
    const pt = this.account.receive(gid, body);
    if (pt) this.onMessage?.(group, new TextDecoder().decode(pt));
  }
}
