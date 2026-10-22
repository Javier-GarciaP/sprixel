# Tasks

## 1. Developer prerequisites

- [x] 1.1 Install the Rust toolchain with `rustup` and the Tauri system libraries, then verify `rustc --version` and a `cargo check` of an empty Tauri app succeed
- [x] 1.2 Document the prerequisites for Linux, Windows and macOS in `CONTRIBUTING.md` and verify the steps on the Fedora machine

## 2. App skeleton

- [x] 2.1 Ask for approval of each new dependency, then scaffold Tauri 2 with Vite, React, TypeScript and Tailwind and verify the dev command opens a window
- [x] 2.2 Create the `src/app`, `src/features`, `src/shared` and `src-tauri/src` folders from `docs/architecture.md` and verify they match the document
- [x] 2.3 Show the app name and version in the window and add a test that checks the text

## 3. Tooling and rules

- [ ] 3.1 Add Prettier and ESLint with the import-boundary rules and verify a forbidden import fails the lint step (add a test fixture for it)
- [ ] 3.2 Add Vitest and `cargo test` setups with one passing test each and verify the documented test command runs both
- [ ] 3.3 Create the Go module in `cli/` with `sprixel version` and one test, and verify `go test ./...` passes

## 4. Version and build

- [ ] 4.1 Add the single version source and pass it to the app and the Go command at build time, and verify they print the same value
- [ ] 4.2 Add the documented build command and verify it produces an app bundle and a `sprixel` executable on Linux

## 5. Continuous integration

- [ ] 5.1 Add the GitHub Actions workflow for lint, test and build on Linux, Windows and macOS and verify all three jobs pass on a test pull request

## 6. Drawing benchmark

- [ ] 6.1 Build the 512x512 drawing benchmark with a typed array and a single canvas and verify it prints the median latency
- [ ] 6.2 Run it on Linux, Windows and macOS, record the numbers in `docs/architecture.md` and verify the 16 ms budget is met (or open a decision on the fallback)

## 7. Documentation

- [ ] 7.1 Update `AGENTS.md` and `docs/architecture.md` with the real commands and structure and verify every documented command runs as written
