import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
// @ts-expect-error node:fs is a nodejs built-in
import { readFileSync } from 'node:fs';
// @ts-expect-error node:url is a nodejs built-in
import { fileURLToPath } from 'node:url';

const pkg = JSON.parse(
  readFileSync(fileURLToPath(new URL('./package.json', import.meta.url)), 'utf8'),
);

// @ts-expect-error process is a nodejs global
const buildSha = process.env.VITE_BUILD_SHA || process.env.GITHUB_SHA || 'dev';

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [sveltekit()],
  define: {
    'import.meta.env.VITE_APP_VERSION': JSON.stringify(pkg.version),
    'import.meta.env.VITE_BUILD_SHA': JSON.stringify(buildSha.slice(0, 7)),
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: 'ws', host, port: 1421 }
      : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
}));
