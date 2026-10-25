// Runs the drawing benchmark inside the real app window and prints the result.
// Exit code is 0 when the median is within the 16 ms budget, 1 otherwise.
import { spawn, execFileSync } from "node:child_process";

const BUDGET_MS = 16;
const isWindows = process.platform === "win32";

const child = spawn("npm", ["run", "tauri", "dev"], {
  env: { ...process.env, VITE_BENCHMARK: "1" },
  shell: isWindows,
  detached: !isWindows,
  stdio: ["ignore", "pipe", "inherit"],
});

function stopApp() {
  if (isWindows) {
    execFileSync("taskkill", ["/pid", String(child.pid), "/T", "/F"]);
  } else {
    process.kill(-child.pid); // the whole process group
  }
}

let buffer = "";
child.stdout.on("data", (chunk) => {
  buffer += chunk;
  const lines = buffer.split("\n");
  buffer = lines.pop();
  for (const line of lines) {
    const marker = line.indexOf("BENCHMARK_RESULT ");
    if (marker === -1) continue;
    const result = JSON.parse(line.slice(marker + "BENCHMARK_RESULT ".length));
    console.log(JSON.stringify(result, null, 2));
    const ok = result.medianMs <= BUDGET_MS;
    console.log(
      `\nMedian ${result.medianMs.toFixed(1)} ms, budget ${BUDGET_MS} ms: ${ok ? "OK" : "TOO SLOW"}`,
    );
    // For information, not part of the budget.
    console.log(
      `Time between frames ${result.frameMs.toFixed(1)} ms, work of a pointer move ${result.workMs.toFixed(1)} ms`,
    );
    if (result.frameMs > 20) {
      console.log(
        `Warning: frames come every ${result.frameMs.toFixed(0)} ms (a 30 Hz screen, or a window that is hidden or covered). ` +
          "The median cannot be lower than that. Show the window and run it again.",
      );
    }
    console.log(
      `Fill of the whole frame ${result.fillMs.toFixed(0)} ms, full repaint ${result.fullRepaintMs.toFixed(0)} ms`,
    );
    stopApp();
    process.exit(ok ? 0 : 1);
  }
});
