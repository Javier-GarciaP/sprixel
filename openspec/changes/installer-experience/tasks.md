# Tasks

## 1. Mode, flags and output in `install.sh`

- [ ] 1.1 Write tests for the options (`--yes`, `--no-menu`, `--connect`, their variables, and an unknown value), then add them to `install.sh`
- [ ] 1.2 Write tests for finding a terminal (`SPRIXEL_TTY` for tests, and none), then add the `INTERACTIVE` switch, with no questions when there is no terminal
- [ ] 1.3 Add plain and colored output (terminal only, `NO_COLOR` respected) and verify the output of a run saved to a file has no control codes

## 2. Summary and confirmation (`install.sh`)

- [ ] 2.1 Print the welcome, the version, the system and the install folders before any download, and verify with a test
- [ ] 2.2 Ask for confirmation (default yes) and verify that "no" downloads nothing, changes nothing and exits with 0
- [ ] 2.3 Verify that a run with no terminal asks nothing and installs exactly as before (the existing tests pass unchanged)

## 3. Progress (`install.sh`)

- [ ] 3.1 Print `[n/5] name` before each download and `curl --progress-bar` on a terminal, keeping the https-only flags, and verify the flags with a test
- [ ] 3.2 Print one plain line per file when there is no terminal, and verify with a test
- [ ] 3.3 Verify that all checksums are still checked before anything is copied, and that a wrong one installs nothing

## 4. Options (`install.sh`)

- [ ] 4.1 Ask whether to connect an AI, offering only the tools found, and run `sprixel mcp add` by its full path; verify the choices, the no-tool case and a failing registration (a warning, the install succeeds)
- [ ] 4.2 After `app-menu-entries` is built, ask whether to create the menu entry, and create the `~/.sprixel/no-menu-entry` marker on "no" or `--no-menu`; verify both answers
- [ ] 4.3 Print the final summary with the choices and the next steps, and verify the error messages and exit codes did not change

## 5. Same behavior in `install.ps1`

- [ ] 5.1 Add the variables, the interactive switch (`Read-Host`, no terminal detected) and the plain and colored output, with tests in `install-ps1.test.js`
- [ ] 5.2 Add the summary, the confirmation and the questions, with the same texts as `install.sh`
- [ ] 5.3 Add the progress: `curl.exe` when found, `Invoke-WebRequest` with silent progress and plain lines otherwise
- [ ] 5.4 Verify that the Windows release smoke test still runs with no questions

## 6. Documentation

- [ ] 6.1 Describe the questions, the flags and the variables in `docs/usage.md`, the README, `README.es.md` and both guides, and verify `install-docs.test.js` still passes
- [ ] 6.2 Write the changelog entry

## 7. Check by hand and release

- [ ] 7.1 On a real Linux terminal, run `curl … | sh` and check the summary, declining, accepting, the progress bar, the AI question and the plain output when redirected to a file
- [ ] 7.2 Check the one-line command with the flags `--yes --no-menu --connect none`
- [ ] 7.3 Run `openspec validate --all --strict`, then release it together with `app-menu-entries` as 0.1.1
