module.exports = {
  apps: [
    {
      name: "swarm-os-api",
      script: "python",
      args: "-m uvicorn swarm_v2.app_v2:app --host 0.0.0.0 --port 8021",
      cwd: "F:/Development sites/TRM-Swarm-OS-v2",
      env: {
        PYTHONPATH: ".",
        NODE_ENV: "production"
      },
      autorestart: true,
      watch: false,
      max_memory_restart: "2G"
    },
    {
      name: "jarvis-advanced-server",
      script: "npx",
      args: "tsx server.ts",
      cwd: "F:/Development sites/Jarvis-Advanced-main/Jarvis-Advanced-main",
      env: {
        PORT: 4000,
        NODE_ENV: "production"
      },
      autorestart: true,
      watch: false,
      max_memory_restart: "2G"
    },
    {
      name: "swarm-dashboard-ui",
      script: "npm",
      args: "run dev -- --port 5183 --host 0.0.0.0",
      cwd: "F:/Development sites/TRM-Swarm-OS-v2/dashboard",
      env: {
        NODE_ENV: "production"
      },
      autorestart: true,
      watch: false
    }
  ]
};
