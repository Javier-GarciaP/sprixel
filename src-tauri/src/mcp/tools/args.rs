//! Reading the arguments of a tool call, with errors an AI can fix. Every
//! wrong argument gives `bad_argument` with the name of the argument.

use crate::document::{Project, SpriteRef};
use crate::mcp::ToolError;
use serde_json::Value;

fn bad(key: &str, expected: &str) -> ToolError {
    ToolError::new("bad_argument", format!("`{key}` must be {expected}")).with("argument", key)
}

/// The arguments must be an object (or missing, which is the same as `{}`).
pub fn object(args: &Value) -> Result<&serde_json::Map<String, Value>, ToolError> {
    static EMPTY: std::sync::OnceLock<serde_json::Map<String, Value>> = std::sync::OnceLock::new();
    match args {
        Value::Object(map) => Ok(map),
        Value::Null => Ok(EMPTY.get_or_init(Default::default)),
        _ => Err(ToolError::new("bad_argument", "the arguments must be a JSON object")),
    }
}

pub fn required_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    optional_str(args, key)?.ok_or_else(|| bad(key, "given, as text"))
}

pub fn optional_str<'a>(args: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    match object(args)?.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => Err(bad(key, "text")),
    }
}

pub fn optional_u32(args: &Value, key: &str) -> Result<Option<u32>, ToolError> {
    match object(args)?.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| bad(key, "a whole number, zero or more")),
    }
}

pub fn optional_u64(args: &Value, key: &str) -> Result<Option<u64>, ToolError> {
    match object(args)?.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| bad(key, "a whole number, zero or more")),
    }
}

pub fn required_u32(args: &Value, key: &str) -> Result<u32, ToolError> {
    optional_u32(args, key)?.ok_or_else(|| bad(key, "given, as a whole number"))
}

pub fn optional_bool(args: &Value, key: &str) -> Result<Option<bool>, ToolError> {
    match object(args)?.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(bad(key, "true or false")),
    }
}

/// A folder path written as `Characters/Humans` (or a list of names). An
/// empty text or a missing value is the project root. Names cannot be `.` or
/// `..`, and a path cannot start with a slash: a path always starts at the
/// project root and never leaves it.
pub fn folder_path(args: &Value, key: &str) -> Result<Option<Vec<String>>, ToolError> {
    let parts: Vec<String> = match object(args)?.get(key) {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(text)) => split_path(text, key)?,
        Some(Value::Array(items)) => {
            let mut parts = Vec::new();
            for item in items {
                let text = item.as_str().ok_or_else(|| bad(key, "a path like \"Characters/Humans\""))?;
                parts.push(text.to_string());
            }
            check_parts(&parts, key)?;
            parts
        }
        Some(_) => return Err(bad(key, "a path like \"Characters/Humans\"")),
    };
    Ok(Some(parts))
}

fn split_path(text: &str, key: &str) -> Result<Vec<String>, ToolError> {
    if text.starts_with(['/', '\\']) || text.contains(':') {
        return Err(invalid_path(key, "a path starts at the project root and is never absolute"));
    }
    let parts: Vec<String> =
        text.split(['/', '\\']).filter(|p| !p.is_empty()).map(str::to_string).collect();
    check_parts(&parts, key)?;
    Ok(parts)
}

fn check_parts(parts: &[String], key: &str) -> Result<(), ToolError> {
    if parts.iter().any(|p| p == "." || p == ".." || p.is_empty() || p.contains(['/', '\\'])) {
        return Err(invalid_path(key, "`.` and `..` are not allowed in a path"));
    }
    Ok(())
}

fn invalid_path(key: &str, why: &str) -> ToolError {
    ToolError::new("invalid_path", format!("`{key}` is not a valid project path: {why}")).with("argument", key)
}

/// The sprite named by `name` (and `folder`). `name` can also be the whole
/// path, like `Characters/goblin`. Without a folder the sprite is searched in
/// the whole project; if two sprites share the name the error lists their
/// folders (`ambiguous_name`).
pub fn sprite(project: &Project, args: &Value) -> Result<SpriteRef, ToolError> {
    sprite_at(project, args, "name")
}

/// Like `sprite`, with the sprite named by another argument (`create_tag`
/// uses `sprite`, because its `name` is the name of the animation).
pub fn sprite_at(project: &Project, args: &Value, key: &str) -> Result<SpriteRef, ToolError> {
    let name = required_str(args, key)?;
    let mut folder = folder_path(args, "folder")?;
    let mut name = name.to_string();
    if folder.is_none() && name.contains(['/', '\\']) {
        let mut parts = split_path(&name, key)?;
        name = parts.pop().ok_or_else(|| bad(key, "the name of a sprite"))?;
        folder = Some(parts);
    }
    if let Some(folder) = folder {
        let wanted = SpriteRef { folder, name };
        return match project.find_sprite(&wanted) {
            Some(found) => Ok(SpriteRef { folder: wanted.folder, name: found.name.clone() }),
            None => Err(not_found(project, &wanted.name)),
        };
    }
    let mut found = Vec::new();
    collect(project, &project.root, &mut Vec::new(), &name, &mut found);
    match found.len() {
        0 => Err(not_found(project, &name)),
        1 => Ok(found.remove(0)),
        _ => {
            let folders: Vec<String> = found.iter().map(|s| join(&s.folder)).collect();
            Err(ToolError::new(
                "ambiguous_name",
                format!("{} sprites are called {name:?}; add `folder` with one of: {}", found.len(), folders.join(", ")),
            )
            .with("folders", folders))
        }
    }
}

fn collect(
    project: &Project,
    folder: &crate::document::Folder,
    path: &mut Vec<String>,
    name: &str,
    found: &mut Vec<SpriteRef>,
) {
    for sprite in &folder.sprites {
        if crate::document::same_name(&sprite.name, name) {
            found.push(SpriteRef { folder: path.clone(), name: sprite.name.clone() });
        }
    }
    for child in &folder.folders {
        path.push(child.name.clone());
        collect(project, child, path, name, found);
        path.pop();
    }
}

/// A path as text, like `Characters/Humans` (empty for the root).
pub fn join(path: &[String]) -> String {
    path.join("/")
}

/// `not_found` for a sprite, with the names that do exist.
fn not_found(project: &Project, name: &str) -> ToolError {
    let mut names = Vec::new();
    names_of(&project.root, &mut names);
    names.truncate(30);
    ToolError::new(
        "not_found",
        format!("there is no sprite {name:?}; the sprites are: {}", if names.is_empty() { "(none)".into() } else { names.join(", ") }),
    )
    .with("name", name)
}

fn names_of(folder: &crate::document::Folder, names: &mut Vec<String>) {
    names.extend(folder.sprites.iter().map(|s| s.name.clone()));
    for child in &folder.folders {
        names_of(child, names);
    }
}
