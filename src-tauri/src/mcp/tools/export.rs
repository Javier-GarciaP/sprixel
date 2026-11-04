//! The `export` tool. It uses the export module of the app (`ipc::export`),
//! so a sprite sheet or PNG files are the same as the ones the person
//! exports. The files go to a folder inside the project and never anywhere
//! else; the path is checked by the export module (no `..`, no absolute path,
//! no link that leaves the project).

use super::args;
use crate::document::SpriteRef;
use crate::ipc::export::{self, FormatArg, JsonFormatArg, OptionsArg, ScopeArg};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "export",
        description: "Exports sprites as files for a game engine, into a folder INSIDE the project (default `assets/sprites`). \
`format` `sheet` writes one PNG sprite sheet and a JSON file that describes it (`json_format` `aseprite` writes the Aseprite layout that Godot and Unity importers read); `png` writes one PNG file for every frame. \
`scope` is `project` (all sprites), `folder` (with `folder`) or `sprites` (with `sprites`, a list of sprite names or paths). \
The answer lists the files written, as paths from the project folder. Exported files cannot be undone.",
        schema: json!({ "type": "object", "properties": {
            "scope": { "type": "string", "enum": ["project", "folder", "sprites"], "description": "Default `project`." },
            "folder": { "type": "string", "description": "For scope `folder`: a path like \"Characters\". Its sub-folders are included." },
            "sprites": { "type": "array", "items": { "type": "string" }, "description": "For scope `sprites`: names or paths like \"Characters/goblin\"." },
            "format": { "type": "string", "enum": ["sheet", "png"], "description": "Default `sheet`." },
            "scale": { "type": "integer", "enum": [1, 2, 4, 8], "description": "Default 1." },
            "spacing": { "type": "integer", "minimum": 0, "maximum": 16, "description": "Pixels between frames in the sheet. Default 0." },
            "json_format": { "type": "string", "enum": ["sprixel", "aseprite"], "description": "For format `sheet`: the layout of the JSON. `sprixel` (default) or `aseprite`, the Aseprite sprite sheet JSON that the Godot and Unity importers read." },
            "naming": { "type": "string", "description": "The name of each frame. Uses {sprite}, {animation} and {frame}. Default `{sprite}_{animation}_{frame}`." },
            "output": { "type": "string", "description": "A folder inside the project. Default `assets/sprites`." },
        }}),
        need: Need::Export,
        run: export,
    }]
}

fn export(ctx: &Ctx, input: &Value) -> ToolResult {
    let state = ctx.core.host.project();
    let scope = match args::optional_str(input, "scope")?.unwrap_or("project") {
        "project" => ScopeArg::Project,
        "folder" => {
            let path = args::folder_path(input, "folder")?
                .ok_or_else(|| ToolError::new("bad_argument", "scope `folder` needs `folder`").with("argument", "folder"))?;
            let found = crate::ipc::project::read(state, |s| s.project.find_folder(&path).is_some())
                .ok_or_else(ToolError::no_project)?;
            if !found {
                return Err(ToolError::new("not_found", format!("there is no folder {:?}", args::join(&path))).with("folder", args::join(&path)));
            }
            ScopeArg::Folder { path }
        }
        "sprites" => ScopeArg::Sprites { sprites: sprites_of(ctx, input)? },
        other => {
            return Err(ToolError::new("bad_argument", format!("`scope` is {other:?}; use project, folder or sprites")).with("argument", "scope"));
        }
    };
    let format = match args::optional_str(input, "format")?.unwrap_or("sheet") {
        "sheet" => FormatArg::Sheet,
        "png" => FormatArg::Png,
        other => {
            return Err(ToolError::new("bad_argument", format!("`format` is {other:?}; use sheet or png")).with("argument", "format"));
        }
    };
    let json_format = match args::optional_str(input, "json_format")?.unwrap_or("sprixel") {
        "sprixel" => JsonFormatArg::Sprixel,
        "aseprite" => JsonFormatArg::Aseprite,
        other => {
            return Err(ToolError::new("bad_argument", format!("`json_format` is {other:?}; use sprixel or aseprite")).with("argument", "json_format"));
        }
    };
    let options = OptionsArg {
        scale: args::optional_u32(input, "scale")?.unwrap_or(1),
        spacing: args::optional_u32(input, "spacing")?.unwrap_or(0),
        naming: args::optional_str(input, "naming")?.map(str::to_string),
        json_format,
    };
    let output = args::optional_str(input, "output")?.unwrap_or(crate::export::DEFAULT_OUTPUT);
    check_output(output)?;
    check_naming(options.naming.as_deref())?;
    // A sheet or a set of images is built in memory: refuse a size that could
    // use all of it (an agent could ask for scale 8 of many frames).
    let size = export::summary(state, &scope, &format, &options).map_err(|e| ToolError::new(&e.code, e.message))?;
    let pixels = u64::from(size.sheet_width) * u64::from(size.sheet_height);
    if pixels > MAX_EXPORT_PIXELS {
        return Err(ToolError::new(
            "too_big",
            format!("this export would draw {pixels} pixels; the maximum is {MAX_EXPORT_PIXELS}. Export fewer sprites, or use a smaller scale"),
        )
        .with("pixels", pixels)
        .with("limit", MAX_EXPORT_PIXELS));
    }
    let written = export::run(state, &scope, &format, &options, output).map_err(|e| ToolError::new(&e.code, e.message))?;
    let files: Vec<String> = written.files.iter().map(|name| format!("{}/{name}", written.output)).collect();
    Ok(Output::Json(json!({ "output": written.output, "files": files })))
}

