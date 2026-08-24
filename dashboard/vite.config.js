import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5183,
    proxy: {
      '/swarm':          'http://127.0.0.1:8022',
      '/health':         'http://127.0.0.1:8022',
      '/artifacts':      'http://127.0.0.1:8022',
      '/infrastructure': 'http://127.0.0.1:8022',
      '/testing':        'http://127.0.0.1:8022',
      '/api':            'http://127.0.0.1:8022',
      '/system':         'http://127.0.0.1:8022',
      '/mesh':           'http://127.0.0.1:8022',
      '/learning':       'http://127.0.0.1:8022',
      '/memory':         'http://127.0.0.1:8022',
      '/mailbox':        'http://127.0.0.1:8022',
      '/research':       'http://127.0.0.1:8022',
      '/federation':     'http://127.0.0.1:8022',
      '/security':       'http://127.0.0.1:8022',
      '/verification':   'http://127.0.0.1:8022',
      '/ddr':            'http://127.0.0.1:8022',
      '/generative-architect': 'http://127.0.0.1:8022',
      '/evolution':      'http://127.0.0.1:8022',
      '/consensus':      'http://127.0.0.1:8022',
      '/experts':        'http://127.0.0.1:8022',
      '/coordination':   'http://127.0.0.1:8022',
      '/kanban':         'http://127.0.0.1:8022',
      '/insights':       'http://127.0.0.1:8022',
      '/changelog':      'http://127.0.0.1:8022',
      '/council':        'http://127.0.0.1:8022',
    }
  }
})
