import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const inject404Build = () => ({
  name: 'inject-404-build',
  closeBundle() {
    const path = resolve(process.cwd(), 'dist/404.html');
    const build = process.env.VITE_BUILD_SHA || 'local';
    writeFileSync(path, readFileSync(path, 'utf8').replaceAll('__BUILD_SHA__', build));
  },
});

export default defineConfig({
  root: 'frontend',
  plugins: [svelte(), inject404Build()],
  build: { outDir: '../dist', emptyOutDir: true, target: 'es2022' },
  server: { proxy: { '/api': 'http://localhost:8080', '/health': 'http://localhost:8080' } },
  test: { environment: 'jsdom', include: ['src/**/*.test.ts'] }
});
