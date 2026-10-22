import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// `npm run benchmark` starts the app with this flag to measure drawing speed.
// In a normal build the flag is empty, so the benchmark code is not included.
const Root = import.meta.env.VITE_BENCHMARK
  ? (await import("./benchmark/Benchmark")).Benchmark
  : App;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
