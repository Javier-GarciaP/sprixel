# Spec Delta

## Purpose

Defines how an agent lists and opens projects.

## ADDED Requirements

### Requirement: List projects
The tool `list_projects` SHALL return the recent projects of the app (name and path) and the project that is open now, and MUST need no permission.

#### Scenario: Recent projects
- **WHEN** an agent calls `list_projects`
- **THEN** it gets the recent projects and the open one

### Requirement: Open a project
The tool `open_project` SHALL open the project in a folder the agent names, and the app MUST show it as if the person opened it. It needs the permission "Open projects", which is off by default.

#### Scenario: Allowed
- **WHEN** the permission is on and the folder holds a Sprixel project
- **THEN** the project opens in the app, enters the recent list and the activity log records the action

#### Scenario: Permission off
- **WHEN** the permission is off
- **THEN** the tool returns `permission_denied` and nothing opens

#### Scenario: Not a project
- **WHEN** the path does not exist or is not a Sprixel project
- **THEN** the tool returns an error with a code and the open project does not change

### Requirement: Permission in the panel
The permissions panel SHALL show "Open projects" in Spanish and English, off by default, and a change SHALL apply without reconnecting.

#### Scenario: Turn it on
- **WHEN** the person turns the permission on while an agent is connected
- **THEN** the next `open_project` call works without reconnecting
