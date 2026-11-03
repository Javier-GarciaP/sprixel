//! Task 3.2: `list_projects` and `open_project`.

use crate::mcp::Permissions;
use crate::mcp::testing::{call, open_demo_project, revision, setup};
use serde_json::json;

// Checks the scenario "Recent projects": the tool needs no permission and
// returns the recent list and the open project (null when none is open).
#[test]
fn list_projects_gives_the_recent_and_the_open_project() {
    let (host, core, session) = setup();
    let none = call(&core, &session, "list_projects", json!({})).unwrap();
    assert_eq!(none["open"], json!(null));
    assert_eq!(none["recent"], json!([]));

    open_demo_project(&host);
    host.recent.lock().unwrap().push(crate::ipc::preferences::RecentProject {
        path: "/some/where".into(),
        name: "Where".into(),
    });
    let listed = call(&core, &session, "list_projects", json!({})).unwrap();
    assert_eq!(listed["open"]["name"], "Kit");
    assert_eq!(listed["recent"], json!([{ "name": "Where", "path": "/some/where" }]));
}

/// A second project in its own folder. Keep the `TempDir` while it is used.
fn second_project() -> (tempfile::TempDir, String) {
    let folder = tempfile::tempdir().unwrap();
    crate::ipc::project::create_project(&crate::ipc::project::ProjectState::default(), folder.path(), "Second").unwrap();
    let path = folder.path().join("Second").to_string_lossy().into_owned();
    (folder, path)
}

// Checks the scenario "Allowed": with the permission on, the project opens,
// enters the recent list, the UI gets the event and the activity log records it.
#[test]
fn open_project_opens_it_like_the_person_does() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { open_project: true, ..core.permissions() }).unwrap();
    let (_folder, path) = second_project();

    let answer = call(&core, &session, "open_project", json!({ "path": path })).unwrap();

    assert_eq!(answer["name"], "Second");
    assert_eq!(call(&core, &session, "list_projects", json!({})).unwrap()["open"]["name"], "Second");
    assert_eq!(host.recent.lock().unwrap()[0].name, "Second");
    let events = host.events.lock().unwrap();
    let (_, payload) = events.iter().rev().find(|(e, _)| e == super::PROJECT_OPENED_EVENT).unwrap();
    assert_eq!(payload["name"], "Second");
    drop(events);
    let log = core.activity();
    assert_eq!(log[0].entry.tool, "open_project");
    assert_eq!(log[0].entry.summary, "Opened the project Second");
}

// Checks the scenario "Permission off": permission_denied and nothing opens.
#[test]
fn open_project_is_denied_by_default() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    let (_folder, path) = second_project();

    let error = call(&core, &session, "open_project", json!({ "path": path })).unwrap_err();

    assert_eq!(error["code"], "permission_denied");
    assert_eq!(error["permission"], "openProject");
    assert_eq!(call(&core, &session, "list_projects", json!({})).unwrap()["open"]["name"], "Kit");
}

// Checks the scenario "Not a project": a missing folder, a folder that is
// not a project, and a relative path each give an error with a code, and the
// open project stays as it was (same revision).
#[test]
fn open_project_errors_keep_the_open_project() {
    let (host, core, session) = setup();
    open_demo_project(&host);
    core.set_permissions(Permissions { open_project: true, ..core.permissions() }).unwrap();
    let before = revision(&host);
    let empty = tempfile::tempdir().unwrap();
    let missing = empty.path().join("nope");

    for path in [empty.path().to_string_lossy().into_owned(), missing.to_string_lossy().into_owned()] {
        let error = call(&core, &session, "open_project", json!({ "path": path })).unwrap_err();
        assert_eq!(error["code"], "not_a_project", "{path}");
    }
    let relative = call(&core, &session, "open_project", json!({ "path": "Kit" })).unwrap_err();
    assert_eq!(relative["code"], "bad_argument");
    let missing_arg = call(&core, &session, "open_project", json!({})).unwrap_err();
    assert_eq!(missing_arg["code"], "bad_argument");

    assert_eq!(revision(&host), before);
    assert_eq!(call(&core, &session, "list_projects", json!({})).unwrap()["open"]["name"], "Kit");
}
