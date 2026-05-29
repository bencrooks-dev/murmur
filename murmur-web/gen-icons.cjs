// Generates Murmur PWA app icons (dark rounded background + teal mark) with zero
// external deps — a tiny hand-rolled PNG encoder. Run: `node gen-icons.js`.
const fs = require("fs");
const zlib = require("zlib");
const path = require("path");

const crcTable = (() => {
  const t = [];
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
const crc32 = (buf) => {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = crcTable[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
};
const chunk = (type, data) => {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const t = Buffer.from(type, "ascii");
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([t, data])), 0);
  return Buffer.concat([len, t, data, crc]);
};
const png = (size, rgba) => {
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // RGBA
  const raw = Buffer.alloc((size * 4 + 1) * size);
  for (let y = 0; y < size; y++) {
    raw[y * (size * 4 + 1)] = 0;
    rgba.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4);
  }
  return Buffer.concat([
    sig,
    chunk("IHDR", ihdr),
    chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
};

const BG = [0x0b, 0x0c, 0x0e, 255];
const TEAL = [0x3d, 0xd4, 0xb8, 255];

const insideRounded = (x, y, x0, y0, x1, y1, rad) => {
  if (x < x0 || x >= x1 || y < y0 || y >= y1) return false;
  if (rad <= 0) return true;
  const cx0 = x0 + rad, cx1 = x1 - rad, cy0 = y0 + rad, cy1 = y1 - rad;
  if (x < cx0 && y < cy0) return (x - cx0) ** 2 + (y - cy0) ** 2 <= rad * rad;
  if (x > cx1 && y < cy0) return (x - cx1) ** 2 + (y - cy0) ** 2 <= rad * rad;
  if (x < cx0 && y > cy1) return (x - cx0) ** 2 + (y - cy1) ** 2 <= rad * rad;
  if (x > cx1 && y > cy1) return (x - cx1) ** 2 + (y - cy1) ** 2 <= rad * rad;
  return true;
};

const render = (size, maskable) => {
  const buf = Buffer.alloc(size * size * 4);
  const outerR = maskable ? 0 : size * 0.22;
  const inset = maskable ? size * 0.26 : size * 0.22;
  const sqR = (size - inset * 2) * 0.3;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let col = [0, 0, 0, 0];
      if (insideRounded(x, y, 0, 0, size, size, outerR)) {
        col = insideRounded(x, y, inset, inset, size - inset, size - inset, sqR) ? TEAL : BG;
      }
      const i = (y * size + x) * 4;
      buf[i] = col[0]; buf[i + 1] = col[1]; buf[i + 2] = col[2]; buf[i + 3] = col[3];
    }
  }
  return buf;
};

const outDir = path.join(__dirname, "public");
fs.mkdirSync(outDir, { recursive: true });
for (const [name, size, mask] of [
  ["icon-192.png", 192, false],
  ["icon-512.png", 512, false],
  ["icon-maskable-512.png", 512, true],
  ["apple-touch-icon.png", 180, false],
]) {
  fs.writeFileSync(path.join(outDir, name), png(size, render(size, mask)));
  console.log("wrote", name);
}
