import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5183,
    // SECURITY 2026-11-03: vite defaulted to 0.0.0.0, exposing the dashboard on
    // the LAN. Loopback by default; TRM_BIND_HOST overrides for a deliberate
    // exposure (e.g. viewing LCARS from a tablet).
    host: process.env.TRM_BIND_HOST || '127.0.0.1',
    proxy: {
      '/swarm':          { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/health':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/artifacts':      { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/infrastructure': { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/infra':          { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/testing':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/api':            { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/system':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/mesh':           { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/learning':       { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/memory':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/mailbox':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/research':       { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/federation':     { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/security':       { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/secrets':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/verification':   { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/ddr':            { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/generative-architect': { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/evolution':      { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/consensus':      { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/experts':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/coordination':   { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/kanban':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/insights':       { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/changelog':      { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/council':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/ultrawork':      { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/skills':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/journal':        { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/monad':          { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/agents':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/gpu':            { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/cosmos':         { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/gate':           { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/k3':             { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/spectral':       { target: 'http://127.0.0.1:8022', changeOrigin: true },
      '/companion':      { target: 'http://127.0.0.1:3001', changeOrigin: true },
    }
  }
})
