# Spec Delta

## Purpose

Defines the main window, how people create and open projects, and how the tabs
and menus behave, so every other feature has a consistent place to live.

## ADDED Requirements

### Requirement: Window layout
The window SHALL have a top bar with the logo, the project menu, the save status, the tabs (Project, Editor, Export, Agents) and the agents indicator; a body area; and a bottom bar with Settings, language, theme and context information.

#### Scenario: Open the app with a project
- **WHEN** a project is open
- **THEN** the top bar shows the project name with a menu arrow, the save status and the four tabs, and the bottom bar shows Settings, language and theme

### Requirement: Tabs without a project
When no project is open, only the Project tab SHALL be usable; the Editor, Export and Agents tabs MUST be dimmed and ignore clicks.

#### Scenario: No project open
- **WHEN** the app starts and no project is open
- **THEN** the Welcome screen is shown and the Editor, Export and Agents tabs are disabled

### Requirement: Welcome screen
The Welcome screen SHALL offer "New project" and "Open project" and list the recent projects, or a short empty message when there are none.

#### Scenario: Recent projects
- **WHEN** the user has opened three projects before
- **THEN** the Welcome screen lists them, most recent first, and clicking one opens it

#### Scenario: A recent project was moved
- **WHEN** the user opens a recent project whose folder no longer exists
- **THEN** a message says the folder was not found and the entry can be removed from the list

### Requirement: New project
The New project dialog SHALL ask for a name, a location and a starting palette, show the folder that will be created, and create the project folder only when the user confirms.

#### Scenario: Create a project
- **WHEN** the user enters "Dungeon Kit", picks `~/Proyectos` and presses "Create project"
- **THEN** the folder `~/Proyectos/Dungeon Kit` is created, the project opens in the Project tab and appears in the recent list

#### Scenario: The folder exists
- **WHEN** the chosen folder name already exists and is not empty
- **THEN** the dialog shows an error and does not create the project

### Requirement: Open project
Opening a project SHALL let the user pick a folder and MUST check that it is a Sprixel project before opening it, showing a clear message if it is not.

#### Scenario: Not a project
- **WHEN** the user picks a folder without `sprixel.json`
- **THEN** a message says "This folder is not a Sprixel project" and nothing is opened

### Requirement: Project menu
The project menu, opened from the project name, SHALL offer New project, Open, Open recent, Rename project, Save a copy as, Show in folder, Export and Close project, with shortcuts, and the same commands MUST be available in the native window menu.

#### Scenario: Open the menu
- **WHEN** the user clicks the project name in the top bar
- **THEN** the menu opens below it and the project name button looks pressed

#### Scenario: There is no Save command
- **WHEN** the project menu is open
- **THEN** it has no "Save" command because the project saves by itself

### Requirement: Save status
The top bar SHALL show "Saved" or "Saving..." next to the project name, and "Error saving" with a Retry action when saving fails.

#### Scenario: Save fails
- **WHEN** a save fails
- **THEN** the top bar shows "Error saving" and a message with the reason and Retry

### Requirement: Window title
The window title SHALL show the project name and the app name.

#### Scenario: Title with a project
- **WHEN** the project "Dungeon Kit" is open
- **THEN** the window title is "Dungeon Kit - Sprixel"
