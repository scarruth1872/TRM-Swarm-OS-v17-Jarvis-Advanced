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
          // Jarvis's own server (server.ts) serves the /api routes; 8021 was the retired Python
          // proxy, so every dev-UI call 500'd while the kernel and Jarvis were both healthy.
          target: 'http://127.0.0.1:3000',
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
