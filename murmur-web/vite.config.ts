import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  // The WASM core is imported as an asset URL (`?url`) and passed to the
  // wasm-bindgen `init()`, so no extra wasm plugin is needed.
});
