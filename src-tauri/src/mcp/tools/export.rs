//! The `export` tool. It uses the export module of the app (`ipc::export`),
//! so a sprite sheet or PNG files are the same as the ones the person
//! exports. The files go to a folder inside the project and never anywhere
//! else; the path is checked by the export module (no `..`, no absolute path,
//! no link that leaves the project).

use super::args;
use crate::document::SpriteRef;
use crate::ipc::export::{self, FormatArg, OptionsArg, ScopeArg};
use crate::mcp::{Ctx, Need, Output, Tool, ToolError, ToolResult};
use serde_json::{Value, json};

pub fn tools() -> Vec<Tool> {
    vec![Tool {
        name: "export",
        description: "Exports sprites as files for a game engine, into a folder INSIDE the project (default `assets/sprites`). \
`format` `sheet` writes one PNG sprite sheet and a JSON file that describes it; `png` writes one PNG file for every frame. \
`scope` is `project` (all sprites), `folder` (with `folder`) or `sprites` (with `sprites`, a list of sprite names or paths). \
The answer lists the files written, as paths from the project folder. Exported files cannot be undone.",
        schema: json!({ "type": "object", "properties": {
            "scope": { "type": "string", "enum": ["project", "folder", "sprites"], "description": "Default `project`." },
            "folder": { "type": "string", "description": "For scope `folder`: a path like \"Characters\". Its sub-folders are included." },
            "sprites": { "type": "array", "items": { "type": "string" }, "description": "For scope `sprites`: names or paths like \"Characters/goblin\"." },
            "format": { "type": "string", "enum": ["sheet", "png"], "description": "Default `sheet`." },
            "scale": { "type": "integer", "enum": [1, 2, 4, 8], "description": "Default 1." },
            "spacing": { "type": "integer", "minimum": 0, "maximum": 16, "description": "Pixels between frames in the sheet. Default 0." },
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
    let options = OptionsArg {
        scale: args::optional_u32(input, "scale")?.unwrap_or(1),
        spacing: args::optional_u32(input, "spacing")?.unwrap_or(0),
        naming: args::optional_str(input, "naming")?.map(str::to_string),
    };
    let output = args::optional_str(input, "output")?.unwrap_or(crate::export::DEFAULT_OUTPUT);
    let written = export::run(state, &scope, &format, &options, output).map_err(|e| ToolError::new(&e.code, e.message))?;
    let files: Vec<String> = written.files.iter().map(|name| format!("{}/{name}", written.output)).collect();
    Ok(Output::Json(json!({ "output": written.output, "files": files })))
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
