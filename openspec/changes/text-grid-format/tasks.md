# Tasks

## 1. Grid module

- [ ] 1.1 Implement the parser and writer for the format and verify tests with the documentation example and golden files
- [ ] 1.2 Implement validation with all error codes, line, row and column, and verify one test per code
- [ ] 1.3 Implement region writing with the keep-existing option and verify the region tests
- [ ] 1.4 Add a round trip property test for random sprites up to 512x512 and verify it passes

## 2. Applying

- [ ] 2.1 Implement applying a grid as one undoable operation with no partial results and verify the all-or-nothing test
- [ ] 2.2 Expose validate and apply as core commands and verify them from the UI layer

## 3. Text view

- [ ] 3.1 Build the Canvas and Text switch and the text area with line numbers and the legend, and verify against screen 21
- [ ] 3.2 Build live validation, the status line and error line highlighting with the last valid preview, and verify against screen 22
- [ ] 3.3 Build Apply, Copy and Paste and verify the paste-and-apply scenario

## 4. Documentation

- [ ] 4.1 Update `docs/text-grid-format.md` with any change and verify every example parses
