import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import {defineConfig} from 'vite';

export default defineConfig(() => {
  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
      },
    },
    server: {
      proxy: {
        '/api': {
          // 127.0.0.1, not "localhost": Node resolves localhost to ::1 first on this host, but
          // the backend binds IPv4, so every proxied call failed with ECONNREFUSED -> HTTP 500.
          target: 'http://127.0.0.1:8021',
          changeOrigin: true,
          // NO rewrite: server.ts registers its routes WITH the /api prefix (/api/chat,
          // /api/memory, ...). Stripping it here turned every UI call into a 404 on :8021 --
          // the interface rendered, the backend looked dead, and the chat box answered nothing.
        },
      },
      // HMR is disabled in AI Studio via DISABLE_HMR env var.
      // Do not modify—file watching is disabled to prevent flickering during agent edits.
      hmr: process.env.DISABLE_HMR !== 'true',
      // Disable file watching when DISABLE_HMR is true to save CPU during agent edits.
      watch: process.env.DISABLE_HMR === 'true' ? null : {
        ignored: [
          '**/user_data/**',
          '**/jarvis_*.json'
        ]
      },
    },
  };
});
