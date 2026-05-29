import { useEffect, useMemo, useRef, useState } from "react";
import { ensureCryptoReady, SecureChannel, toHex } from "./lib/murmur";

/* ---- Iconography: one set, consistent 1.5px stroke (see design language) ---- */
const Icon = ({ path, size = 16 }: { path: string; size?: number }) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="1.5"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d={path} />
  </svg>
);
const HASH = "M4 9h16M4 15h16M10 3 8 21M16 3l-2 18";
const LOCK = "M6 11V8a6 6 0 1 1 12 0v3M5 11h14a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1Z";
const SEND = "M7 11 12 6l5 5M12 6v13";
const AT = "M16 12a4 4 0 1 0-8 0 4 4 0 0 0 8 0Zm0 0v1.5a2.5 2.5 0 0 0 5 0V12a9 9 0 1 0-3.6 7.2";

interface Message {
  id: number;
  author: string;
  body: string;
  time: string;
  ciphertext: Uint8Array;
  delivered: boolean;
}

const DM_ID = "bob";
const CHANNELS = [
  { id: "general", name: "general", kind: "chan" as const },
  { id: "design", name: "design", kind: "chan" as const },
  { id: "security", name: "security", kind: "chan" as const },
];

export default function App() {
  const [ready, setReady] = useState(false);
  const [active, setActive] = useState<string>(DM_ID);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [openCipher, setOpenCipher] = useState<number | null>(null);
  const channelRef = useRef<SecureChannel | null>(null);
  const threadRef = useRef<HTMLDivElement>(null);
  const nextId = useRef(1);

  useEffect(() => {
    let live = true;
    ensureCryptoReady().then(() => {
      if (!live) return;
      channelRef.current = new SecureChannel("You", "Bob");
      setReady(true);
    });
    return () => {
      live = false;
    };
  }, []);

  useEffect(() => {
    threadRef.current?.scrollTo({ top: threadRef.current.scrollHeight });
  }, [messages, active]);

  const fingerprint = useMemo(
    () => (ready ? channelRef.current!.fingerprint() : ""),
    [ready],
  );
  const memberCount = ready ? channelRef.current!.memberCount() : 0;

  function send() {
    const text = draft.trim();
    if (!text || !ready || active !== DM_ID) return;
    const receipt = channelRef.current!.send(text);
    setMessages((m) => [
      ...m,
      {
        id: nextId.current++,
        author: "You",
        body: text,
        time: new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
        ciphertext: receipt.ciphertext,
        delivered: receipt.delivered,
      },
    ]);
    setDraft("");
  }

  const onDM = active === DM_ID;

  return (
    <div className="shell">
      <aside className="rail">
        <div className="rail__brand">
          <span className="rail__dot" />
          Murmur
        </div>

        <div className="rail__section">Direct messages</div>
        <ul className="rail__list" style={{ flex: "none" }}>
          <li>
            <button
              className="chan"
              aria-current={onDM}
              onClick={() => setActive(DM_ID)}
            >
              <span className="avatar" style={{ width: 20, height: 20, fontSize: 10 }}>
                B
              </span>
              Bob
              <span className="lockpill" style={{ marginLeft: "auto" }}>
                <Icon path={LOCK} size={11} /> live
              </span>
            </button>
          </li>
        </ul>

        <div className="rail__section">Channels</div>
        <ul className="rail__list">
          {CHANNELS.map((c) => (
            <li key={c.id}>
              <button
                className="chan"
                aria-current={active === c.id}
                onClick={() => setActive(c.id)}
              >
                <span className="chan__glyph">
                  <Icon path={HASH} size={15} />
                </span>
                {c.name}
              </button>
            </li>
          ))}
        </ul>

        <div className="rail__me">
          <span className="avatar">YOU</span>
          <div>
            <div className="rail__me-name">You</div>
            <div className="rail__me-sub">{ready ? "encrypted · online" : "connecting…"}</div>
          </div>
        </div>
      </aside>

      <main className="conv">
        <header className="conv__head">
          <span className="chan__glyph" style={{ color: "var(--text-tertiary)" }}>
            <Icon path={onDM ? AT : HASH} />
          </span>
          <span className="conv__title">{onDM ? "Bob" : CHANNELS.find((c) => c.id === active)?.name}</span>
          {onDM && (
            <div className="conv__meta">
              <span>{memberCount} members</span>
              <span className="lockpill">
                <Icon path={LOCK} size={11} /> E2EE · MLS
              </span>
              <span className="fp" title="Channel fingerprint (exporter secret)">
                {fingerprint}
              </span>
            </div>
          )}
        </header>

        <div className="thread" ref={threadRef}>
          {onDM && messages.length === 0 && (
            <div className="empty">
              <span className="empty__icon">
                <Icon path={LOCK} size={20} />
              </span>
              <h2>This conversation is end-to-end encrypted</h2>
              <p>
                Messages are sealed with MLS (RFC 9420) in your browser. The relay
                only ever sees ciphertext. Say something to see it encrypt.
              </p>
            </div>
          )}

          {onDM &&
            messages.map((m) => (
              <div className="msg" key={m.id}>
                <span className="avatar">{m.author === "You" ? "YOU" : "B"}</span>
                <div>
                  <div className="msg__by">
                    <span className="msg__name">{m.author}</span>
                    <span className="msg__time">{m.time}</span>
                  </div>
                  <div className="msg__body">{m.body}</div>
                  <button
                    className="msg__enc msg__enc--ok"
                    onClick={() => setOpenCipher(openCipher === m.id ? null : m.id)}
                  >
                    <Icon path={LOCK} size={12} />
                    {m.delivered ? "encrypted · delivered" : "encrypted"} ·{" "}
                    {m.ciphertext.length} bytes
                  </button>
                  {openCipher === m.id && (
                    <div className="cipher">{toHex(m.ciphertext)}</div>
                  )}
                </div>
              </div>
            ))}

          {!onDM && (
            <div className="empty">
              <span className="empty__icon">
                <Icon path={HASH} size={20} />
              </span>
              <h2>#{CHANNELS.find((c) => c.id === active)?.name}</h2>
              <p>
                Demo build — the live encrypted channel is your DM with Bob. Group
                channels arrive with the server (Phase 3).
              </p>
            </div>
          )}
        </div>

        <div className="composer">
          <div className="composer__box">
            <textarea
              rows={1}
              placeholder={onDM ? "Message Bob — encrypted before it leaves" : "Demo channel"}
              value={draft}
              disabled={!ready || !onDM}
              onChange={(e) => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  send();
                }
              }}
            />
            <button
              className="btn btn--accent btn--icon"
              onClick={send}
              disabled={!ready || !onDM || draft.trim() === ""}
              aria-label="Send"
            >
              <Icon path={SEND} />
            </button>
          </div>
          <div className="composer__hint">
            <Icon path={LOCK} size={11} />
            {ready
              ? "Sealed with MLS in your browser — the server never sees plaintext."
              : "Initializing crypto core…"}
          </div>
        </div>
      </main>
    </div>
  );
}
