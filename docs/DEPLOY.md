# Deploying Murmur so you (and peers) can install it on a phone

There is **no native app store build yet**. The easiest cross-platform install is
the **PWA**: open the web app's URL in a phone browser and tap **Add to Home
Screen** — it installs with its own icon and launches full-screen, like an app.

You need two things publicly reachable:

1. **The web client (PWA)** — static files. Host free on GitHub Pages / Netlify / Vercel.
2. **The relay** — a small always-on server. Host on Fly.io / Render / a VPS, or
   expose your local relay through a tunnel.

The web client finds the relay via the `VITE_RELAY_URL` build-time variable
(e.g. `wss://relay.yourdomain.com/ws`). Over HTTPS you **must** use `wss://`.

---

## Option A — Same Wi-Fi only (zero hosting, instant)
Already works. On your computer:
```bash
cd murmur-web && npm run dev -- --host 0.0.0.0   # web on :5173
cd murmur-server && cargo run                      # relay on :8787
```
On your phone (same Wi-Fi): open `http://<computer-LAN-ip>:5173/`, then Add to
Home Screen. Allow ports 5173 and 8787 through the computer's firewall.

## Option B — Fastest public setup (tunnel the local relay)
Keep the relay on your machine, give it a public URL with a tunnel:
```bash
cargo run                                  # relay on :8787
cloudflared tunnel --url http://localhost:8787   # prints https://xxxx.trycloudflare.com
```
Then build + deploy the PWA pointing at it (wss):
```bash
cd murmur-web
VITE_RELAY_URL="wss://xxxx.trycloudflare.com/ws" MURMUR_BASE="/murmur/" npm run build
```
Publish `dist/` (see Option C). Peers anywhere open the Pages URL and install.

## Option C — GitHub Pages for the PWA (free, public)
This repo includes `.github/workflows/deploy-pages.yml`. To use it:
1. Push to GitHub (done).
2. Repo → **Settings → Pages → Source: GitHub Actions**.
3. Repo → **Settings → Secrets and variables → Actions → Variables**: add
   `RELAY_URL` = your public relay (`wss://.../ws`).
4. Push to `main` (or run the workflow). Your PWA goes live at
   `https://<user>.github.io/murmur/`. Share that link; anyone can install it.

## Option D — Relay on a host (always-on, no tunnel)
`murmur-server/Dockerfile` builds the relay. On Fly.io:
```bash
cd murmur-server
fly launch --no-deploy           # internal port 8787
fly deploy
```
Fly gives you `https://<app>.fly.dev` (wss supported). Use that as `RELAY_URL`.

---

## Toward a real installable APK (later)
Once the PWA is live on HTTPS, an Android APK can be produced with a Trusted Web
Activity wrapper (Bubblewrap) and offered as a GitHub **Release** asset. iOS has no
sideload path; the PWA (Add to Home Screen) is the iOS install story. Native RN
clients are a planned phase.
