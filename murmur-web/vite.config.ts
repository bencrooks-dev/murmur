import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

// `base` lets the same build work at a sub-path (e.g. GitHub Pages /murmur/).
// Set MURMUR_BASE=/murmur/ when deploying there; defaults to root for LAN use.
const base = process.env.MURMUR_BASE || "/";

export default defineConfig({
  base,
  plugins: [
    react(),
    VitePWA({
      registerType: "autoUpdate",
      includeAssets: ["apple-touch-icon.png"],
      manifest: {
        name: "Murmur",
        short_name: "Murmur",
        description: "End-to-end encrypted messaging.",
        theme_color: "#0b0c0e",
        background_color: "#0b0c0e",
        display: "standalone",
        start_url: base,
        scope: base,
        icons: [
          { src: "icon-192.png", sizes: "192x192", type: "image/png" },
          { src: "icon-512.png", sizes: "512x512", type: "image/png" },
          {
            src: "icon-maskable-512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "maskable",
          },
        ],
      },
      workbox: {
        // The WASM core is ~1.5 MB; allow it to be precached for offline launch.
        maximumFileSizeToCacheInBytes: 3 * 1024 * 1024,
        globPatterns: ["**/*.{js,css,html,wasm,png,svg}"],
      },
    }),
  ],
  // The WASM core is imported as an asset URL (`?url`) and passed to init().
});
