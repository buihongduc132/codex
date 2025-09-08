// PM2 ecosystem configuration for qoo-bridge
// - Uses run.sh to manage venv and start the bridge (and optional LiteLLM)
// - Retries on error with a delay to avoid tight crash loops
// - Can be saved/resurrected via `pm2 save` and `pm2 resurrect`

module.exports = {
  apps: [
    {
      name: "qoo-bridge",
      script: "./run.sh",
      cwd: __dirname,
      interpreter: "/bin/bash",
      exec_mode: "fork",
      autorestart: true,
      watch: false,
      max_restarts: 20,
      restart_delay: 5000, // 5s between restarts
      min_uptime: 5000, // consider "up" if runs >= 5s
      time: true,
      env: {
        BRIDGE_HOST: "0.0.0.0",
        BRIDGE_PORT: "4050",
        // Uncomment to enable LiteLLM externally if installed
        // LITELLM_HOST: "0.0.0.0",
        // LITELLM_PORT: "4000",
      },
    },
  ],
};

