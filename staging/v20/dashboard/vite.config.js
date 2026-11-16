import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  // Relative asset paths so the built bundle can be served from any mount point. The app's API
  // calls are root-relative (API_BASE = ''), so serving the UI and the API from ONE origin --
  // the kernel -- makes every fetch resolve without a proxy, which is what retires the Node
  // dev server this platform has been running for.
  base: './',
  server: {
    port: 5183,
    proxy: {
      // 1. Live Python Swarm Core Stateful Data Stores (Port 8021)
      // Vector memory pool, pipeline artifacts, neural mesh, skills registry, DDR antibodies, agent mailboxes
      '/swarm/chat':         'http://127.0.0.1:8022',
      '/swarm/orchestrator': 'http://127.0.0.1:8022',
      '/memory':             'http://127.0.0.1:8022',
      '/artifacts':          'http://127.0.0.1:8022',
      '/mesh':               'http://127.0.0.1:8022',
      '/learning':           'http://127.0.0.1:8022',
      '/skills':             'http://127.0.0.1:8022',
      '/ddr':                'http://127.0.0.1:8022',
      '/mailbox':            'http://127.0.0.1:8022',
      '/secrets':            'http://127.0.0.1:8022',
      '/infra':              'http://127.0.0.1:8022',
      '/infrastructure':     'http://127.0.0.1:8022',
      '/aura':               'http://127.0.0.1:8022',
      '/ultrawork':          'http://127.0.0.1:8022',
      '/system/modules':     'http://127.0.0.1:8022',
      '/wetlab':             'http://127.0.0.1:8022',
      '/sensory':            'http://127.0.0.1:8022',
      '/thz':                'http://127.0.0.1:8022',
      '/ledger':             'http://127.0.0.1:8022',
      '/regulatory':         'http://127.0.0.1:8022',

      // 2. Native Rust Sovereign Microkernel Services (Port 8022)
      // Authoritative for hardware telemetry, Phi_V, clocks, CPU/RAM %, Monad truth anchor, PBFT consensus
      '/swarm/telemetry':    'http://127.0.0.1:8022',
      '/swarm':              'http://127.0.0.1:8022',
      '/system':             'http://127.0.0.1:8022',
      '/health':             'http://127.0.0.1:8022',
      '/k3':                 'http://127.0.0.1:8022',
      '/models':             'http://127.0.0.1:8022',
      '/substrate':          'http://127.0.0.1:8022',
      '/council':            'http://127.0.0.1:8022',
      '/pbft':               'http://127.0.0.1:8022',
      '/consensus':          'http://127.0.0.1:8022',
      '/federation':         'http://127.0.0.1:8022',
      '/telemetry':          'http://127.0.0.1:8022',
      '/journal':            'http://127.0.0.1:8022',
      '/generative-architect': 'http://127.0.0.1:8022',
      '/services':           'http://127.0.0.1:8022',
      '/discovery':          'http://127.0.0.1:8022',
      '/humanistic':         'http://127.0.0.1:8022',
      '/testing':            'http://127.0.0.1:8022',
      '/kanban':             'http://127.0.0.1:8022',
      '/security':           'http://127.0.0.1:8022',
      '/verification':       'http://127.0.0.1:8022',
      '/crate':              'http://127.0.0.1:8022',
      // The native surface: the kernel's own modules and the v18 crates wired in behind /native/*.
      // Absent until now, which is why the LCARS app could not reach any of it -- the tab below needs these.
      '/native':             'http://127.0.0.1:8022',
      '/sram':               'http://127.0.0.1:8022',
      '/research':           'http://127.0.0.1:8022',
      '/evolution':          'http://127.0.0.1:8022',
      '/dna':                'http://127.0.0.1:8022',
      '/coordination':       'http://127.0.0.1:8022',
      '/insights':           'http://127.0.0.1:8022',
      '/changelog':          'http://127.0.0.1:8022',
      '/experts':            'http://127.0.0.1:8022',
      '/api':                'http://127.0.0.1:8022',
    }
  }
})
