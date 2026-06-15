import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

// Tauri expects a fixed port, fail if that port is not available
const host = process.env['TAURI_DEV_HOST'];

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1430,
    strictPort: true,
    host: '0.0.0.0',
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching src-tauri, .git, and agent/temp directories
      ignored: [
        '**/src-tauri/**',
        '**/.git/**',
        '**/.remember/**',
        '**/.kiro/**',
        '**/.mimocode/**',
        '**/.opencode/**',
        '**/.commandcode/**',
        '**/_bmad/**',
        '**/_bmad-output/**',
      ],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target:
      process.env['TAURI_ENV_PLATFORM'] === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env['TAURI_ENV_DEBUG'] ? 'esbuild' : false,
    sourcemap: !!process.env['TAURI_ENV_DEBUG'],
  },
});
