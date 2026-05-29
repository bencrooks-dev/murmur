// End-to-end proof of the full networked handshake against the LIVE relay:
// two independent clients register, publish/fetch key packages, exchange a
// Welcome, and send encrypted messages both ways — exactly the web client flow,
// using the real WASM core + Node's global WebSocket. Relay must be running on :8787.
const assert = require("assert");
const { WasmAccount } = require("./pkg-node/murmur_wasm.js");

const URL = "ws://127.0.0.1:8787/ws";
const b64 = (u8) => Buffer.from(u8).toString("base64");
const unb64 = (s) => new Uint8Array(Buffer.from(s, "base64"));
const hex = (u8) => Buffer.from(u8).toString("hex");
const enc = (s) => new TextEncoder().encode(s);
const dec = (b) => new TextDecoder().decode(b);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

class Client {
  constructor(user) {
    this.user = user;
    this.acc = new WasmAccount(user);
    this.groups = new Map();
    this.kpWaiters = new Map();
    this.onMessage = null;
    this.onWelcome = null;
  }
  connect() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(URL);
      this.ws.addEventListener("open", () => {
        this.send({ op: "register", user: this.user });
        this.send({ op: "publishkp", user: this.user, kp: b64(this.acc.keyPackage()) });
        resolve();
      });
      this.ws.addEventListener("error", reject);
      this.ws.addEventListener("message", (e) => this.handle(JSON.parse(e.data)));
    });
  }
  send(o) { this.ws.send(JSON.stringify(o)); }
  fetchKp(user) {
    return new Promise((r) => { this.kpWaiters.set(user, r); this.send({ op: "fetchkp", user }); });
  }
  async startChat(peer) {
    const kp = await this.fetchKp(peer);
    if (!kp) throw new Error(`${peer}: no key package`);
    const gid = this.acc.createGroup();
    const welcome = this.acc.addMember(gid, kp);
    const gh = hex(gid);
    this.groups.set(gh, gid);
    this.send({ op: "sub", group: gh });
    this.send({ op: "welcome", to: peer, body: b64(welcome) });
    return gh;
  }
  sendMsg(gh, text) {
    const ct = this.acc.send(this.groups.get(gh), enc(text));
    this.send({ op: "send", group: gh, body: b64(ct) });
  }
  handle(m) {
    if (m.evt === "kp") {
      const w = this.kpWaiters.get(m.user);
      if (w) { this.kpWaiters.delete(m.user); w(m.kp ? unb64(m.kp) : null); }
    } else if (m.evt === "welcome") {
      const gid = this.acc.joinGroup(unb64(m.body));
      const gh = hex(gid);
      this.groups.set(gh, gid);
      this.send({ op: "sub", group: gh });
      this.onWelcome && this.onWelcome(m.from, gh);
    } else if (m.evt === "msg") {
      const gid = this.groups.get(m.group);
      if (gid) {
        const pt = this.acc.receive(gid, unb64(m.body));
        if (pt) this.onMessage && this.onMessage(m.group, dec(pt));
      }
    }
  }
}

(async () => {
  const received = [];
  const alice = new Client("alice");
  const bob = new Client("bob");
  alice.onMessage = (_g, t) => received.push(["alice", t]);
  bob.onMessage = (_g, t) => received.push(["bob", t]);
  let bobGroup = null;
  bob.onWelcome = (_from, gh) => { bobGroup = gh; };

  await bob.connect();
  await alice.connect();
  await sleep(150);

  const gh = await alice.startChat("bob");
  await sleep(250); // welcome delivered, bob subscribes

  alice.sendMsg(gh, "hello bob, over the wire");
  await sleep(250);
  assert.ok(bobGroup, "bob joined a group");
  bob.sendMsg(bobGroup, "hi alice, decrypted fine");
  await sleep(250);

  assert.deepStrictEqual(
    received.find((r) => r[0] === "bob"),
    ["bob", "hello bob, over the wire"],
  );
  assert.deepStrictEqual(
    received.find((r) => r[0] === "alice"),
    ["alice", "hi alice, decrypted fine"],
  );

  console.log("NETWORKED E2EE HANDSHAKE OK:");
  received.forEach(([who, t]) => console.log(`  ${who} received: "${t}"`));
  console.log("ALL E2E ASSERTIONS PASSED");
  process.exit(0);
})().catch((e) => { console.error("E2E FAILED:", e.message); process.exit(1); });
