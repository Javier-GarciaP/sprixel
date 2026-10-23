# Contributing to Sprixel

Thank you for your interest in Sprixel.

> **Contributions are welcome.** Sprixel is open source under the
> [MIT License](LICENSE). You can open an issue, suggest an idea or send a pull
> request. By sending a contribution you agree that it is released under the
> same MIT License.

This guide explains how we work: branches, specs, commits, pull requests,
versions and releases.

## How to contribute

- **Found a bug or have an idea?** Open an issue. Say what you did, what you
  expected and what happened. For an idea, say the problem it solves.
- **Want to send code?** For something small (a bug fix, a typo, a test), open a
  pull request. For a new feature, or a change that touches several files,
  **open an issue first**: the project works spec by spec, and a feature needs an
  approved spec before code (see [Specs first](#specs-first)).
- **How:** fork the repository, create a branch from `dev`, and open the pull
  request into `dev`. Follow the rules below for commits, tests and the pull
  request description.
- The maintainer reviews every pull request and may ask for changes. A pull
  request that does not follow these rules can be closed with an explanation.
- Be kind and patient. Everyone here is learning.

## Table of contents

- [The golden rules](#the-golden-rules)
- [Branches](#branches)
- [Specs first](#specs-first)
- [Commits](#commits)
- [Pull requests](#pull-requests)
- [Versioning](#versioning)
- [Releasing a version](#releasing-a-version)
- [Changelog](#changelog)
- [Tests](#tests)
- [Prerequisites](#prerequisites)
- [Code style and structure](#code-style-and-structure)

## The golden rules

1. **Work only in `dev`.** Every change enters `dev` through a pull request.
   Never commit or push directly to `dev` or `main`. If you work from a fork,
   branch from `dev` and open the pull request into `dev`.
2. **`main` only holds stable releases.** It changes only when a release pull
   request from `dev` is merged.
3. **Specs first.** A new feature, or any change that touches several files,
   needs an approved spec before code.
4. **Small steps.** One idea per commit and one task per pull request.
5. **Write everything in English:** code, comments, commits, docs and pull
   requests.

## Branches

```
feature/*  ──PR──▶  dev  ──release PR──▶  main  ──tag──▶  vX.Y.Z
```

| Branch | Purpose | Who writes to it |
| --- | --- | --- |
| `main` | Stable releases only. Every commit is a released version. | Release pull requests from `dev` |
| `dev` | The working branch. It always builds and passes its tests. | Pull requests from `feature/*` |
| `feature/<name>` | One task or one spec task. Created from `dev`. | You |

Name branches in lowercase with hyphens, for example
`feature/pixel-canvas-undo` or `feature/cli-doctor`. Other prefixes are used
in the same way when they make sense: `fix/<name>` for bug fixes and
`docs/<name>` for documentation only.

Typical flow:

```sh
git switch dev
git pull
git switch -c feature/project-browser-trash

# ... work, commit in small steps ...

git push -u origin feature/project-browser-trash
# open a pull request into dev
```

Delete your branch after it is merged.

## Specs first

Sprixel uses [OpenSpec](https://github.com/Fission-AI/OpenSpec). Every
feature has one spec (a "change") in `openspec/changes/<change>/` with:

- `proposal.md` - why we do it and what changes;
- `design.md` - the technical decisions (when needed);
- `specs/` - the requirements and scenarios;
- `tasks.md` - the list of tasks, in order.

Work through `tasks.md` one task at a time and make **one commit per task**.
When a change is finished and merged, archive it (`/opsx:archive`).

The list of version 1 specs is in [docs/roadmap.md](docs/roadmap.md).

## Commits

We use [Conventional Commits](https://www.conventionalcommits.org/).

```
<type>(<optional scope>): <short description>

<optional body: what and why>
```

Types:

| Type | Use it for |
| --- | --- |
| `feat` | A new feature for the user |
| `fix` | A bug fix |
| `docs` | Documentation only |
| `refactor` | Code change that does not change behavior |
| `test` | Adding or fixing tests |
| `chore` | Tooling, build, dependencies, release tasks |

Rules:

- English, imperative mood ("add", not "added"), no final period.
- Subject of **72 characters or less**.
- **One idea per commit.** Never mix two ideas.
- Before committing, check `git status` and `git diff`, and stage only the
  files that belong to the change.
- Do not add `Co-Authored-By` or "Generated with" lines.
- A breaking change adds `!` after the type, for example
  `feat(project)!: change the sprite file format`, and explains it in the body.

Good examples:

```
feat(canvas): add onion skin for the previous frame
fix(export): keep the frame order in the sprite sheet
docs: explain how to connect OpenCode
chore(release): v0.2.0
```

## Pull requests

- Open pull requests into `dev` (release pull requests go into `main`).
- Use a title in the same Conventional Commits style.
- The description must have these four parts:

```markdown
## Summary
One or two sentences: what this pull request does and why.

## What changed
- A short list of the main changes.

## How to test
Steps to check it works, and the tests that cover it.

## Related spec
openspec/changes/<change>/ (task number)
```

- Keep it small and focused. Squash or merge as you prefer, but keep the
  history readable.
- All checks must pass before merging. They run on Linux, Windows and macOS.
- If you used an AI tool to write the code, you are still the author: read it,
  run the tests and be ready to explain it.

## Versioning

Sprixel follows [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`.

| Part | Increase it when |
| --- | --- |
| `MAJOR` | A change breaks projects or commands (for example a new sprite file format that needs migration) |
| `MINOR` | A new feature that works with existing projects |
| `PATCH` | A bug fix only |

- While the version is `0.x.y`, anything can change. **`1.0.0` is the first
  stable release**, when all the version 1 specs are done.
- Pre-releases use a suffix: `0.3.0-alpha.1`, `0.3.0-beta.1`, `1.0.0-rc.1`.
- Git tags are the version with a `v`: `v1.0.0`.
- One version number is shared by every part of Sprixel (the app, the
  `sprixel` command and the installers).
- Projects saved on disk have their own `formatVersion`. Opening an older
  project must keep working through a migration.

The version lives in several files. The release task bumps all of them in
the same commit (the list is kept in the
[`install-and-release` spec](openspec/changes/install-and-release/)):
`package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and the Go
build settings of the CLI.

## Releasing a version

Only the maintainer releases. A release moves `dev` into `main`.

1. **Check `dev`.** It builds, all tests pass and finished specs are archived.
2. **Prepare the release.** Create `feature/release-vX.Y.Z` from `dev`. In one
   commit (`chore(release): vX.Y.Z`) bump the version in every file and move
   the "Unreleased" notes of [CHANGELOG.md](CHANGELOG.md) under the new
   version. Open a pull request into `dev` and merge it.
3. **Open the release pull request** from `dev` into `main`, titled
   `release: vX.Y.Z`. Use a **merge commit** (no squash) so both branches keep
   the same history.
4. **Tag and publish.** After the merge, tag the merge commit on `main`:

   ```sh
   git switch main
   git pull
   git tag -a vX.Y.Z -m "Sprixel vX.Y.Z"
   git push origin vX.Y.Z
   ```

   The tag starts the release pipeline: it builds the installers for Linux,
   Windows and macOS, the `sprixel` command and the install scripts, and
   publishes them in a GitHub Release with the changelog notes.
5. **Check the release.** Run the install script on a clean machine and run
   `sprixel doctor`.

Fixing a released version: create `fix/<name>` from `dev`, merge it into `dev`,
and release a new `PATCH` version in the same way.

## Changelog

[CHANGELOG.md](CHANGELOG.md) follows
[Keep a Changelog](https://keepachangelog.com/). Add a line under
`## [Unreleased]` for every user-visible change (`Added`, `Changed`, `Fixed`,
`Removed`) in the same pull request that makes the change.

## Tests

- Write tests for new behavior, in the same pull request.
- Tests live next to the code they test (`*.test.ts`, `*_test.go`, Rust
  `#[cfg(test)]` modules).
- A bug fix starts with a test that fails.
- Each test should check one thing and have a clear name.

## Prerequisites

To build and run Sprixel on your machine you need, on every operating
system:

- **Node and npm** (for the UI: Vite, React, TypeScript, Tailwind).
- **Go** (for the `sprixel` command).
- **Git**.
- **Rust** (`rustc` and `cargo`), for the Tauri core.

Then install the system pieces for your platform.

### Fedora Linux

These commands were run on Fedora and work:

```sh
sudo dnf install -y webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install -y c-development
sudo dnf install -y rust cargo
```

On other distros the package names are different. They are listed in the
Tauri 2 prerequisites page:
[https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/).

### Windows

- Install **Microsoft C++ Build Tools** with the "Desktop development with C++"
  workload.
- **WebView2** is already included in Windows 10 and Windows 11.
- Install **Rust** with rustup: [https://rustup.rs](https://rustup.rs).

### macOS

- Install the **Xcode Command Line Tools**:

  ```sh
  xcode-select --install
  ```

- Install **Rust** with rustup: [https://rustup.rs](https://rustup.rs).

### Verify

After installing, check that Rust is ready:

```sh
rustc --version
cargo --version
```

## Code style and structure

- Keep folders ordered and predictable. The layout is described in
  [docs/architecture.md](docs/architecture.md).
- Prefer the simplest solution that works. Do not add dependencies without
  discussing them first.
- A feature never imports from another feature, only from `shared/` or through
  the feature's `index.ts`.
- Text for users goes through the translation files (Spanish and English). Do
  not write user-facing text directly in the code.
