use super::*;
use crate::document::{Color, Folder, Frame, Palette, Project, Sprite, SpriteRef};

fn project() -> Project {
    let mut p = Project::new("Dungeon Kit");
    p.palettes.push(Palette {
        name: "main".to_string(),
        colors: vec![Color { key: 'R', name: "red".to_string(), hex: "#FF0000".to_string() }],
    });
    let mut goblin = Sprite::new("goblin", 8, 8, "main").unwrap();
    goblin.frame_durations = vec![100; 14];
    goblin.layers[0].frames = vec![Frame::blank(8, 8); 14];
    p.root.folders.push(Folder {
        name: "Personajes".to_string(),
        folders: vec![],
        sprites: vec![goblin],
    });
    p.root.sprites.push(Sprite::new("tree", 8, 8, "main").unwrap());
    p
}

fn folder() -> Scope {
    Scope::Folder(vec!["Personajes".to_string()])
}

fn options(spacing: u32) -> Options {
    Options { spacing, ..Options::default() }
}

// Checks the spec scenario of the scope: the folder has 1 sprite and 14 frames,
// the sheet is 71x17 and the "Sheet + JSON" format writes 2 files.
#[test]
fn counts_for_a_folder() {
    let s = summarize(&project(), &folder(), Format::SheetJson, &options(1), "personajes").unwrap();
    assert_eq!((s.sprites, s.frames), (1, 14));
    assert_eq!((s.width, s.height), (71, 17));
    assert_eq!(s.files, ["personajes.png", "personajes.json"]);
}

// Checks that the whole project counts every sprite, and that "PNG files"
// lists one file per frame.
#[test]
fn counts_for_the_project_and_png_files() {
    let s = summarize(&project(), &Scope::Project, Format::PngFiles, &options(0), "x").unwrap();
    assert_eq!((s.sprites, s.frames, s.files.len()), (2, 15, 15));
}

// Checks that a selection counts only the chosen sprites, and that a name
// that does not exist is ignored.
#[test]
fn counts_for_a_selection() {
    let scope = Scope::Sprites(vec![SpriteRef::new(&[], "tree"), SpriteRef::new(&[], "ghost")]);
    let s = summarize(&project(), &scope, Format::SheetJson, &options(0), "x").unwrap();
    assert_eq!((s.sprites, s.frames), (1, 1));
}

// Checks the empty case: no sprites gives zeros and no files, not an error.
#[test]
fn an_empty_scope_gives_zeros() {
    let scope = Scope::Sprites(vec![]);
    let s = summarize(&project(), &scope, Format::SheetJson, &options(0), "x").unwrap();
    assert_eq!((s.sprites, s.frames, s.width, s.height), (0, 0, 0, 0));
    assert!(s.files.is_empty());
}

// Checks that the preview is the sheet at 1x even when the scale is bigger,
// and that an empty scope has no preview.
#[test]
fn the_preview_is_always_at_1x() {
    let big = Options { scale: 8, spacing: 1, ..Options::default() };
    let image = preview(&project(), &folder(), &big).unwrap();
    assert_eq!((image.width, image.height), (71, 17));
    assert_eq!(preview(&project(), &Scope::Sprites(vec![]), &big).unwrap_err().code, "nothing_to_export");
}

// Checks the default file names: the folder, the only sprite, or the project;
// lower case, `-` for spaces, and a fallback when nothing is left.
#[test]
fn default_names() {
    let p = project();
    assert_eq!(default_name(&p, &folder()), "personajes");
    assert_eq!(default_name(&p, &Scope::Project), "dungeon-kit");
    assert_eq!(default_name(&p, &Scope::Sprites(vec![SpriteRef::new(&[], "tree")])), "tree");
    assert_eq!(default_name(&Project::new("***"), &Scope::Project), "sprites");
}
