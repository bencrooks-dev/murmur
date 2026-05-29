/* tslint:disable */
/* eslint-disable */
/**
* A local account, owned by the browser tab. Holds the key store, identity, and
* joined groups in memory (persistent storage is a later phase).
*/
export class WasmAccount {
  free(): void;
/**
* @param {Uint8Array} group_id
* @param {Uint8Array} key_package
* @returns {Uint8Array}
*/
  addMember(group_id: Uint8Array, key_package: Uint8Array): Uint8Array;
/**
* @param {Uint8Array} welcome
* @returns {Uint8Array}
*/
  joinGroup(welcome: Uint8Array): Uint8Array;
/**
* @returns {Uint8Array}
*/
  keyPackage(): Uint8Array;
/**
* @returns {Uint8Array}
*/
  createGroup(): Uint8Array;
/**
* @param {Uint8Array} group_id
* @returns {number}
*/
  memberCount(group_id: Uint8Array): number;
/**
* @param {Uint8Array} group_id
* @param {number} leaf_index
* @returns {Uint8Array}
*/
  removeMember(group_id: Uint8Array, leaf_index: number): Uint8Array;
/**
* @param {Uint8Array} group_id
* @param {string} label
* @param {number} length
* @returns {Uint8Array}
*/
  exporterSecret(group_id: Uint8Array, label: string, length: number): Uint8Array;
/**
* @param {string} name
*/
  constructor(name: string);
/**
* @param {Uint8Array} group_id
* @param {Uint8Array} plaintext
* @returns {Uint8Array}
*/
  send(group_id: Uint8Array, plaintext: Uint8Array): Uint8Array;
/**
* Returns the decrypted bytes for an application message, or `undefined` for
* handshake traffic.
* @param {Uint8Array} group_id
* @param {Uint8Array} message
* @returns {Uint8Array | undefined}
*/
  receive(group_id: Uint8Array, message: Uint8Array): Uint8Array | undefined;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmaccount_free: (a: number) => void;
  readonly wasmaccount_addMember: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasmaccount_createGroup: (a: number, b: number) => void;
  readonly wasmaccount_exporterSecret: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmaccount_joinGroup: (a: number, b: number, c: number, d: number) => void;
  readonly wasmaccount_keyPackage: (a: number, b: number) => void;
  readonly wasmaccount_memberCount: (a: number, b: number, c: number, d: number) => void;
  readonly wasmaccount_new: (a: number, b: number, c: number) => void;
  readonly wasmaccount_receive: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasmaccount_removeMember: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasmaccount_send: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
