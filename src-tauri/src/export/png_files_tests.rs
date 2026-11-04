use super::*;
use crate::document::{Color, Frame, Palette, Project, Sprite, Tag, TagMode};

fn project() -> Project {
    let mut p = Project::new("Kit");
    p.palettes.push(Palette {
        name: "main".to_string(),
        colors: vec![Color { key: 'R', name: "red".to_string(), hex: "#FF0000".to_string() }],
    });
    p
}

fn sprite(name: &str, frames: usize) -> Sprite {
    let mut s = Sprite::new(name, 2, 2, "main").unwrap();
    s.frame_durations = vec![100; frames];
    s.layers[0].frames = vec![Frame { pixels: b"R...".to_vec() }; frames];
    s
}

fn tag(name: &str, from: u32, to: u32) -> Tag {
    Tag { name: name.to_string(), from, to, fps: 12, mode: TagMode::Loop }
}

fn names(files: &[ExportFile]) -> Vec<&str> {
    files.iter().map(|f| f.name.as_str()).collect()
}

fn export(p: &Project, naming: &str) -> Result<Vec<ExportFile>, ExportError> {
    let options = Options { naming: naming.to_string(), ..Options::default() };
    build_png_files(p, &Scope::Project, &options)
}

// Checks the spec scenario: the sprite "goblin" with the animation "walk" of
// 4 frames gives `goblin_walk_1.png` to `goblin_walk_4.png`.
#[test]
fn default_pattern_names_the_walk_frames() {
    let mut p = project();
    let mut goblin = sprite("goblin", 4);
    goblin.tags.push(tag("walk", 1, 4));
    p.root.sprites.push(goblin);
    let files = export(&p, DEFAULT_NAMING).unwrap();
    assert_eq!(
        names(&files),
        ["goblin_walk_1.png", "goblin_walk_2.png", "goblin_walk_3.png", "goblin_walk_4.png"]
    );
}

// Checks that frames are numbered inside their animation (walk on frames 3 and
// 4 is walk_1 and walk_2), and that a frame in no animation has no animation
// name, with no double separator.
#[test]
fn numbers_count_inside_the_animation() {
    let mut p = project();
    let mut goblin = sprite("goblin", 4);
    goblin.tags.push(tag("walk", 3, 4));
    p.root.sprites.push(goblin);
    let files = export(&p, DEFAULT_NAMING).unwrap();
    assert_eq!(names(&files), ["goblin_1.png", "goblin_2.png", "goblin_walk_1.png", "goblin_walk_2.png"]);
}

// Checks a custom pattern.
#[test]
fn a_custom_pattern_is_used() {
    let mut p = project();
    p.root.sprites.push(sprite("orc", 2));
    let files = export(&p, "{frame}-{sprite}").unwrap();
    assert_eq!(names(&files), ["1-orc.png", "2-orc.png"]);
}

// Checks that every file is a valid PNG of the scaled frame size.
#[test]
fn files_are_valid_pngs() {
    let mut p = project();
    p.root.sprites.push(sprite("orc", 1));
    let options = Options { scale: 4, ..Options::default() };
    let files = build_png_files(&p, &Scope::Project, &options).unwrap();
    let image = image::load_from_memory(&files[0].bytes).unwrap().to_rgba8();
    assert_eq!((image.width(), image.height()), (8, 8));
    assert_eq!(image.get_pixel(3, 3).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(4, 0).0, [0, 0, 0, 0]);
}

// Checks that two sprites with the same name in different folders do not
// overwrite each other: the second gets a `_2` suffix.
#[test]
fn equal_names_get_a_suffix() {
    use crate::document::Folder;
    let mut p = project();
    p.root.sprites.push(sprite("orc", 1));
    p.root.folders.push(Folder { name: "b".to_string(), folders: vec![], sprites: vec![sprite("orc", 1)] });
    let files = export(&p, DEFAULT_NAMING).unwrap();
    assert_eq!(names(&files), ["orc_1.png", "orc_1_2.png"]);
}

// Checks that a pattern that could write outside the folder is refused.
#[test]
fn patterns_with_slashes_or_dots_are_refused() {
    let mut p = project();
    p.root.sprites.push(sprite("orc", 1));
    for bad in ["../{sprite}", "a/{sprite}", "a\\{sprite}", "{sprite}..", "  "] {
        assert_eq!(export(&p, bad).unwrap_err().code, "bad_options", "{bad}");
    }
}

// Checks that the same export twice gives the same bytes.
#[test]
fn same_export_same_files() {
    let mut p = project();
    p.root.sprites.push(sprite("orc", 3));
    assert_eq!(export(&p, DEFAULT_NAMING).unwrap(), export(&p, DEFAULT_NAMING).unwrap());
}
