# app-platform Specification

## Purpose

Defines the platform the app runs on: the supported systems, how it is built
and checked, the single version number, and the drawing speed budget that the
rest of the product must respect.

## Requirements

### Requirement: Supported systems
The app SHALL run on Linux (x64), Windows (x64) and macOS (Apple silicon and Intel) and open a main window that shows the app name and version.

#### Scenario: Launch on each system
- **WHEN** the built app is started on any supported system
- **THEN** a main window opens within 3 seconds and shows "Sprixel" and the current version

#### Scenario: Window can be resized
- **WHEN** the user resizes the window down to 1024x640
- **THEN** the layout stays usable and nothing is cut off

### Requirement: Documented commands
The repository SHALL provide documented commands, run from the repository root, to start the app in development, run all tests and build the release app and the `sprixel` command.

#### Scenario: Run the tests
- **WHEN** a developer runs the documented test command
- **THEN** the UI, the Rust core and the Go command tests all run and the exit code reflects the result

#### Scenario: Build the release
- **WHEN** a developer runs the documented build command on a supported system
- **THEN** an installable app bundle and a `sprixel` executable are produced

### Requirement: Single version number
The product SHALL have one version number, stored in one source of truth, that is shown by the app, by `sprixel version` and by the release artifacts.

#### Scenario: Versions agree
- **WHEN** the app, `sprixel version` and the release tag are compared
- **THEN** they show the same `MAJOR.MINOR.PATCH` value

### Requirement: Automated checks
Every pull request SHALL run formatting, linting, tests and a build on Linux, Windows and macOS, and MUST fail if any step fails.

#### Scenario: A pull request breaks a test
- **WHEN** a pull request contains a failing test
- **THEN** the check fails on the affected system and the pull request cannot be merged

### Requirement: Feature isolation
Import rules SHALL be checked automatically: a feature MUST NOT import from another feature (only from `shared/` or the other feature's `index.ts`) and `shared/` MUST NOT import from `features/`.

#### Scenario: Forbidden import
- **WHEN** code in one feature imports a file inside another feature that is not its `index.ts`
- **THEN** the lint step reports an error

### Requirement: Drawing speed budget
Painting a stroke on a 512x512 sprite SHALL stay smooth: the median time from pointer movement to the pixel being visible MUST be 16 ms or less on the reference machine, on every supported system.

The budget was verified on Linux only (median 9.5 to 10 ms on Fedora with WebKitGTK). By decision of the maintainer, Windows and macOS were not measured when this requirement was accepted; the budget still applies to them and `npm run benchmark` measures it there.

#### Scenario: Benchmark on a large sprite
- **WHEN** the drawing benchmark paints a continuous stroke across a 512x512 sprite
- **THEN** the median latency is 16 ms or less and the result is saved in the repository documentation

#### Scenario: Budget not met
- **WHEN** the benchmark median is above 16 ms on any supported system
- **THEN** the foundation is not complete and a rendering alternative must be chosen before more UI work starts
