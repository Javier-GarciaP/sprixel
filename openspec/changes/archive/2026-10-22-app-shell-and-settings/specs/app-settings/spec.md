# Spec Delta

## Purpose

Defines the personal preferences of the app: theme, language, canvas options and
keyboard shortcuts, and how they are stored and applied.

## ADDED Requirements

### Requirement: Theme
The app SHALL offer the themes Light, Dark and System, apply the choice immediately without a restart, and remember it. With System, the app MUST follow the operating system and change when it changes.

#### Scenario: Switch theme
- **WHEN** the user chooses Dark in Settings or with the quick button in the bottom bar
- **THEN** every screen changes to dark at once and the choice is still dark after restarting

### Requirement: Language
The app SHALL offer Spanish and English, apply the choice immediately and remember it. The first start MUST use the system language when it is Spanish or English, and English otherwise.

#### Scenario: Switch language
- **WHEN** the user switches from Spanish to English
- **THEN** all visible text changes to English without restarting

### Requirement: All text is translated
Every text shown to the user SHALL come from a translation file. When a key is missing in the chosen language, the English text MUST be shown, and a missing key in English MUST fail the automated checks.

#### Scenario: A missing Spanish key
- **WHEN** a screen uses a key that has no Spanish text
- **THEN** the English text is shown instead of an empty space or a raw key

### Requirement: Settings screen
Settings SHALL be opened from the bottom bar, not from the top bar, and have the pages "Appearance and language" and "Keyboard shortcuts".

#### Scenario: Open Settings
- **WHEN** the user presses "Settings" in the bottom bar
- **THEN** the Settings screen opens with the "Settings" button shown as active

### Requirement: Canvas options
Settings SHALL include an option to show a pixel grid when the canvas is zoomed in, on by default, applied immediately.

#### Scenario: Turn the grid off
- **WHEN** the user turns the pixel grid option off
- **THEN** the open canvas stops showing the grid lines

### Requirement: Keyboard shortcuts
The app SHALL provide the shortcuts listed in `docs/usage.md` (tools, project, edit and view), show them in Settings grouped by area, and use the Command key on macOS instead of Control.

#### Scenario: Shortcut in the list and in use
- **WHEN** the user presses `Ctrl+Z` in the editor
- **THEN** the last action is undone and the shortcut appears in the shortcuts page

### Requirement: Preferences storage
Preferences and the recent projects list SHALL be stored in the system config folder of the user and MUST NOT be saved inside projects.

#### Scenario: Preferences are personal
- **WHEN** a project folder is copied to another computer
- **THEN** it does not carry the theme, language or recent projects of the first computer
