# Spec Delta

## Purpose

Defines how the `sprixel` command and the app keep sessions, order calls,
answer approvals and start the right build.

## ADDED Requirements

### Requirement: Sessions are closed
The commands `sprixel tool`, `sprixel tools` and `sprixel doctor` SHALL end the app session they opened before they exit, also when the call failed.

#### Scenario: Many calls
- **WHEN** `sprixel tool` runs 20 times in a row
- **THEN** the app has no open session left from those calls and the next call is not refused for too many clients

### Requirement: Calls keep their order
The stdio bridge SHALL send the `tools/call` messages to the app one after another in the order they arrived, without blocking the reading of other messages.

#### Scenario: Fast calls
- **WHEN** an AI tool sends 20 `tools/call` messages without waiting for answers
- **THEN** the app runs them in that same order

### Requirement: Approval answer
After the person answers an approval request, the app SHALL close the dialog and refresh the client list. If the answer cannot be saved, the dialog MUST stay open and show the error.

#### Scenario: Allow
- **WHEN** the person presses Allow
- **THEN** the dialog closes and the client shows as connected

#### Scenario: Answer fails
- **WHEN** the answer cannot be saved
- **THEN** the dialog shows the error and stays open

#### Scenario: Request already gone
- **WHEN** the person answers a request that no longer exists
- **THEN** the client list is refreshed and the dialog closes

### Requirement: Images are saved to files
When `sprixel tool` receives an image in the result (for example from `render_preview`), it SHALL save the PNG to a file in the folder `previews` of the Sprixel data folder, written atomically and readable only by the user, and SHALL print the path so an AI that only uses the shell can open the file and see the result. It MUST keep only the 20 newest preview files. With `--json` each image keeps its `data` and also has a `path`.

#### Scenario: Preview from the shell
- **WHEN** `sprixel tool render_preview name=goblin` runs
- **THEN** the output says `image saved:` with the path of a PNG file that holds the image

#### Scenario: Many previews
- **WHEN** more than 20 previews were saved
- **THEN** only the 20 newest files stay in the folder

### Requirement: Start tells which build
`sprixel start` SHALL print the path and the version of the app it starts, SHALL warn when that version differs from the version of the command, and the variable `SPRIXEL_APP` SHALL choose the app to start.

#### Scenario: Old build
- **WHEN** the app found has another version than the command
- **THEN** the output names the path and both versions and says how to use `SPRIXEL_APP`