/// The most pixels an export may draw (64 million, about 256 MB in memory).
const MAX_EXPORT_PIXELS: u64 = 64 * 1024 * 1024;

/// The output folder must be a normal folder of the project where the person
/// keeps assets: not a hidden folder (like `.git`), not the folders that hold
/// the project files (`sprites`, `palettes`), and each name must be valid on
/// every system (no `CON`, no trailing dot). That `..`, absolute paths and
/// links never leave the project is checked by the export module.
fn check_output(output: &str) -> Result<(), ToolError> {
    let invalid = |why: &str| Err(ToolError::new("invalid_path", format!("`output` is not allowed: {why}")).with("argument", "output"));
    let parts: Vec<&str> = output.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".").collect();
    for (index, part) in parts.iter().enumerate() {
        if *part == ".." {
            continue; // refused by the export module as `outside_project`
        }
        if part.starts_with('.') {
            return invalid("a hidden folder cannot be used");
        }
        if index == 0 && ["sprites", "palettes"].iter().any(|reserved| part.eq_ignore_ascii_case(reserved)) {
            return invalid("this folder holds the files of the project");
        }
        if let Err(error) = crate::document::validate_name(part) {
            // A drive like `C:` or a path with `:` is refused by the export module.
            if part.contains(':') {
                continue;
            }
            return invalid(&error.message);
        }
    }
    Ok(())
}

/// The names of the files come from the naming pattern, so it must make names
/// that are valid on every system.
fn check_naming(naming: Option<&str>) -> Result<(), ToolError> {
    let Some(pattern) = naming else { return Ok(()) };
    let sample = pattern.replace("{sprite}", "sprite").replace("{animation}", "walk").replace("{frame}", "1");
    crate::document::validate_name(&sample).map_err(|e| {
        ToolError::new("bad_options", format!("the naming pattern makes a file name that is not valid: {}", e.message))
            .with("argument", "naming")
    })
}

fn sprites_of(ctx: &Ctx, input: &Value) -> Result<Vec<export::SpriteArg>, ToolError> {
    let names = input["sprites"]
        .as_array()
        .filter(|list| !list.is_empty())
        .ok_or_else(|| ToolError::new("bad_argument", "scope `sprites` needs `sprites`, a list of sprite names").with("argument", "sprites"))?;
    let state = ctx.core.host.project();
    let mut found = Vec::new();
    for name in names {
        let one = json!({ "name": name });
        let SpriteRef { folder, name } = crate::ipc::project::read(state, |s| args::sprite(&s.project, &one))
            .ok_or_else(ToolError::no_project)??;
        found.push(export::SpriteArg { folder, name });
    }
    Ok(found)
}
