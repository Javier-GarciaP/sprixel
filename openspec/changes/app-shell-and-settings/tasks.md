# Tasks

## 1. Design tokens and fonts

- [x] 1.1 Add the color, type and spacing tokens with dark and light values and the three fonts, and verify a test page renders both themes
- [ ] 1.2 Build the shared base components (button, chip, field, switch, dialog, menu, toast) and verify each in both themes by screenshot comparison with `docs/design/`

## 2. Translation

- [ ] 2.1 Add the i18n layer with Spanish and English files and the fallback rule, and verify tests for fallback and missing key detection
- [ ] 2.2 Add the language setting, the system language detection and the quick button, and verify switching without a restart

## 3. Window layout

- [ ] 3.1 Build the top bar, tabs, body and bottom bar and verify the layout at 1024x640 and 1440x900
- [ ] 3.2 Add the disabled tab behavior with no project and verify a test that clicks a disabled tab
- [ ] 3.3 Add the save status area and the window title and verify the states Saved, Saving and Error

## 4. Commands, menus and shortcuts

- [ ] 4.1 Build the command registry and the project menu (anchored to the project name) and verify there is no Save item
- [ ] 4.2 Build the native window menu from the same registry and verify the commands match the project menu
- [ ] 4.3 Add the shortcuts for the platform (Control or Command) and verify the shortcuts page lists them

## 5. Welcome and projects

- [ ] 5.1 Build the Welcome screen, recent projects and the moved-folder message and verify the scenarios in the spec
- [ ] 5.2 Build the New project dialog and Open project flow on top of the storage layer and verify creating and opening a project end to end

## 6. Settings

- [ ] 6.1 Build the Settings screen (appearance and language, shortcuts), the theme setting with System and the pixel grid option, and verify persistence after a restart
- [ ] 6.2 Document the settings and shortcuts in `docs/usage.md` and verify they match the app
