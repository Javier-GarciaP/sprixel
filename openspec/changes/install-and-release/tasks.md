# Tasks

## 1. Versioning

- [ ] 1.1 Add the version source and the bump script that updates and checks every version file, and verify a test that bumps a copy of the repository
- [ ] 1.2 Add the check that fails when the tag and the code version differ and verify with a wrong tag

## 2. Installers

- [ ] 2.1 Write `install.sh` with system detection, checksum verification, user-only install, PATH handling and the MIT license notice, and verify it in a clean Linux container and on macOS
- [ ] 2.2 Write `install.ps1` with the same behavior and verify it on Windows
- [ ] 2.3 Add reinstall safety (keep projects, settings and tokens) and verify a reinstall test on each system

## 3. Update and uninstall

- [ ] 3.1 Implement `sprixel update` with the same checks and verify the up-to-date and upgrade cases
- [ ] 3.2 Implement `sprixel uninstall` with confirmations and verify projects stay untouched

## 4. Release pipeline

- [ ] 4.1 Build the app bundles and the command for each target in the pipeline and verify the artifacts exist
- [ ] 4.2 Add the checksums, the changelog notes extraction and the release creation, and verify a pre-release on a test tag
- [ ] 4.3 Add the branch rule (tags only from `main`) and the smoke test on clean machines, and verify both with test tags

## 5. Documentation

- [ ] 5.1 Update `README.md`, `CONTRIBUTING.md` and `docs/usage.md` (install, update, uninstall, system warnings) and verify the commands match the real scripts
- [ ] 5.2 Publish `0.1.0` following the release steps and verify the install command works from a clean machine
