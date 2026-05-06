import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    svelte({
      hot: false,
      compilerOptions: {
        runes: true,
      },
    }),
  ],
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, 'src/lib'),
      $app: path.resolve(__dirname, 'tests/__mocks__/$app'),
    },
    conditions: ['browser'],
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.test.ts'],
  },
});
