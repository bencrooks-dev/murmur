// WebSocket client for the Murmur relay. Speaks the JSON control protocol and
// carries opaque bytes (key packages, Welcomes, ciphertext) as base64. The relay
// never sees plaintext; this layer never sees keys.

export function b64(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s);
}
export function unb64(str: string): Uint8Array {
  const s = atob(str);
  const a = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) a[i] = s.charCodeAt(i);
  return a;
}

export class RelayClient {
  private ws: WebSocket;
  private ready: Promise<void>;
  private kpWaiters = new Map<string, (kp: Uint8Array | null) => void>();

  onMessage?: (group: string, body: Uint8Array) => void;
  onWelcome?: (from: string, body: Uint8Array) => void;

  constructor(url: string) {
    this.ws = new WebSocket(url);
    this.ready = new Promise((resolve, reject) => {
      this.ws.onopen = () => resolve();
      this.ws.onerror = () => reject(new Error("relay connection failed"));
    });
    this.ws.onmessage = (e) => this.handle(JSON.parse(e.data as string));
  }

  connect(): Promise<void> {
    return this.ready;
  }

  private send(obj: unknown) {
    this.ws.send(JSON.stringify(obj));
  }

  register(user: string) {
    this.send({ op: "register", user });
  }
  publishKp(user: string, kp: Uint8Array) {
    this.send({ op: "publishkp", user, kp: b64(kp) });
  }
  fetchKp(user: string): Promise<Uint8Array | null> {
    return new Promise((resolve) => {
      this.kpWaiters.set(user, resolve);
      this.send({ op: "fetchkp", user });
    });
  }
  welcome(to: string, body: Uint8Array) {
    this.send({ op: "welcome", to, body: b64(body) });
  }
  sub(group: string) {
    this.send({ op: "sub", group });
  }
  sendCipher(group: string, body: Uint8Array) {
    this.send({ op: "send", group, body: b64(body) });
  }

  private handle(m: Record<string, string>) {
    switch (m.evt) {
      case "kp": {
        const w = this.kpWaiters.get(m.user);
        if (w) {
          this.kpWaiters.delete(m.user);
          w(m.kp ? unb64(m.kp) : null);
        }
        break;
      }
      case "welcome":
        this.onWelcome?.(m.from, unb64(m.body));
        break;
      case "msg":
        this.onMessage?.(m.group, unb64(m.body));
        break;
    }
  }
}
