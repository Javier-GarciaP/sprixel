import { useEffect, useRef, useState } from "react";
import { runBenchmark, type BenchmarkResult } from "./runBenchmark";

const BUDGET_MS = 16;

// Development-only screen. It runs the drawing benchmark and sends the result
// to the dev server (see vite.config.ts), which prints it in the terminal.
export function Benchmark() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [result, setResult] = useState<BenchmarkResult | null>(null);

  useEffect(() => {
    if (!canvasRef.current) return;
    runBenchmark(canvasRef.current).then((value) => {
      setResult(value);
      void fetch("/__benchmark", {
        method: "POST",
        body: JSON.stringify(value),
      });
    });
  }, []);

  return (
    <main className="flex h-screen flex-col items-center justify-center gap-3">
      <canvas
        ref={canvasRef}
        className="border"
        style={{ imageRendering: "pixelated" }}
      />
      <p className="font-mono text-sm">
        {result
          ? `median ${result.medianMs.toFixed(1)} ms (budget ${BUDGET_MS} ms), fill ${result.fillMs.toFixed(0)} ms, full repaint ${result.fullRepaintMs.toFixed(0)} ms`
          : "Running benchmark..."}
      </p>
    </main>
  );
}
