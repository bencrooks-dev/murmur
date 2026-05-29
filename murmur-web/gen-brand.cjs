// Generates Murmur brand assets from the source logos in ./brand:
//   murmur-dark.png  = black-background square (used for app/PWA/favicon icons)
//   murmur-light.png = white logo on transparent (used in the dark UI)
// Run: `node gen-brand.cjs`  (requires `sharp`).
const sharp = require("sharp");
const path = require("path");

const BRAND = path.join(__dirname, "brand");
const PUB = path.join(__dirname, "public");
const dark = path.join(BRAND, "murmur-dark.png");
const light = path.join(BRAND, "murmur-light.png");

(async () => {
  // App / PWA / favicon icons — from the self-contained black-bg logo.
  for (const [name, size] of [
    ["icon-192.png", 192],
    ["icon-512.png", 512],
    ["icon-maskable-512.png", 512],
    ["apple-touch-icon.png", 180],
    ["favicon-32.png", 32],
  ]) {
    await sharp(dark).resize(size, size, { fit: "cover" }).png().toFile(path.join(PUB, name));
  }

  // Full stacked logo (transparent) for the connect screen.
  await sharp(light).trim().resize({ width: 520 }).png().toFile(path.join(PUB, "murmur-logo.png"));

  // Shield mark only (transparent) for the sidebar — crop the upper region, then
  // trim to the artwork so the box need not be pixel-perfect.
  const meta = await sharp(light).metadata();
  const W = meta.width, H = meta.height;
  const left = Math.round(W * 0.29);
  const top = Math.round(H * 0.15);
  const side = Math.round(W * 0.42);
  await sharp(light)
    .extract({ left, top, width: side, height: Math.min(side, H - top) })
    .trim()
    .resize(120, 120, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toFile(path.join(PUB, "murmur-mark.png"));

  console.log("brand assets written to public/");
})().catch((e) => { console.error(e); process.exit(1); });
