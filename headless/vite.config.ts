import preact from '@preact/preset-vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

const server = process.env.BRIDGETHING_CONSOLE_DEV_SERVER ?? 'http://127.0.0.1:8899';

export default defineConfig({
  plugins: [preact(), tailwindcss()],
  clearScreen: false,
  // `bun run dev` serves the page while the rust console answers the api
  server: {
    port: 1421,
    strictPort: true,
    proxy: {
      '/api': { target: server, changeOrigin: true, ws: true },
    },
  },
  build: { target: 'es2022', emptyOutDir: true },
});
