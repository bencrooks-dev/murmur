import { useEffect, useMemo, useRef, useState } from "react";
import { ensureCryptoReady, MurmurClient } from "./lib/murmur";

/* Iconography: one set, consistent 1.5px stroke (see design language). */
const Icon = ({ path, size = 16 }: { path: string; size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor"
    strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
    <path d={path} />
  </svg>
);
const LOCK = "M6 11V8a6 6 0 1 1 12 0v3M5 11h14a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1Z";
const SEND = "M7 11 12 6l5 5M12 6v13";
const AT = "M16 12a4 4 0 1 0-8 0 4 4 0 0 0 8 0Zm0 0v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.6 7.2";
const PLUS = "M12 5v14M5 12h14";

interface Message { id: number; mine: boolean; author: string; text: string; time: string; }
interface Channel { group: string; peer: string; }

const now = () => new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

export default function App() {
  const [user, setUser] = useState("");
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [channels, setChannels] = useState<Channel[]>([]);
  const [active, setActive] = useState<string | null>(null);
  const [byGroup, setByGroup] = useState<Record<string, Message[]>>({});
  const [peerInput, setPeerInput] = useState("");
  const [draft, setDraft] = useState("");

  const clientRef = useRef<MurmurClient | null>(null);
  const channelsRef = useRef<Channel[]>([]);
  const threadRef = useRef<HTMLDivElement>(null);
  const nextId = useRef(1);

  useEffect(() => {
    channelsRef.current = channels;
  }, [channels]);

  useEffect(() => {
    threadRef.current?.scrollTo({ top: threadRef.current.scrollHeight });
  }, [byGroup, active]);

  function pushMessage(group: string, m: Omit<Message, "id">) {
    setByGroup((prev) => ({
      ...prev,
      [group]: [...(prev[group] ?? []), { ...m, id: nextId.current++ }],
    }));
  }

  async function connect() {
    const handle = user.trim();
    if (!handle) return;
    setConnecting(true);
    setError(null);
    try {
      await ensureCryptoReady();
      const client = new MurmurClient(handle);
      client.onChannelReady = (group, peer) => {
        setChannels((cs) => (cs.some((c) => c.group === group) ? cs : [...cs, { group, peer }]));
        setActive((a) => a ?? group);
      };
      client.onMessage = (group, text) => {
        const peer = channelPeer(group);
        pushMessage(group, { mine: false, author: peer, text, time: now() });
      };
      await client.connect();
      clientRef.current = client;
      setConnected(true);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setConnecting(false);
    }
  }

  function channelPeer(group: string): string {
    return channelsRef.current.find((c) => c.group === group)?.peer ?? "peer";
  }

  async function startChat() {
    const peer = peerInput.trim();
    if (!peer || !clientRef.current) return;
    setError(null);
    try {
      const group = await clientRef.current.startChat(peer);
      setChannels((cs) => (cs.some((c) => c.group === group) ? cs : [...cs, { group, peer }]));
      setActive(group);
      setPeerInput("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  function send() {
    const text = draft.trim();
    if (!text || !active || !clientRef.current) return;
    clientRef.current.send(active, text);
    pushMessage(active, { mine: true, author: user, text, time: now() });
    setDraft("");
  }

  const fingerprint = useMemo(
    () => (active && clientRef.current ? clientRef.current.fingerprint(active) : ""),
    [active, channels],
  );

  /* ---------- Connect screen ---------- */
  if (!connected) {
    return (
      <div className="setup">
        <div className="setup__card">
          <div className="setup__brand">
            <span className="rail__dot" /> Murmur
          </div>
          <h1>Connect</h1>
          <p>
            Choose a handle. Your device generates an MLS identity and publishes a
            key package to the relay — the server never holds your keys.
          </p>
          <label className="field">
            <input
              autoFocus
              placeholder="your handle (e.g. alice)"
              value={user}
              onChange={(e) => setUser(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && connect()}
            />
          </label>
          <button className="btn btn--accent btn--block" onClick={connect} disabled={connecting || !user.trim()}>
            <Icon path={LOCK} size={14} /> {connecting ? "Connecting…" : "Connect securely"}
          </button>
          {error && <div className="err">{error}</div>}
        </div>
      </div>
    );
  }

  const activeChannel = channels.find((c) => c.group === active);
  const messages = active ? byGroup[active] ?? [] : [];

  /* ---------- Main app ---------- */
  return (
    <div className="shell">
      <aside className="rail">
        <div className="rail__brand"><span className="rail__dot" /> Murmur</div>

        <div className="rail__section">New conversation</div>
        <div className="newchat">
          <input
            placeholder="peer handle…"
            value={peerInput}
            onChange={(e) => setPeerInput(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && startChat()}
          />
          <button className="btn btn--accent btn--sm btn--icon" onClick={startChat} aria-label="Start chat">
            <Icon path={PLUS} size={15} />
          </button>
        </div>
        {error && <div className="err" style={{ padding: "0 16px 8px" }}>{error}</div>}

        <div className="rail__section">Conversations</div>
        <ul className="rail__list">
          {channels.length === 0 && (
            <li style={{ padding: "4px 12px", color: "var(--text-tertiary)", fontSize: 13 }}>
              No conversations yet.
            </li>
          )}
          {channels.map((c) => (
            <li key={c.group}>
              <button className="chan" aria-current={active === c.group} onClick={() => setActive(c.group)}>
                <span className="avatar" style={{ width: 20, height: 20, fontSize: 10 }}>
                  {c.peer.slice(0, 1).toUpperCase()}
                </span>
                {c.peer}
                <span className="lockpill" style={{ marginLeft: "auto" }}>
                  <Icon path={LOCK} size={11} />
                </span>
              </button>
            </li>
          ))}
        </ul>

        <div className="rail__me">
          <span className="avatar">{user.slice(0, 3).toUpperCase()}</span>
          <div>
            <div className="rail__me-name">{user}</div>
            <div className="rail__me-sub">encrypted · online</div>
          </div>
        </div>
      </aside>

      <main className="conv">
        {activeChannel ? (
          <>
            <header className="conv__head">
              <span className="chan__glyph" style={{ color: "var(--text-tertiary)" }}><Icon path={AT} /></span>
              <span className="conv__title">{activeChannel.peer}</span>
              <div className="conv__meta">
                <span className="lockpill"><Icon path={LOCK} size={11} /> E2EE · MLS</span>
                <span className="fp" title="Channel fingerprint (MLS exporter secret)">{fingerprint}</span>
              </div>
            </header>

            <div className="thread" ref={threadRef}>
              {messages.length === 0 && (
                <div className="empty">
                  <span className="empty__icon"><Icon path={LOCK} size={20} /></span>
                  <h2>End-to-end encrypted with {activeChannel.peer}</h2>
                  <p>Messages are sealed with MLS in your browser. The relay only routes ciphertext.</p>
                </div>
              )}
              {messages.map((m) => (
                <div className="msg" key={m.id}>
                  <span className="avatar">{m.author.slice(0, 3).toUpperCase()}</span>
                  <div>
                    <div className="msg__by">
                      <span className="msg__name">{m.mine ? "You" : m.author}</span>
                      <span className="msg__time">{m.time}</span>
                    </div>
                    <div className="msg__body">{m.text}</div>
                  </div>
                </div>
              ))}
            </div>

            <div className="composer">
              <div className="composer__box">
                <textarea
                  rows={1}
                  placeholder={`Message ${activeChannel.peer} — encrypted before it leaves`}
                  value={draft}
                  onChange={(e) => setDraft(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); send(); }
                  }}
                />
                <button className="btn btn--accent btn--icon" onClick={send} disabled={draft.trim() === ""} aria-label="Send">
                  <Icon path={SEND} />
                </button>
              </div>
              <div className="composer__hint">
                <Icon path={LOCK} size={11} /> Sealed with MLS in your browser — the server never sees plaintext.
              </div>
            </div>
          </>
        ) : (
          <div className="thread">
            <div className="empty">
              <span className="empty__icon"><Icon path={AT} size={20} /></span>
              <h2>Start a conversation</h2>
              <p>
                Enter a peer's handle in the sidebar. They must be connected and have
                published a key package. Open a second browser tab as another user to try it.
              </p>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
