# Spec Delta

## Purpose

Defines how versions are numbered and how a release is built and published so
that every release is repeatable and traceable.

## ADDED Requirements

### Requirement: Semantic versions
Releases SHALL follow Semantic Versioning, use Git tags in the form `vMAJOR.MINOR.PATCH` (with `-alpha.N`, `-beta.N` or `-rc.N` for pre-releases), and the version 1.0.0 MUST be the first stable release.

#### Scenario: A pre-release tag
- **WHEN** the tag `v0.3.0-alpha.1` is pushed
- **THEN** the release is published marked as a pre-release

### Requirement: Version bump
A script SHALL update the version in every file that holds it in one step (the version source, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` and the Go build settings) and fail if they disagree afterwards.

#### Scenario: Bump to 0.2.0
- **WHEN** the bump script is run with `0.2.0`
- **THEN** every version file shows `0.2.0` and a check confirms they match

### Requirement: Pipeline on tag
Pushing a version tag SHALL start a pipeline that builds the app bundles and the `sprixel` command for Linux (x64), Windows (x64) and macOS (Apple silicon and Intel), runs the tests, and creates the release only if every step passes.

#### Scenario: A test fails
- **WHEN** a test fails during the release pipeline
- **THEN** no release is published

### Requirement: Release contents
Each release SHALL contain the installers or bundles for every system, the `sprixel` command for every system, the install scripts, a checksums file (SHA-256) and release notes taken from the matching section of `CHANGELOG.md`.

#### Scenario: Release files
- **WHEN** a release is published
- **THEN** it lists the files for the three systems, `install.sh`, `install.ps1` and `checksums.txt`

### Requirement: Tag matches the code
The pipeline SHALL fail if the tag version differs from the version in the code.

#### Scenario: Wrong tag
- **WHEN** the tag is `v1.2.0` and the code says `1.1.0`
- **THEN** the pipeline fails before building

### Requirement: Branch rules
Releases SHALL be created only from commits on `main`, after a release pull request from `dev`, as described in `CONTRIBUTING.md`.

#### Scenario: Tag on another branch
- **WHEN** a version tag points to a commit that is not on `main`
- **THEN** the pipeline fails and says releases come from `main`

### Requirement: Installer check
After building, the pipeline SHALL install the release on a clean machine of each system and run `sprixel doctor` as a smoke test.

#### Scenario: Smoke test
- **WHEN** the artifacts are built
- **THEN** each system installs them and `sprixel version` shows the release version
