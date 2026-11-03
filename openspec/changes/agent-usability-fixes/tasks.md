# Tasks

## 1. Sessions and order

- [x] 1.1 Add `Client.Close` and use it in `tool`, `tools` and `doctor`; verify with a test that 20 calls leave no open session
- [x] 1.2 Send `tools/call` through one ordered worker in the bridge; verify 20 fast calls run in order

## 2. Approval

- [x] 2.1 Refresh the clients after any answer, show the error, and emit the event when the entry is gone; verify allow, failed save and missing request

## 3. Projects

- [x] 3.1 Add the permission "Open projects" (policy, panel, Spanish and English); verify off by default and change without reconnecting
- [x] 3.2 Add `list_projects` and `open_project` with their `Host` method, the shell event and the tests; verify allowed, denied and not-a-project

## 4. Skill

- [x] 4.1 Write the embedded `SKILL.md` and install it from `sprixel mcp add claude`; verify first add and re-run
- [x] 4.2 Delete it in remove and uninstall and check it in doctor; verify each

## 5. Launcher

- [x] 5.1 Print path and version in `sprixel start`, warn on a different version and document `SPRIXEL_APP`; verify with the test app
- [x] 5.2 Check if the app reads a project path argument and make the command honest about it; verify

- [x] 5.3 Save the PNG that `sprixel tool` receives (`render_preview`) in `~/.sprixel/previews/`, print its path, keep the 20 newest, and tell the skill to open it; verify path, permissions, pruning and `--json`

## 6. Docs

- [ ] 6.1 Update `docs/ai-and-mcp.md` and `docs/roadmap.md` and run `openspec validate --all --strict`
