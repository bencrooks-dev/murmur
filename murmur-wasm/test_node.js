// Runtime proof: drive the compiled Murmur WASM core from Node, exactly as the
// browser web client will. Two accounts, a real MLS group, an E2EE round-trip,
// matching exporter media keys, and member removal.
const assert = require("assert");
const { WasmAccount } = require("./pkg-node/murmur_wasm.js");

const enc = (s) => new TextEncoder().encode(s);
const dec = (b) => new TextDecoder().decode(b);
const eqBytes = (a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)) === 0;

const alice = new WasmAccount("alice");
const bob = new WasmAccount("bob");

const bobKp = bob.keyPackage();
const gid = alice.createGroup();
const welcome = alice.addMember(gid, bobKp);
const bobGid = bob.joinGroup(welcome);

assert.ok(eqBytes(gid, bobGid), "shared group id");
assert.strictEqual(alice.memberCount(gid), 2);
assert.strictEqual(bob.memberCount(bobGid), 2);

const ct = alice.send(gid, enc("hello from compiled WASM"));
const pt = bob.receive(bobGid, ct);
assert.strictEqual(dec(pt), "hello from compiled WASM");

const ka = alice.exporterSecret(gid, "murmur/media", 32);
const kb = bob.exporterSecret(bobGid, "murmur/media", 32);
assert.ok(eqBytes(ka, kb) && ka.length === 32, "matching media key");

alice.removeMember(gid, 1);
assert.strictEqual(alice.memberCount(gid), 1);

console.log("WASM E2EE round-trip OK:", dec(pt));
console.log("media key (hex):", Buffer.from(ka).toString("hex").slice(0, 24) + "...");
console.log("ALL WASM ASSERTIONS PASSED");
