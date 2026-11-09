import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, ".", "");
  // Из корневого .env бэкенда читаем только PORT (в бандл он не попадает),
  // чтобы прокси всегда смотрел туда же, где слушает `cargo run`.
  const backendEnv = loadEnv(mode, "..", ["PORT"]);
  const proxyTarget = env.VITE_PROXY_TARGET || `http://127.0.0.1:${backendEnv.PORT || 8080}`;

  return {
    plugins: [react()],
    server: {
      host: true,
      port: 5173,
      strictPort: true,
      // Пропускаем запросы через HTTPS-туннель (npm run tg) при открытии из Telegram.
      allowedHosts: true,
      proxy: {
        // В dev /api проксируется на Rust-бэкенд — один origin, без CORS.
        "/api": { target: proxyTarget, changeOrigin: true },
      },
    },
  };
});
