import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
// @ts-expect-error type error without @types/node package
import process from "node:process";
import { fileURLToPath } from "node:url";
import pkg from "./package.json" with { type: "json" };
const host = process.env.TAURI_DEV_HOST;

// Dev only: prints the result sent by the drawing benchmark in the terminal.
function benchmarkReporter(): Plugin {
  return {
    name: "benchmark-reporter",
    apply: "serve",
    configureServer(server) {
      server.middlewares.use("/__benchmark", (req, res) => {
        let body = "";
        req.on("data", (chunk) => (body += chunk));
        req.on("end", () => {
          console.log(`BENCHMARK_RESULT ${body}`);
          res.end("ok");
        });
      });
    },
  };
}

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwindcss(), benchmarkReporter()],

  // The app shows the version from package.json (see App.tsx)
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },

  define: { __APP_VERSION__: JSON.stringify(pkg.version) },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
